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

use crate::utilities::tree_sitter_utilities::get_node_for_range;

use super::{
  capture_group_patterns::CGPattern, default_configs::default_child_count,
  default_configs::default_sibling_count, rule::InstantiatedRule, rule_store::RuleStore,
  source_code_unit::SourceCodeUnit, Validator,
};

use crate::utilities::Instantiate;

use super::default_configs::{
  default_contains_at_least, default_contains_at_most, default_contains_query,
  default_enclosing_node, default_not_contains_queries, default_not_enclosing_node,
};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Getters, Builder)]
#[pyclass]
#[builder(build_fn(name = "create"))]
pub struct Filter {
  /// AST patterns that some ancestor node of the primary match should match
  /// In case of multiple ancestors matching the AST pattern it will consider the innermost ancestor that matches.
  #[builder(default = "default_enclosing_node()")]
  #[get = "pub"]
  #[serde(default = "default_enclosing_node")]
  #[pyo3(get)]
  enclosing_node: CGPattern,

  /// AST patterns that some ancestor node of the primary match should match
  /// In case of multiple ancestors matching the AST pattern it will consider the outermost ancestor that matches.
  #[builder(default = "default_enclosing_node()")]
  #[get = "pub"]
  #[serde(default = "default_enclosing_node")]
  #[pyo3(get)]
  outermost_enclosing_node: CGPattern,

  /// AST patterns NO ancestor node of the primary match should match
  #[builder(default = "default_not_enclosing_node()")]
  #[get = "pub"]
  #[serde(default = "default_not_enclosing_node")]
  #[pyo3(get)]
  not_enclosing_node: CGPattern,
  /// AST patterns that should not match any subtree of node matching `enclosing_node` pattern
  #[builder(default = "default_not_contains_queries()")]
  #[get = "pub"]
  #[serde(default = "default_not_contains_queries")]
  #[pyo3(get)]
  not_contains: Vec<CGPattern>,
  /// AST patterns that should match any subtree of node matching `enclosing_node` pattern
  #[builder(default = "default_contains_query()")]
  #[get = "pub"]
  #[serde(default = "default_contains_query")]
  #[pyo3(get)]
  contains: CGPattern,
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

  // number of named children under the primary matched node
  #[builder(default = "default_child_count()")]
  #[get = "pub"]
  #[serde(default = "default_child_count")]
  #[pyo3(get)]
  child_count: u32,

  // number of named siblings of the primary matched node (inclusive)
  #[builder(default = "default_sibling_count()")]
  #[get = "pub"]
  #[serde(default = "default_sibling_count")]
  #[pyo3(get)]
  sibling_count: u32,
}

#[pymethods]
impl Filter {
  #[new]
  fn py_new(
    enclosing_node: Option<String>, outermost_enclosing_node: Option<String>,
    not_enclosing_node: Option<String>, not_contains: Option<Vec<String>>,
    contains: Option<String>, at_least: Option<u32>, at_most: Option<u32>,
    child_count: Option<u32>, sibling_count: Option<u32>,
  ) -> Self {
    FilterBuilder::default()
      .enclosing_node(CGPattern::new(enclosing_node.unwrap_or_default()))
      .outermost_enclosing_node(CGPattern::new(outermost_enclosing_node.unwrap_or_default()))
      .not_enclosing_node(CGPattern::new(not_enclosing_node.unwrap_or_default()))
      .not_contains(
        not_contains
          .unwrap_or_default()
          .iter()
          .map(|x| CGPattern::new(x.to_string()))
          .collect_vec(),
      )
      .contains(CGPattern::new(contains.unwrap_or_default()))
      .at_least(at_least.unwrap_or(default_contains_at_least()))
      .at_most(at_most.unwrap_or(default_contains_at_most()))
      .child_count(child_count.unwrap_or(default_child_count()))
      .sibling_count(sibling_count.unwrap_or(default_sibling_count()))
      .build()
  }

