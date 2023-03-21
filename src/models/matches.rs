/*
Copyright (c) 2022 Uber Technologies, Inc.

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
use log::{debug, trace};
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::utilities::{
  gen_py_str_methods,
  tree_sitter_utilities::{get_all_matches_for_query, get_node_for_range},
  MapOfVec,
};

use super::{rule::InstantiatedRule, rule_store::RuleStore, source_code_unit::SourceCodeUnit};

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

  #[pyo3(get)]
  #[get]
  #[get_mut]
  associated_matches: HashMap<String, Vec<Range>>,
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
      associated_matches: HashMap::new(),
    }
  }

  pub(crate) fn expand_to_associated_matches(&mut self, code: &String) {
    let associated_ranges = self
      .associated_matches
      .values()
      .flatten()
      .sorted()
      .cloned()
      .collect_vec();
    let start_range = associated_ranges.first().cloned().unwrap_or(self.range);
    let end_range = associated_ranges.last().cloned().unwrap_or(self.range);
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

    // Return the first match that satisfies constraint of the rule
    for p_match in all_query_matches.iter_mut() {
      if rule.rule().name().contains("simplify_if_statement_false") {
        print!("Here");
      }
      let matched_node = get_node_for_range(
        self.root_node(),
        p_match.range().start_byte,
        p_match.range().end_byte,
      );
      if self.is_satisfied(matched_node, rule, p_match.matches(), rule_store) {
        self.populate_associated_elements(&matched_node, p_match);
        trace!("Found match {:#?}", p_match);
        output.push(p_match.clone());
      }
    }
    debug!("Matches found {}", output.len());
    output
  }

  fn populate_associated_elements(&self, node: &Node, mtch: &mut Match) {
    self.get_trailing_associated_elements(node, mtch);
    self.get_leading_associated_elements(node, mtch);
  }

  fn get_trailing_associated_elements(&self, node: &Node, mtch: &mut Match) {
    let mut current_node = *node;
    let mut buf = 2;
    let mut found_comment = false;
    let mut found_comma = false;
    loop {
      if current_node.next_sibling().is_some() {
        while let Some(next_sibling) = current_node.next_sibling() {
          let content = next_sibling.utf8_text(self.code().as_bytes()).unwrap();
          if content.trim().eq(",") && !found_comma {
            mtch
              .associated_matches
              .collect("Comma".to_string(), Range::from(next_sibling.range()));
            current_node = next_sibling;
            found_comma = true;
          } else if self.is_comment(next_sibling.kind().to_string())
            && next_sibling.start_position().row == mtch.range().end_point.row
          {
            mtch
              .associated_matches
              .collect("Comment".to_string(), Range::from(next_sibling.range()));
            current_node = next_sibling;
            found_comment = true;
          } else {
            break;
          }
        }
      }
      if buf >= 0 && (!found_comma || !found_comment) {
        if let Some(p) = current_node.parent() {
          current_node = p;
          buf -= 1;
          continue;
        }
      }
      break;
    }
  }

  fn get_leading_associated_elements(&self, node: &Node, mtch: &mut Match) {
    let mut current_node = *node;
    let mut buf = 2;
    let mut found_comment = false;
    let mut found_comma = mtch.associated_matches.contains_key("Comma");
    loop {
      if current_node.prev_sibling().is_some() {
        while let Some(previous_sibling) = current_node.prev_sibling() {
          let content = previous_sibling.utf8_text(self.code().as_bytes()).unwrap();
          if content.trim().eq(",") && !found_comma {
            mtch
              .associated_matches
              .collect("Comma".to_string(), Range::from(previous_sibling.range()));
            current_node = previous_sibling;
            found_comma = true;
            continue;
          } else if self._is_comment_safe_to_delete(&previous_sibling, node, mtch) {
            mtch
              .associated_matches
              .collect("Comment".to_string(), Range::from(previous_sibling.range()));
            current_node = previous_sibling;
            found_comment = true;
            continue;
          }
          break;
        }
      }
      if buf >= 0 && (!found_comma || !found_comment) {
        if let Some(p) = current_node.parent() {
          current_node = p;
          buf -= 1;
          continue;
        }
      }
      break;
    }
  }

  fn _is_comment_safe_to_delete(&self, comment: &Node, deleted_node: &Node, mtch: &Match) -> bool {
    if !self.is_comment(comment.kind().to_string()) {
      return false;
    }
    if comment.range().start_point.row == deleted_node.range().start_point.row
      && comment.range().end_point.row == deleted_node.range().start_point.row
    {
      return true;
    }

    if let Some(previous_node) = comment.prev_sibling() {
      if self.overlaps(comment, &previous_node) {
        return false;
      }
    }
    if let Some(next_node) = comment.next_sibling() {
      if next_node.start_byte() < deleted_node.start_byte() {
        let associated_ranges = mtch
          .associated_matches
          .values()
          .flatten()
          .sorted()
          .cloned()
          .collect_vec();
        let start_range = associated_ranges
          .first()
          .cloned()
          .unwrap_or(Range::from(deleted_node.range()));
        if next_node.start_byte() < start_range.start_byte {
          return false;
        }
      }
    }
    true
  }

  fn overlaps(&self, node_1: &Node, node_2: &Node) -> bool {
    (node_1.start_position().row < node_2.start_position().row
      && node_2.start_position().row < node_1.end_position().row)
      || (node_1.start_position().row < node_2.end_position().row
        && node_2.end_position().row < node_1.end_position().row)
  }
}
