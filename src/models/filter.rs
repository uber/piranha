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

use derive_builder::Builder;
use getset::Getters;
use itertools::Itertools;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::Deserialize;
use tree_sitter::Node;

use crate::utilities::{
  gen_py_str_methods,
  tree_sitter_utilities::{get_all_matches_for_query, get_match_for_query, get_node_for_range},
};

use super::{rule::InstantiatedRule, rule_store::RuleStore, source_code_unit::SourceCodeUnit};

use crate::utilities::{tree_sitter_utilities::TSQuery, Instantiate};

use super::default_configs::{
  default_contains_at_least, default_contains_at_most, default_enclosing_node, default_queries,
};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Getters, Builder)]
#[pyclass]
pub struct Filter {
  /// AST patterns that some ancestor node of the primary match should comply
  #[builder(default = "default_enclosing_node()")]
  #[get = "pub"]
  #[pyo3(get)]
  enclosing_node: TSQuery,
  /// AST patterns that should not match any subtree of node matching `enclosing_node` pattern
  #[builder(default = "default_queries()")]
  #[get = "pub"]
  #[serde(default)]
  #[pyo3(get)]
  not_contains: Vec<TSQuery>,
  /// AST patterns that should match any subtree of node matching `enclosing_node` pattern
  #[builder(default = "default_queries()")]
  #[get = "pub"]
  #[serde(default)]
  #[pyo3(get)]
  contains: Vec<TSQuery>,

  /// Least number of matches we should find for the contains query
  #[builder(default = "default_contains_at_least()")]
  #[get = "pub"]
  #[serde(default)]
  #[pyo3(get)]
  at_least: u32,

  /// Most number of matches we should find for the contains query
  #[builder(default = "default_contains_at_most()")]
  #[get = "pub"]
  #[serde(default)]
  #[pyo3(get)]
  at_most: u32,
}

#[pymethods]
impl Filter {
  #[new]
  fn py_new(
    enclosing_node: String, not_contains: Option<Vec<String>>, contains: Option<Vec<String>>,
    at_least: Option<u32>, at_most: Option<u32>,
  ) -> Self {
    FilterBuilder::default()
      .enclosing_node(TSQuery::new(enclosing_node))
      .not_contains(
        not_contains
          .unwrap_or_default()
          .iter()
          .map(|x| TSQuery::new(x.to_string()))
          .collect_vec(),
      )
      .contains(
        contains
          .unwrap_or_default()
          .iter()
          .map(|x| TSQuery::new(x.to_string()))
          .collect_vec(),
      )
      .at_least(at_least.unwrap_or(1))
      .at_most(at_most.unwrap_or(u32::MAX))
      .build()
      .unwrap()
  }
  gen_py_str_methods!();
}

#[macro_export]
/// This macro constructs a FilterBuilder for creating filter queries. It provides a more "dynamic" way to use the builder pattern.
///
/// 'enclosing_node' is a required parameter that specifies the node to be inspected.
///
/// 'not_contains' and 'contains' are optional parameters, accepting a list of queries that should not and should match
/// within the 'enclosing_node' respectively.
///
/// 'at_least' and 'at_most' specify the inclusive range for the count of matches 'contains' queries should find within
/// the 'enclosing_node'. These parameters provide control over the desired quantity of matches.
///
/// Usage:
///
/// ```
/// filter! {
///   enclosing_node = "(method_declaration) @md",
///   not_contains= ["(method_invocation name: (_) @name)"],
///   contains= ["(parameter_list)"],
///   at_least = 1,
///   at_most = 10
/// }
/// ```
///
/// expands to
///
/// ```
/// FilterBuilder::default()
///      .enclosing_node(TSQuery::new("(method_declaration) @md"))
///      .not_contains(vec![TSQuery::new("(method_invocation name: (_) @name)")])
///      .contains(vec![TSQuery::new("(parameter_list)")])
///      .at_least(1)
///      .at_most(10)
///      .build().unwrap()
/// ```
///
macro_rules! filter {
  (enclosing_node = $enclosing_node:expr $(, not_contains= [$($q:expr,)*])? $(, contains= [$($p:expr,)*])? $(, at_least=$min:expr)? $(, at_most=$max:expr)?) => {
    $crate::models::filter::FilterBuilder::default()
      .enclosing_node($crate::utilities::tree_sitter_utilities::TSQuery::new($enclosing_node.to_string()))
      $(.not_contains(vec![$($crate::utilities::tree_sitter_utilities::TSQuery::new($q.to_string()),)*]))?
      $(.contains(vec![$($crate::utilities::tree_sitter_utilities::TSQuery::new($p.to_string()),)*]))?
      $(.at_least($min))?
      $(.at_most($max))?
      .build().unwrap()
  };
}