  fn __repr__(&self) -> String {
    format!("{self:?}")
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}

impl Validator for Filter {
  fn validate(&self) -> Result<(), String> {
    // Only allow users to set either contains or not_contains, but not both
    if *self.contains() != default_contains_query()
      && *self.not_contains() != default_not_contains_queries()
    {
      return Err(
        "Invalid Filter Argument. `contains` and `not_contains` cannot be set at the same time !!! Please use two filters instead."
          .to_string(),
      );
    }

    if self.at_least > self.at_most {
      return Err(
        "Invalid Filter Argument. `at_least` should be less than or equal to `at_most` !!!"
          .to_string(),
      );
    }

    // If the user set `at_least` or `at_most`, then the contains query cannot be empty
    if (self.at_least != default_contains_at_least() || self.at_most != default_contains_at_most())
      && self.contains().pattern().is_empty()
    {
      return Err(
        "Invalid Filter Argument. `at_least` or `at_most` is set, but `contains` is empty !!!"
          .to_string(),
      );
    }

    if *self.enclosing_node() != default_enclosing_node() {
      self.enclosing_node().validate()?
    }

    if *self.outermost_enclosing_node() != default_enclosing_node() {
      self.outermost_enclosing_node().validate()?
    }

    if *self.not_enclosing_node() != default_not_enclosing_node() {
      self.not_enclosing_node().validate()?
    }

    if *self.contains() != default_contains_query() {
      self.contains().validate()?
    }

    if *self.not_contains() != default_not_contains_queries() {
      self.not_contains().iter().try_for_each(|x| x.validate())?
    }

    if (*self.child_count() != default_child_count()
      || *self.sibling_count() != default_sibling_count())
      && (*self.enclosing_node() != default_enclosing_node()
        || *self.not_enclosing_node() != default_not_enclosing_node()
        || *self.outermost_enclosing_node() != default_enclosing_node()
        || *self.contains() != default_contains_query()
        || *self.not_contains() != default_not_contains_queries())
    {
      return Err("The child/sibling count operator is not compatible with (not) enclosing node and (not) contains operator".to_string());
    }

    Ok(())
  }
}

impl FilterBuilder {
  /// Builds Filter from FilterBuilder
  /// * create Filter from the builder
  /// * validates new argument combinations
  pub fn build(&self) -> Filter {
    match &self._validate() {
      Ok(filter) => filter.clone(),
      Err(e) => panic!("Invalid filter - {e}"),
    }
  }

