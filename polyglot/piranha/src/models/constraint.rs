use std::collections::HashMap;

use serde_derive::Deserialize;
use tree_sitter::Node;

use crate::utilities::tree_sitter_utilities::{
  get_node_for_range, substitute_tags, PiranhaHelpers,
};

use super::{rule_store::RuleStore, source_code_unit::SourceCodeUnit};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Constraint {
  /// Scope in which the constraint query has to be applied
  matcher: String,
  /// The Tree-sitter queries that need to be applied in the `matcher` scope
  queries: Vec<String>,
}

impl Constraint {
  pub(crate) fn queries(&self) -> &[String] {
    &self.queries
  }

  pub(crate) fn matcher(&self, substitutions: &HashMap<String, String>) -> String {
    substitute_tags(String::from(&self.matcher), substitutions)
  }

  /// Checks if the node satisfies the constraints.
  /// Constraint has two parts (i) `constraint.matcher` (ii) `constraint.query`.
  /// This function traverses the ancestors of the given `node` until `constraint.matcher` matches
  /// i.e. finds scope for constraint.
  /// Within this scope it checks if the `constraint.query` DOES NOT MATCH any sub-tree.
  pub(crate) fn is_satisfied(
    &self, node: Node, source_code_unit: SourceCodeUnit, rule_store: &mut RuleStore,
    substitutions: &HashMap<String, String>,
  ) -> bool {
    let mut current_node = node;
    // This ensures that the below while loop considers the current node too when checking for constraints.
    // It does not make sense to check for constraint if current node is a "leaf" node.
    if node.child_count() > 0 {
      current_node = node.child(0).unwrap();
    }
    // Get the scope_node of the constraint (`scope.matcher`)
    let mut matched_matcher = false;
    while let Some(parent) = current_node.parent() {
      let query_str = &self.matcher(substitutions);
      if let Some(p_match) =
        parent.get_match_for_query(&source_code_unit.code(), rule_store.query(query_str), false)
      {
        matched_matcher = true;
        let scope_node = get_node_for_range(
          source_code_unit.root_node(),
          p_match.range().start_byte,
          p_match.range().end_byte,
        );
        for query_with_holes in self.queries() {
          let query_str = substitute_tags(query_with_holes.to_string(), substitutions);
          let query = &rule_store.query(&query_str);
          // If this query matches anywhere within the scope, return false.
          if scope_node
            .get_match_for_query(&source_code_unit.code(), query, true)
            .is_some()
          {
            return false;
          }
        }
        break;
      }
      current_node = parent;
    }
    matched_matcher
  }
}

impl Constraint {
  #[cfg(test)]
  pub(crate) fn new(matcher: String, queries: Vec<String>) -> Self {
    Self { matcher, queries }
  }
}
