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
  tree_sitter_utilities::{get_match_for_query, get_node_for_range},
};

use super::{rule::InstantiatedRule, rule_store::RuleStore, source_code_unit::SourceCodeUnit};

use crate::utilities::{tree_sitter_utilities::TSQuery, Instantiate};

use super::default_configs::{default_enclosing_node, default_queries};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Getters, Builder)]
#[pyclass]
pub struct Filter {
  /// Scope in which the constraint query has to be applied
  #[builder(default = "default_enclosing_node()")]
  #[get = "pub"]
  #[pyo3(get)]
  enclosing_node: TSQuery,
  /// The Tree-sitter queries that need to be applied in the `enclosing_node` scope
  #[builder(default = "default_queries()")]
  #[get = "pub"]
  #[serde(default)]
  #[pyo3(get)]
  queries: Vec<TSQuery>,
}

#[pymethods]
impl Filter {
  #[new]
  fn py_new(enclosing_node: String, queries: Option<Vec<String>>) -> Self {
    FilterBuilder::default()
      .enclosing_node(TSQuery::new(enclosing_node))
      .queries(
        queries
          .unwrap_or_default()
          .iter()
          .map(|x| TSQuery::new(x.to_string()))
          .collect_vec(),
      )
      .build()
      .unwrap()
  }
  gen_py_str_methods!();
}

#[macro_export]
/// This macro can be used to construct a Constraint (via the builder)'
/// Allows to use builder pattern more "dynamically"
///
/// Usage:
///
/// ```
/// constraint! {
///   enclosing_node = "(method_declaration) @md".to_string(),
///   queries=  ["(method_invocation name: (_) @name) @mi".to_string()]
/// }
/// ```
///
/// expands to
///
/// ```
/// FilterBuilder::default()
///      .enclosing_node("(method_declaration) @md".to_string())
///      .queries(vec!["(method_invocation name: (_) @name) @mi".to_string()])
///      .build()
/// ```
///
macro_rules! constraint {
  (enclosing_node = $enclosing_node:expr, queries= [$($q:expr,)*]) => {
    $crate::models::filter::FilterBuilder::default()
      .enclosing_node($crate::utilities::tree_sitter_utilities::TSQuery::new($enclosing_node.to_string()))
      .queries(vec![$($crate::utilities::tree_sitter_utilities::TSQuery::new($q.to_string()),)*])
      .build().unwrap()
  };
}

pub use constraint;

impl Instantiate for Filter {
  /// Create a new query from `self` by updating the `query` and `replace` based on the substitutions.
  fn instantiate(&self, substitutions_for_holes: &HashMap<String, String>) -> Filter {
    Filter {
      enclosing_node: self.enclosing_node().instantiate(substitutions_for_holes),
      queries: self
        .queries()
        .iter()
        .map(|x| x.instantiate(substitutions_for_holes))
        .collect_vec(),
    }
  }
}

// Implements instance methods related to applying a constraint
impl SourceCodeUnit {
  pub(crate) fn is_satisfied(
    &self, node: Node, rule: &InstantiatedRule, substitutions: &HashMap<String, String>,
    rule_store: &mut RuleStore,
  ) -> bool {
    let mut updated_substitutions = self.piranha_arguments().input_substitutions();
    updated_substitutions.extend(substitutions.clone());
    rule.constraints().iter().all(|constraint| {
      self._is_satisfied(constraint.clone(), node, rule_store, &updated_substitutions)
    })
  }

  /// Checks if the node satisfies the constraints.
  /// Constraint has two parts (i) `constraint.enclosing_node` (ii) `constraint.query`.
  /// This function traverses the ancestors of the given `node` until `constraint.enclosing_node` matches
  /// i.e. finds scope for constraint.
  /// Within this scope it checks if the `constraint.query` DOES NOT MATCH any sub-tree.
  fn _is_satisfied(
    &self, constraint: Filter, node: Node, rule_store: &mut RuleStore,
    substitutions: &HashMap<String, String>,
  ) -> bool {
    let mut current_node = node;
    // This ensures that the below while loop considers the current node too when checking for constraints.
    // It does not make sense to check for constraint if current node is a "leaf" node.
    if node.child_count() > 0 {
      current_node = node.child(0).unwrap();
    }
    // Get the scope_node of the constraint (`scope.enclosing_node`)
    let mut matched_enclosing_node = false;
    while let Some(parent) = current_node.parent() {
      let instantiated_constraint = constraint.instantiate(substitutions);
      let enclosing_node_query_str = instantiated_constraint.enclosing_node();
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
        for query_with_holes in constraint.queries() {
          let query = &rule_store.query(&query_with_holes.instantiate(substitutions));

          if get_match_for_query(&scope_node, self.code(), query, true).is_some() {
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
