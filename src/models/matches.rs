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

use getset::{Getters, MutGetters};
use itertools::Itertools;
use log::trace;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::utilities::{
  gen_py_str_methods,
  tree_sitter_utilities::{get_all_matches_for_query, get_node_for_range},
};

use super::{
  piranha_arguments::PiranhaArguments, rule::InstantiatedRule, rule_store::RuleStore,
  source_code_unit::SourceCodeUnit,
};

#[derive(Serialize, Debug, Clone, Getters, MutGetters, Deserialize)]
#[pyclass]
pub(crate) struct Match {
  // Code snippet that matched
  #[get = "pub"]
  #[pyo3(get)]
  matched_string: String,
  // Range of the entire AST node captured by the match
  #[pyo3(get)]
  range: Range,
  // The mapping between tags and string representation of the AST captured.
  #[pyo3(get)]
  #[get = "pub"]
  matches: HashMap<String, String>,
  // Captures the range of the associated comma
  #[get]
  #[get_mut]
  #[serde(skip)]
  associated_comma: Option<Range>,
  // Captures the range(s) of the associated comments
  #[get]
  #[get_mut]
  #[serde(skip)]
  associated_comments: Vec<Range>,
}
gen_py_str_methods!(Match);

impl Match {
  pub(crate) fn new(
    matched_string: String, range: tree_sitter::Range, matches: HashMap<String, String>,
  ) -> Self {
    Self {
      matched_string,
      range: Range::from(range),
      matches,
      associated_comma: None,
      associated_comments: Vec::new(),
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
  pub(crate) fn expand_to_associated_matches(&mut self, code: &String) {
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

  /// Get the edit's replacement range.
  pub(crate) fn range(&self) -> tree_sitter::Range {
    tree_sitter::Range {
      start_byte: self.range.start_byte,
      end_byte: self.range.end_byte,
      start_point: tree_sitter::Point {
        row: self.range.start_point.row,
        column: self.range.start_point.column,
      },
      end_point: tree_sitter::Point {
        row: self.range.end_point.row,
        column: self.range.end_point.column,
      },
    }
  }

  // Populates the leading and trailing comma and comment ranges for the match.
  fn populate_associated_elements(
    &mut self, node: &Node, code: &String, piranha_arguments: &PiranhaArguments,
  ) {
    self.get_associated_elements(node, code, piranha_arguments, true);
    self.get_associated_elements(node, code, piranha_arguments, false);
  }

  /// Get the associated elements for the match.
  /// We currently capture leading and trailing comments and commas.
  fn get_associated_elements(
    &mut self, node: &Node, code: &String, piranha_arguments: &PiranhaArguments, trailing: bool,
  ) {
    let mut current_node = *node;
    let mut buf = *piranha_arguments.cleanup_comments_buffer();
    let mut found_comment = !self.associated_comments().is_empty();
    let mut found_comma = self.associated_comma().is_some();
    loop {
      // If we are looking for trailing elements, we start from the next sibling of the node
      // Else we start from the previous sibling of the node
      while let Some(sibling) = if trailing {
        current_node.next_sibling()
      } else {
        current_node.prev_sibling()
      } {
        let content = sibling.utf8_text(code.as_bytes()).unwrap();
        // Check if the sibling is a comment
        if !found_comma && content.trim().eq(",") {
          // Add the comma to the associated matches
          self.associated_comma = Some(Range::from(sibling.range()));
          current_node = sibling;
          found_comma = true;
          continue; // Continue the inner loop (i.e. evaluate next sibling)
        } else if self._is_comment_safe_to_delete(&sibling, node, piranha_arguments, trailing) {
          // Add the comment to the associated matches
          self.associated_comments.push(Range::from(sibling.range()));
          current_node = sibling;
          found_comment = true;
          continue; // Continue the inner loop (i.e. evaluate next sibling)
        }
        break; // Break the inner loop
      }

      let parent = current_node.parent();
      // If buf is <0 or we have found a comment and a comma, we break
      if buf < 0 || (found_comma && found_comment) || parent.is_none() {
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
  serde_derive::Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize,
)]
#[pyclass]
struct Range {
  #[pyo3(get)]
  start_byte: usize,
  #[pyo3(get)]
  end_byte: usize,
  #[pyo3(get)]
  start_point: Point,
  #[pyo3(get)]
  end_point: Point,
}

impl From<tree_sitter::Range> for Range {
  fn from(range: tree_sitter::Range) -> Self {
    Self {
      start_byte: range.start_byte,
      end_byte: range.end_byte,
      start_point: Point {
        row: range.start_point.row,
        column: range.start_point.column,
      },
      end_point: Point {
        row: range.end_point.row,
        column: range.end_point.column,
      },
    }
  }
}
gen_py_str_methods!(Range);

/// A range of positions in a multi-line text document, both in terms of bytes and of
/// rows and columns.
#[derive(
  serde_derive::Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize,
)]
#[pyclass]
struct Point {
  #[pyo3(get)]
  row: usize,
  #[pyo3(get)]
  column: usize,
}
gen_py_str_methods!(Point);

// Implements instance methods related to getting matches for rule
impl SourceCodeUnit {
  /// Gets the first match for the rule in `self`
  pub(crate) fn get_matches(
    &self, rule: &InstantiatedRule, rule_store: &mut RuleStore, node: Node, recursive: bool,
  ) -> Vec<Match> {
    let mut output: Vec<Match> = vec![];
    // Get all matches for the query in the given scope `node`.
    let replace_node_tag = if rule.rule().is_match_only_rule() || rule.rule().is_dummy_rule() {
      None
    } else {
      Some(rule.replace_node())
    };
    let mut all_query_matches = get_all_matches_for_query(
      &node,
      self.code().to_string(),
      rule_store.query(&rule.query()),
      recursive,
      replace_node_tag,
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
