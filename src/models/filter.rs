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
  default_contains_at_least, default_contains_at_most, default_contains_query,
  default_enclosing_node, default_not_contains_queries, default_not_enclosing_node,
};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Getters, Builder)]
#[pyclass]
pub struct Filter {
  /// AST patterns that some ancestor node of the primary match should match
  #[builder(default = "default_enclosing_node()")]
  #[get = "pub"]
  #[serde(default = "default_enclosing_node")]
  #[pyo3(get)]
  enclosing_node: TSQuery,
  /// AST patterns NO ancestor node of the primary match should match
  #[builder(default = "default_not_enclosing_node()")]
  #[get = "pub"]
  #[serde(default = "default_not_enclosing_node")]
  #[pyo3(get)]
  not_enclosing_node: TSQuery,
  /// AST patterns that should not match any subtree of node matching `enclosing_node` pattern
  #[builder(default = "default_not_contains_queries()")]
  #[get = "pub"]
  #[serde(default = "default_not_contains_queries")]
  #[pyo3(get)]
  not_contains: Vec<TSQuery>,
  /// AST patterns that should match any subtree of node matching `enclosing_node` pattern
  #[builder(default = "default_contains_query()")]
  #[get = "pub"]
  #[serde(default = "default_contains_query")]
  #[pyo3(get)]
  contains: TSQuery,
  /// Least number of matches we should find for the contains query
  #[builder(default = "default_contains_at_least()")]
  #[get = "pub"]
  #[serde(default = "default_contains_at_least")]
  #[pyo3(get)]
  at_least: u32,
  /// Most number of matches we should find for the contains query
  #[builder(default = "default_contains_at_most()")]
  #[get = "pub"]
  #[serde(default = "default_contains_at_most")]
  #[pyo3(get)]
  at_most: u32,
}

#[pymethods]
impl Filter {
  #[new]
  fn py_new(
    enclosing_node: Option<String>, not_enclosing_node: Option<String>,
    not_contains: Option<Vec<String>>, contains: Option<String>, at_least: Option<u32>,
    at_most: Option<u32>,
  ) -> Self {
    FilterBuilder::default()
      .enclosing_node(TSQuery::new(enclosing_node.unwrap_or_default()))
      .not_enclosing_node(TSQuery::new(not_enclosing_node.unwrap_or_default()))
      .not_contains(
        not_contains
          .unwrap_or_default()
          .iter()
          .map(|x| TSQuery::new(x.to_string()))
          .collect_vec(),
      )
      .contains(TSQuery::new(contains.unwrap_or_default()))
      .at_least(at_least.unwrap_or(default_contains_at_least()))
      .at_most(at_most.unwrap_or(default_contains_at_most()))
      .build()
      .unwrap()
  }
  gen_py_str_methods!();
}