  fn _validate(&self) -> Result<Filter, String> {
    let _filter: Filter = self.create().unwrap();
    _filter.validate().map(|_| _filter)
  }
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
  ($(enclosing_node = $enclosing_node:expr)? $(, outermost_enclosing_node=$outermost_enclosing_node:expr)? $(, not_enclosing_node=$not_enclosing_node:expr)? $(, not_contains= [$($q:expr,)*])? $(, contains= $p:expr)? $(, at_least=$min:expr)? $(, at_most=$max:expr)? $(, child_count=$nChildren:expr)? $(, sibling_count=$nSibling:expr)?) => {
    $crate::models::filter::FilterBuilder::default()
      $(.enclosing_node($crate::models::capture_group_patterns::CGPattern::new($enclosing_node.to_string())))?
      $(.outermost_enclosing_node($crate::models::capture_group_patterns::CGPattern::new($outermost_enclosing_node.to_string())))?
      $(.not_enclosing_node($crate::models::capture_group_patterns::CGPattern::new($not_enclosing_node.to_string())))?
      $(.not_contains(vec![$($crate::models::capture_group_patterns::CGPattern::new($q.to_string()),)*]))?
      $(.contains($crate::models::capture_group_patterns::CGPattern::new($p.to_string())))?
      $(.at_least($min))?
      $(.at_most($max))?
      $(.child_count($nChildren))?
      $(.sibling_count($nSibling))?
      .build()
  };
}

pub use filter;

impl Instantiate for Filter {
  /// Create a new filter from `self` by updating the all queries (i.e., `enclosing_node`, `not_enclosing_node`, `contains`, `not_contains`) based on the substitutions.
  fn instantiate(&self, substitutions_for_holes: &HashMap<String, String>) -> Filter {
    Filter {
      enclosing_node: self.enclosing_node().instantiate(substitutions_for_holes),
      outermost_enclosing_node: self
        .outermost_enclosing_node()
        .instantiate(substitutions_for_holes),
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
      child_count: self.child_count,
      sibling_count: self.sibling_count,
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
  /// (ii) `not_enclosing_node`, optional query that no ancestor of the primary match should match,
  /// (iii) `not_contains` and `contains`, optional queries that should not and should match within the `enclosing_node`,
  /// (iv) `at_least` and `at_most`, optional parameters indicating the acceptable range of matches for `contains` within the `enclosing_node`.
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

    if *filter.child_count() != default_child_count() {
      return node.named_child_count() == (*filter.child_count() as usize);
    }

    if *filter.sibling_count() != default_sibling_count() {
      return node.parent().unwrap().named_child_count() == (*filter.sibling_count() as usize);
    }

    // Check if no ancestor matches the query for not_enclosing_node
    if !self._check_not_enclosing_node(rule_store, node_to_check, &instantiated_filter) {
      return false;
    }
    // If an enclosing node is provided
    let query = instantiated_filter.enclosing_node();
    if !query.pattern().is_empty() {
      if let Some(result) = self._match_ancestor(rule_store, node_to_check, query) {
        node_to_check = result;
      } else {
        return false;
      }
    }

    // If an outermost enclosing node is provided
    let query = instantiated_filter.outermost_enclosing_node();
    if !query.pattern().is_empty() {
      if let Some(result) = self._match_outermost_ancestor(rule_store, node_to_check, query) {
        node_to_check = result;
      } else {
        return false;
      }
    }

    self._check_filter_not_contains(&instantiated_filter, rule_store, &node_to_check)
      && self._check_filter_contains(&instantiated_filter, rule_store, &node_to_check)
  }

  /// Check if the `node` does not have any ancestor that matches the `not_enclosing_node` query
  fn _check_not_enclosing_node(
    &self, rule_store: &mut RuleStore, node_to_check: Node, instantiated_filter: &Filter,
  ) -> bool {
    let query = instantiated_filter.not_enclosing_node();
    if !query.pattern().is_empty() {
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

  /// Search for outermost ancestor of `node` (including itself) that matches `query_str`
  fn _match_outermost_ancestor(
    &self, rule_store: &mut RuleStore, node: Node, ts_query: &CGPattern,
  ) -> Option<Node> {
    let mut matched_ancestor = self._match_ancestor(rule_store, node, ts_query);
    loop {
      if let Some(outer_matched_ancestor) = matched_ancestor
        .and_then(|m| m.parent().filter(|p| p.range() != m.range()))
        .and_then(|parent| self._match_ancestor(rule_store, parent, ts_query))
      {
        matched_ancestor = Some(outer_matched_ancestor);
        continue;
      }
      return matched_ancestor;
    }
  }

  /// Search for innermost ancestor of `node` (including itself) that matches `query_str`
  fn _match_ancestor(
    &self, rule_store: &mut RuleStore, node: Node, ts_query: &CGPattern,
  ) -> Option<Node> {
    let mut current_node = node;
    // This ensures that the below while loop considers the current node too when checking for filters.
    if current_node.child_count() > 0 {
      current_node = current_node.child(0).unwrap();
    }

    while let Some(parent) = current_node.parent() {
      let pattern = rule_store.query(ts_query);
      if let Some(p_match) = pattern.get_match(&parent, self.code(), false) {
        let matched_ancestor = get_node_for_range(
          self.root_node(),
          *p_match.range().start_byte(),
          *p_match.range().end_byte(),
        );
        return Some(matched_ancestor);
      }
      current_node = parent;
    }
    None
  }

  /// Check if the contains filter is satisfied by ancestor or any of its descendants
  fn _check_filter_contains(
    &self, filter: &Filter, rule_store: &mut RuleStore, ancestor: &Node,
  ) -> bool {
    // If the query is empty
    let ts_query = filter.contains();
    if ts_query.pattern().is_empty() {
      return true;
    }

    // Retrieve all matches within the ancestor node
    let contains_query = &rule_store.query(filter.contains());
    let matches = contains_query.get_matches(ancestor, self.code().to_string(), true, None, None);
    let at_least = filter.at_least as usize;
    let at_most = filter.at_most as usize;
    // Validate if the count of matches falls within the expected range
    at_least <= matches.len() && matches.len() <= at_most
  }

  /// Check if the not_contains filter is satisfied by ancestor or any of its descendants
  fn _check_filter_not_contains(
    &self, filter: &Filter, rule_store: &mut RuleStore, ancestor: &Node,
  ) -> bool {
    for ts_query in filter.not_contains() {
      // Check if there's a match within the scope node
      // If one of the filters is not satisfied, return false
      let query = &rule_store.query(ts_query);
      if query.get_match(ancestor, self.code(), true).is_some() {
        return false;
      }
    }
    true
  }
}
