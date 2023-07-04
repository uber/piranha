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

use super::{matches::Match, Validator};
use crate::utilities::{
  tree_sitter_utilities::{
    get_all_matches_for_query, get_match_for_query, get_ts_query_parser, number_of_errors,
  },
  Instantiate,
};
use pyo3::prelude::pyclass;
use regex::Regex;
use serde_derive::Deserialize;
use tree_sitter::{Node, Query};

#[pyclass]
#[derive(Deserialize, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct CGPattern(pub String);

impl CGPattern {
  pub(crate) fn new(query: String) -> Self {
    Self(query)
  }

  pub(crate) fn get_query(&self) -> String {
    self.0.to_string()
  }
}

impl Validator for CGPattern {
  fn validate(&self) -> Result<(), String> {
    if self.get_query().starts_with("rgx ") {
      let mut _val = self.get_query();
      _val.replace_range(..4, "rgx ");
      return Regex::new(_val.as_str())
        .map(|_| Ok(()))
        .unwrap_or(Err(format!(
          "Cannot parse the regex - {}",
          self.get_query()
        )));
    }
    let mut parser = get_ts_query_parser();
    parser
      .parse(self.get_query(), None)
      .filter(|x| number_of_errors(&x.root_node()) == 0)
      .map(|_| Ok(()))
      .unwrap_or(Err(format!("Cannot parse - {}", self.get_query())))
  }
}

impl Instantiate for CGPattern {
  fn instantiate(&self, substitutions: &HashMap<String, String>) -> Self {
    let substitutions = substitutions
      .iter()
      .map(|(k, v)| (k.to_string(), v.replace('\n', "\\n")))
      .collect();
    CGPattern::new(self.get_query().instantiate(&substitutions))
  }
}

#[derive(Debug)]
pub(crate) enum CompiledCGPattern {
  Q(Query),
  R(Regex), // Regex is not yet supported
}

impl CompiledCGPattern {
  /// Applies the pattern upon the given node, and gets all the matches
  pub(crate) fn get_match_for_capture_node(
    &self, node: &Node, source_code: &str, recursive: bool,
  ) -> Option<Match> {
    match self {
      CompiledCGPattern::Q(query) => get_match_for_query(node, source_code, query, recursive),
      CompiledCGPattern::R(_) => panic!("Regex is not yet supported!!!"),
    }
  }

  /// Applies the pattern upon the given `node`, and gets all the matches
  pub(crate) fn get_all_matches_for_capture_node(
    &self, node: &Node, source_code: String, recursive: bool, replace_node: Option<String>,
    replace_node_idx: Option<u8>,
  ) -> Vec<Match> {
    match self {
      CompiledCGPattern::Q(query) => get_all_matches_for_query(
        node,
        source_code,
        query,
        recursive,
        replace_node,
        replace_node_idx,
      ),
      CompiledCGPattern::R(_) => panic!("Regex is not yet supported!!!"),
    }
  }
}