#[macro_export]
/// This macro constructs a FilterBuilder for creating filter queries. It provides a more "dynamic" way to use the builder pattern.
///
/// 'enclosing_node' is an optional parameter that specifies the node to be inspected. If it is not provided
/// piranha will check the filters against the matched node
///
/// 'not_enclosing_node' is an optional parameter that specifies the nodes that should not enclose the matched node
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
///   not_enclosing_node = "(while_statement) @wt",
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
///      .not_enclosing_node(TSQuery::new("(while_statement) @wt"))
///      .not_contains(vec![TSQuery::new("(method_invocation name: (_) @name)")])
///      .contains(TSQuery::new("(parameter_list)"))
///      .at_least(1)
///      .at_most(10)
///      .build().unwrap()
/// ```
///
macro_rules! filter {
  ($(enclosing_node = $enclosing_node:expr)? $(, not_enclosing_node=$not_enclosing_node:expr)? $(, not_contains= [$($q:expr,)*])? $(, contains= $p:expr)? $(, at_least=$min:expr)? $(, at_most=$max:expr)?) => {
    $crate::models::filter::FilterBuilder::default()
      $(.enclosing_node($crate::utilities::tree_sitter_utilities::TSQuery::new($enclosing_node.to_string())))?
      $(.not_enclosing_node($crate::utilities::tree_sitter_utilities::TSQuery::new($not_enclosing_node.to_string())))?
      $(.not_contains(vec![$($crate::utilities::tree_sitter_utilities::TSQuery::new($q.to_string()),)*]))?
      $(.contains($crate::utilities::tree_sitter_utilities::TSQuery::new($p.to_string())))?
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
      not_enclosing_node: self
        .not_enclosing_node()
        .instantiate(substitutions_for_holes),
      not_contains: self
        .not_contains()
        .iter()
        .map(|x| x.instantiate(substitutions_for_holes))
        .collect_vec(),
      contains: self.contains().instantiate(substitutions_for_holes),
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
  /// (i) `enclosing_node`, the node to inspect, optional. If not provided we check whether the contains or non_contains are satisfied in the current node.
  /// (ii) `not_contains` and `contains`, optional queries that should not and should match within the `enclosing_node`,
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
    let mut node_to_check = node;
    let instantiated_filter = filter.instantiate(substitutions);

    // Check if no ancestor matches the query for not_enclosing_node
    if !self._check_not_enclosing_node(rule_store, node_to_check, &instantiated_filter) {
      return false;
    }
    // If an enclosing node is provided
    let query = instantiated_filter.enclosing_node();
    if !query.get_query().is_empty() {
      if let Some(result) = self._match_ancestor(rule_store, node_to_check, query) {
        node_to_check = result;
      } else {
        return false;
      }
    }

    self._check_filter_not_contains(&instantiated_filter, rule_store, &node_to_check)
      && self._check_filter_contains(&instantiated_filter, rule_store, &node_to_check)
  }

  fn _check_not_enclosing_node(
    &self, rule_store: &mut RuleStore, node_to_check: Node, instantiated_filter: &Filter,
  ) -> bool {
    let query = instantiated_filter.not_enclosing_node();
    if !query.get_query().is_empty() {
      // No ancestor should match with it
      if self
        ._match_ancestor(rule_store, node_to_check, query)
        .is_some()
      {
        return false;
      }
    }
    true
  }

  /// Search for any ancestor of `node` (including itself) that matches `query_str`
  fn _match_ancestor(
    &self, rule_store: &mut RuleStore, node: Node, ts_query: &TSQuery,
  ) -> Option<Node> {
    let mut current_node = node;

    // This ensures that the below while loop considers the current node too when checking for filters.
    if current_node.child_count() > 0 {
      current_node = current_node.child(0).unwrap();
    }

    while let Some(parent) = current_node.parent() {
      if let Some(p_match) =
        get_match_for_query(&parent, self.code(), rule_store.query(ts_query), false)
      {
        let matched_ancestor = get_node_for_range(
          self.root_node(),
          p_match.range().start_byte,
          p_match.range().end_byte,
        );
        return Some(matched_ancestor);
      }
      current_node = parent;
    }
    None
  }

  fn _check_filter_contains(
    &self, filter: &Filter, rule_store: &mut RuleStore, scope_node: &Node,
  ) -> bool {
    // If the query is empty
    let ts_query = filter.contains();
    if ts_query.get_query().is_empty() {
      return true;
    }

    //  Retrieve all matches within the scope node
    let contains_query = &rule_store.query(filter.contains());
    let matches = get_all_matches_for_query(
      scope_node,
      self.code().to_string(),
      contains_query,
      true,
      None,
    );
    let at_least = filter.at_least as usize;
    let at_most = filter.at_most as usize;
    // Validate if the count of matches falls within the expected range
    at_least <= matches.len() && matches.len() <= at_most
  }

  fn _check_filter_not_contains(
    &self, filter: &Filter, rule_store: &mut RuleStore, scope_node: &Node,
  ) -> bool {
    for ts_query in filter.not_contains() {
      // Instantiate the query and check if there's a match within the scope node
      // If there is the filter is not satisfied
      let query = &rule_store.query(ts_query);
      if get_match_for_query(scope_node, self.code(), query, true).is_some() {
        return false;
      }
    }
    true
  }
}
