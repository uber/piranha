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

use crate::models::concrete_syntax::get_all_matches_for_concrete_syntax;
use crate::{
  models::Validator,
  utilities::{
    regex_utilities::get_all_matches_for_regex,
    tree_sitter_utilities::{get_all_matches_for_query, get_ts_query_parser, number_of_errors},
    Instantiate,
  },
};
use pyo3::prelude::pyclass;
use regex::Regex;
use serde_derive::Deserialize;
use std::collections::HashMap;
use tree_sitter::{Node, Query};

#[derive(Debug)]
pub struct ConcreteSyntax(pub String);
use super::{
  default_configs::{CONCRETE_SYNTAX_QUERY_PREFIX, REGEX_QUERY_PREFIX},
  matches::Match,
};

pub enum PatternType {
  Tsq,
  Regex,
  Unknown,
}

#[pyclass]
#[derive(Deserialize, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct CGPattern(pub String);

impl CGPattern {
  pub(crate) fn new(query: String) -> Self {
    Self(query)
  }

  pub(crate) fn pattern(&self) -> String {
    self.0.to_string()
  }

  pub(crate) fn extract_regex(&self) -> Result<Regex, regex::Error> {
    let mut _val = &self.pattern()[REGEX_QUERY_PREFIX.len()..];
    Regex::new(_val)
  }

  pub(crate) fn extract_concrete_syntax(&self) -> ConcreteSyntax {
    let mut _val = &self.pattern()[CONCRETE_SYNTAX_QUERY_PREFIX.len()..];
    ConcreteSyntax(_val.to_string())
  }

  pub(crate) fn pattern_type(&self) -> PatternType {
    match self.0.as_str() {
      pattern if pattern.starts_with("rgx") => PatternType::Regex,
      pattern if pattern.trim().starts_with('(') => PatternType::Tsq,
      _ => PatternType::Unknown,
    }
  }
}

impl Validator for CGPattern {
  fn validate(&self) -> Result<(), String> {
    if self.pattern().starts_with("rgx ") {
      return self
        .extract_regex()
        .map(|_| Ok(()))
        .unwrap_or(Err(format!("Cannot parse the regex - {}", self.pattern())));
    }
    if self.pattern().starts_with("cs ") {
      return Ok(());
    }
    let mut parser = get_ts_query_parser();
    parser
      .parse(self.pattern(), None)
      .filter(|x| number_of_errors(&x.root_node()) == 0)
      .map(|_| Ok(()))
      .unwrap_or(Err(format!(
        "Cannot parse the tree-sitter query - {}",
        self.pattern()
      )))
  }
}

impl Instantiate for CGPattern {
  fn instantiate(&self, substitutions: &HashMap<String, String>) -> Self {
    let substitutions = substitutions
      .iter()
      .map(|(k, v)| (k.to_string(), v.replace('\n', "\\n")))
      .collect();
    CGPattern::new(self.pattern().instantiate(&substitutions))
  }
}

#[derive(Debug)]
pub(crate) enum CompiledCGPattern {
  Q(Query),
  R(Regex),
  M(ConcreteSyntax),
}

impl CompiledCGPattern {
  /// Applies the CGPattern (self) upon the input `node`, and returns the first match
  /// # Arguments
  /// * `node` - the root node to apply the query upon
  /// * `source_code` - the corresponding source code string for the node.
  /// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
  pub(crate) fn get_match(&self, node: &Node, source_code: &str, recursive: bool) -> Option<Match> {
    if let Some(m) = self
      .get_matches(node, source_code.to_string(), recursive, None, None)
      .first()
    {
      return Some(m.clone());
    }
    None
  }

  /// Applies the pattern upon the given `node`, and gets all the matches
  pub(crate) fn get_matches(
    &self, node: &Node, source_code: String, recursive: bool, replace_node: Option<String>,
    replace_node_idx: Option<u8>,
  ) -> Vec<Match> {
    let code_str = source_code.as_bytes();
    match self {
      CompiledCGPattern::Q(query) => get_all_matches_for_query(
        node,
        source_code,
        query,
        recursive,
        replace_node,
        replace_node_idx,
      ),
      CompiledCGPattern::R(regex) => {
        get_all_matches_for_regex(node, source_code, regex, recursive, replace_node)
      }
      CompiledCGPattern::M(concrete_syntax) => {
        let matches = get_all_matches_for_concrete_syntax(
          node,
          code_str,
          concrete_syntax,
          recursive,
          replace_node,
        );
        matches.0
      }
    }
  }
}
