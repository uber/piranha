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

use getset::Getters;
use log::{debug, trace};
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::utilities::{
  gen_py_str_methods,
  tree_sitter_utilities::{get_all_matches_for_query, get_node_for_range},
};

use super::{rule::InstantiatedRule, rule_store::RuleStore, source_code_unit::SourceCodeUnit};

#[derive(Serialize, Debug, Clone, Getters, Deserialize)]
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
}
gen_py_str_methods!(Match);

impl Match {
  pub(crate) fn new(
    matched_string: String, range: tree_sitter::Range, matches: HashMap<String, String>,
  ) -> Self {
    Self {
      matched_string,
      range: Range {
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
      },
      matches,
    }
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
    let all_query_matches = get_all_matches_for_query(
      &node,
      self.code().to_string(),
      rule_store.query(&rule.query()),
      recursive,
      replace_node_tag,
    );

    // Return the first match that satisfies constraint of the rule
    for p_match in all_query_matches {
      let matched_node = get_node_for_range(
        self.root_node(),
        p_match.range().start_byte,
        p_match.range().end_byte,
      );

      if self.is_satisfied(matched_node, rule, p_match.matches(), rule_store) {
        trace!("Found match {:#?}", p_match);
        output.push(p_match);
      }
    }
    debug!("Matches found {}", output.len());
    output
  }
}
