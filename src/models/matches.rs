/*
Copyright (c) 2023 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use std::collections::HashMap;

use crate::utilities::tree_sitter_utilities::get_node_for_range;
use getset::{Getters, MutGetters};
use itertools::Itertools;
use log::trace;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::{Deserialize, Serialize};
use tree_sitter::Node;

use super::{
  piranha_arguments::PiranhaArguments, rule::InstantiatedRule, rule_store::RuleStore,
  source_code_unit::SourceCodeUnit,
};

#[derive(Serialize, Debug, Default, Clone, Getters, MutGetters, Deserialize)]
#[pyclass]
pub struct Match {
  // Code snippet that matched
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) matched_string: String,
  // Range of the entire AST node captured by the match
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) range: Range,
  // The mapping between tags and string representation of the AST captured.
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) matches: HashMap<String, String>,
  // Captures the range of the associated comma
  #[get]
  #[get_mut]
  #[serde(skip)]
  pub(crate) associated_comma: Option<Range>,
  // Captures the range(s) of the associated comments
  #[get]
  #[get_mut]
  #[serde(skip)]
  pub(crate) associated_comments: Vec<Range>,
  #[get]
  #[get_mut]
  #[serde(skip)]
  pub(crate) associated_leading_empty_lines: Vec<Range>,
}

#[pymethods]
impl Match {
  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}

impl Match {
  pub(crate) fn from_regex(
    mtch: &regex::Match, matches: HashMap<String, String>, source_code: &str,
  ) -> Self {
    Match {
      matched_string: mtch.as_str().to_string(),
      range: Range::from_regex_match(mtch, source_code),
      matches,
      associated_comma: None,
      associated_comments: Vec::new(),
      associated_leading_empty_lines: Vec::new(),
    }
  }

  ///
  /// Returns the first and last associated ranges for the match.
  /// If there are no associated ranges, returns the range of the match itself.
  fn get_first_and_last_associated_ranges(&self) -> (Range, Range) {
    // Sort all the associated ranges
    let associated_ranges = [
      self.associated_comma().iter().collect_vec(),
      self.associated_comments().iter().collect_vec(),
      self.associated_leading_empty_lines().iter().collect_vec(),
    ]
    .concat()
    .iter()
    .sorted()
    .cloned()
    .collect_vec();
    let start_range = associated_ranges.first().cloned().unwrap_or(&self.range);
    let end_range = associated_ranges.last().cloned().unwrap_or(&self.range);
    (*start_range, *end_range)
  }

  /// Merge the associated matches of the given match into the current match.
  /// It basically extends the range to include the first and last associated match of the given match.
  pub(crate) fn expand_to_associated_matches(&mut self, code: &str) {
    let (start_range, end_range) = self.get_first_and_last_associated_ranges();
    if start_range.start_byte < self.range.start_byte {
      self.range.start_byte = start_range.start_byte;
      self.range.start_point = start_range.start_point;
    }
    if end_range.end_byte > self.range.end_byte {
      self.range.end_byte = end_range.end_byte;
      self.range.end_point = end_range.end_point;
    }
    self.matched_string = code[self.range.start_byte..self.range.end_byte].to_string()
  }

  // Populates the leading and trailing comma and comment ranges for the match.
  fn populate_associated_elements(
    &mut self, node: &Node, code: &String, piranha_arguments: &PiranhaArguments,
  ) {
    self.get_associated_elements(node, code, piranha_arguments, true);
    self.get_associated_elements(node, code, piranha_arguments, false);
    self.get_associated_leading_empty_lines(node, code);
  }

  fn get_associated_leading_empty_lines(&mut self, matched_node: &Node, code: &String) -> () {
    if let Some(empty_range) = self.find_empty_line_range(code, matched_node.start_byte()) {
      let skipped_range = Range {
        start_byte: empty_range.start,
        end_byte: empty_range.end,
        start_point: position_for_offset(code.as_bytes(), empty_range.start),
        end_point: position_for_offset(code.as_bytes(), empty_range.end),
      };
      self
        .associated_leading_empty_lines_mut()
        .push(skipped_range);
    }
  }

  fn find_empty_line_range(&self, code: &str, start_byte: usize) -> Option<std::ops::Range<usize>> {
    let mut end_byte = start_byte;
    let code_bytes = code.as_bytes();
    let mut found = false;
    while end_byte > 0 {
      let prev_char = code_bytes[end_byte - 1] as char;
      if prev_char.is_whitespace() {
        end_byte -= 1;
        if prev_char == '\n' {
          found = true;
          break;
        }
      } else {
        break;
      }
    }
    if found {
      Some(end_byte..start_byte)
    } else {
      None
    }
  }
  fn found_comma(&self) -> bool {
    self.associated_comma().is_some()
  }

  fn found_comment(&self) -> bool {
    !self.associated_comments().is_empty()
  }

  /// Get the associated elements for the match.
  /// We currently capture leading and trailing comments and commas.
  fn get_associated_elements(
    &mut self, node: &Node, code: &String, piranha_arguments: &PiranhaArguments, trailing: bool,
  ) {
    let mut current_node = *node;
    let mut buf = *piranha_arguments.cleanup_comments_buffer();
    loop {
      // If we are looking for trailing elements, we start from the next sibling of the node
      // Else we start from the previous sibling of the node
      while let Some(sibling) = if trailing {
        current_node.next_sibling()
      } else {
        current_node.prev_sibling()
      } {
        // Check if the sibling is a comma
        if !self.found_comma() && self.is_comma_safe_to_delete(&sibling, code, trailing) {
          // Add the comma to the associated matches
          self.associated_comma = Some(sibling.range().into());
          current_node = sibling;
          continue; // Continue the inner loop (i.e. evaluate next sibling)
        } else if self._is_comment_safe_to_delete(&sibling, node, piranha_arguments, trailing) {
          // Add the comment to the associated matches
          self.associated_comments.push(sibling.range().into());
          current_node = sibling;
          continue; // Continue the inner loop (i.e. evaluate next sibling)
        }
        break; // Break the inner loop
      }

      let parent = current_node.parent();
      // If buf is <0 or we have found a comment and a comma, we break
      if buf < 0 || (self.found_comma() && self.found_comment()) || parent.is_none() {
        break; // Break the outer loop
      }
      current_node = parent.unwrap();
      buf -= 1;
      continue; // Continue the outer loop (i.e. lookup parent's siblings for comma/comment)
    }
  }

  /// Checks if the given node kind is a comment in the language (determined from piranha arguments)
  fn is_comment(&self, kind: String, piranha_arguments: &PiranhaArguments) -> bool {
    *piranha_arguments.cleanup_comments()
      && piranha_arguments.language().comment_nodes().contains(&kind)
  }
  // Checks if the given node is a comma
  fn is_comma(&self, code: &str, node: &Node) -> bool {
    let content = node.utf8_text(code.as_bytes()).unwrap();
    return content.trim().eq(",");
  }

  /// Checks whether it is safe to delete the provided comma node, by checking if
  /// no neighbor node lies between the comma and the of nodes intended to be deleted.
  ///
  /// When `trailing` is true, it considers the previous node sibling (wrt the comma
  /// Otherwise, it considers the next sibling (wrt the comma)
  fn is_comma_safe_to_delete(&self, comma: &Node, code: &str, trailing: bool) -> bool {
    if !self.is_comma(code, comma) {
      return false;
    }
    let (start_range, end_range) = self.get_first_and_last_associated_ranges();

    if trailing {
      if let Some(prev_node) = comma.prev_sibling() {
        // Ensure that the previous node's end byte is not beyond the deleted node's end byte
        if prev_node.end_byte() > end_range.end_byte {
          return false;
        }
      }
    } else if let Some(next_node) = comma.next_sibling() {
      // Ensure that the next node's start byte is not before the deleted node's start byte
      if next_node.start_byte() < start_range.start_byte {
        return false;
      }
    }

    // It is safe to delete the comma node
    true
  }

  /// Checks if the given comment is safe to delete.
  fn _is_comment_safe_to_delete(
    &mut self, comment: &Node, deleted_node: &Node, piranha_arguments: &PiranhaArguments,
    trailing: bool,
  ) -> bool {
    // Check if the comment is a comment in the language
    if !self.is_comment(comment.kind().to_string(), piranha_arguments) {
      return false;
    }
    // If trailing, check if the comment is on the same line as the deleted node
    // i.e. where the deleted node ends or starts
    let is_on_same_line = comment.range().start_point.row == deleted_node.range().end_point.row
      || comment.range().start_point.row == deleted_node.range().start_point.row;

    // If trailing, return if the comment is on the same line as the deleted node
    if trailing {
      return is_on_same_line;
    }
    // If not trailing, return if the comment is on the same line as the deleted node
    if is_on_same_line {
      return true;
    }
    // Check if the previous node does not overlap with the comment
    if let Some(previous_node) = comment.prev_sibling() {
      if self.overlaps(comment, &previous_node) {
        return false;
      }
    }
    // Check if there exists no node between the comment and the deleted node
    if let Some(next_node) = comment.next_sibling() {
      if next_node.start_byte() < deleted_node.start_byte() {
        let (start_range, _) = self.get_first_and_last_associated_ranges();
        if next_node.start_byte() < start_range.start_byte {
          return false;
        }
      }
    }
    true
  }

  /// Checks if the given node_1 overlaps with the given node_2
  fn overlaps(&self, node_1: &Node, node_2: &Node) -> bool {
    (node_1.start_position().row < node_2.start_position().row
      && node_2.start_position().row < node_1.end_position().row)
      || (node_1.start_position().row < node_2.end_position().row
        && node_2.end_position().row < node_1.end_position().row)
  }
}
/// A range of positions in a multi-line text document, both in terms of bytes and of
/// rows and columns.
/// Note `LocalRange` derives serialize.
#[derive(
  serde_derive::Serialize,
  Clone,
  Copy,
  Debug,
  PartialEq,
  Default,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Deserialize,
  Getters,
)]
#[pyclass]
pub struct Range {
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) start_byte: usize,
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) end_byte: usize,
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) start_point: Point,
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) end_point: Point,
}

impl From<tree_sitter::Range> for Range {
  fn from(range: tree_sitter::Range) -> Self {
    Self {
      start_byte: range.start_byte,
      end_byte: range.end_byte,
      start_point: range.start_point.into(),
      end_point: range.end_point.into(),
    }
  }
}

#[pymethods]
impl Range {
  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}

impl Range {
  pub(crate) fn from_regex_match(mtch: &regex::Match, source_code: &str) -> Self {
    Self {
      start_byte: mtch.start(),
      end_byte: mtch.end(),
      start_point: position_for_offset(source_code.as_bytes(), mtch.start()),
      end_point: position_for_offset(source_code.as_bytes(), mtch.end()),
    }
  }
}

// Finds the position (col and row number) for a given offset.
// Copied from tree-sitter tests [https://github.com/tree-sitter/tree-sitter/blob/d0029a15273e526925a764033e9b7f18f96a7ce5/cli/src/parse.rs#L364]
fn position_for_offset(input: &[u8], offset: usize) -> Point {
  let mut result = Point { row: 0, column: 0 };
  for c in &input[0..offset] {
    if *c as char == '\n' {
      result.row += 1;
      result.column = 0;
    } else {
      result.column += 1;
    }
  }
  result
}

/// A range of positions in a multi-line text document, both in terms of bytes and of
/// rows and columns.
#[derive(
  serde_derive::Serialize,
  Clone,
  Copy,
  Debug,
  Default,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Deserialize,
  Getters,
)]
#[pyclass]
pub struct Point {
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) row: usize,
  #[pyo3(get)]
  #[get = "pub"]
  pub(crate) column: usize,
}

impl From<Point> for tree_sitter::Point {
  fn from(value: Point) -> Self {
    tree_sitter::Point {
      row: *value.row(),
      column: *value.column(),
    }
  }
}

impl From<tree_sitter::Point> for Point {
  fn from(point: tree_sitter::Point) -> Self {
    Self {
      row: point.row,
      column: point.column,
    }
  }
}

#[pymethods]
impl Point {
  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}

// Implements instance methods related to getting matches for rule
impl SourceCodeUnit {
  /// Gets the first match for the rule in `self`
  pub(crate) fn get_matches(
    &self, rule: &InstantiatedRule, rule_store: &mut RuleStore, node: Node, recursive: bool,
  ) -> Vec<Match> {
    let mut output: Vec<Match> = vec![];
    // Get all matches for the query in the given scope `node`.
    let (replace_node_tag, replace_node_idx) =
      if rule.rule().is_match_only_rule() || rule.rule().is_dummy_rule() {
        (None, None)
      } else {
        (rule.replace_node(), rule.replace_idx())
      };

    let pattern = rule_store.query(&rule.query());
    let mut all_query_matches = pattern.get_matches(
      &node,
      self.code().to_string(),
      recursive,
      replace_node_tag,
      replace_node_idx,
    );

    // Applies the filter and returns the first element
    for p_match in all_query_matches.iter_mut() {
      let matched_node = get_node_for_range(
        self.root_node(),
        p_match.range().start_byte,
        p_match.range().end_byte,
      );
      if self.is_satisfied(matched_node, rule, p_match.matches(), rule_store) {
        p_match.populate_associated_elements(&matched_node, self.code(), self.piranha_arguments());
        trace!("Found match {:#?}", p_match);
        output.push(p_match.clone());
      }
    }
    trace!("Matches found {}", output.len());
    output
  }
}