pub use filter;

impl Instantiate for Filter {
  /// Create a new query from `self` by updating the `query` and `replace` based on the substitutions.
  fn instantiate(&self, substitutions_for_holes: &HashMap<String, String>) -> Filter {
    Filter {
      enclosing_node: self.enclosing_node().instantiate(substitutions_for_holes),
      not_contains: self
        .not_contains()
        .iter()
        .map(|x| x.instantiate(substitutions_for_holes))
        .collect_vec(),
      contains: self
        .contains()
        .iter()
        .map(|x| x.instantiate(substitutions_for_holes))
        .collect_vec(),
      at_least: self.at_least,
      at_most: self.at_most,
    }
  }
}

// Implements instance methods related to applying a filter
impl SourceCodeUnit {
  pub(crate) fn is_satisfied(
    &self, node: Node, rule: &InstantiatedRule, substitutions: &HashMap<String, String>,
    rule_store: &mut RuleStore,
  ) -> bool {
    let mut updated_substitutions = self.piranha_arguments().input_substitutions();
    updated_substitutions.extend(substitutions.clone());
    rule
      .filters()
      .iter()
      .all(|filter| self._check(filter.clone(), node, rule_store, &updated_substitutions))
  }

  /// Determines if the given `node` meets the conditions specified by the `filter`.
  ///
  /// The `filter` is composed of:
  /// (i) `enclosing_node`, the node to inspect,
  /// (ii) `not_contains` and `contains`, optional sets of queries that should not and should match within the `enclosing_node`,
  /// (iii) `at_least` and `at_most`, optional parameters indicating the acceptable range of matches for `contains` within the `enclosing_node`.
  ///
  /// The function identifies the `enclosing_node` by traversing the ancestors of the `node`. Within this node:
  /// (i) if `not_contains` is provided, it ensures no sub-tree matches any of these queries,
  /// (ii) if `contains` is provided, it ensures the number sub-trees matching `contains` fall within the specified range.
  ///
  /// If these conditions hold, the function returns true, indicating the `node` meets the `filter`'s criteria.
  fn _check(
    &self, filter: Filter, node: Node, rule_store: &mut RuleStore,
    substitutions: &HashMap<String, String>,
  ) -> bool {
    let mut current_node = node;
    // This ensures that the below while loop considers the current node too when checking for filters.
    // It does not make sense to check for filter if current node is a "leaf" node.
    if node.child_count() > 0 {
      current_node = node.child(0).unwrap();
    }
    // Get the enclosing node matching the pattern specified in the filter (`filter.enclosing_node`)
    let mut matched_enclosing_node = false;
    while let Some(parent) = current_node.parent() {
      let instantiated_filter = filter.instantiate(substitutions);
      let enclosing_node_query_str = instantiated_filter.enclosing_node();
      if let Some(p_match) = get_match_for_query(
        &parent,
        self.code(),
        rule_store.query(enclosing_node_query_str),
        false,
      ) {
        matched_enclosing_node = true;
        let scope_node = get_node_for_range(
          self.root_node(),
          p_match.range().start_byte,
          p_match.range().end_byte,
        );
        for query_with_holes in filter.not_contains() {
          // Instantiate the query and check if there's a match within the scope node
          // If there is the filter is not satisfied
          let query = &rule_store.query(&query_with_holes.instantiate(substitutions));

          if get_match_for_query(&scope_node, self.code(), query, true).is_some() {
            return false;
          }
        }
        for query_with_holes in filter.contains() {
          // Instantiate the query and retrieve all matches within the scope node
          let query = &rule_store.query(&query_with_holes.instantiate(substitutions));
          let matches =
            get_all_matches_for_query(&scope_node, self.code().to_string(), query, true, None);
          let at_least = filter.at_least as usize;
          let at_most = filter.at_most as usize;
          // Validate if the count of matches falls within the expected range
          if !(at_least <= matches.len() && matches.len() <= at_most) {
            return false;
          }
        }
        break;
      }
      current_node = parent;
    }
    matched_enclosing_node
  }
}