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

use super::capture_group_patterns::CGPattern;
use super::{rule_store::RuleStore, source_code_unit::SourceCodeUnit};
use crate::utilities::tree_sitter_utilities::get_node_for_range;
use crate::utilities::Instantiate;
use derive_builder::Builder;
use getset::Getters;
use log::trace;
use serde_derive::Deserialize;

// Represents the content in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Getters)]
pub(crate) struct ScopeConfig {
  #[get = "pub"]
  scopes: Vec<ScopeGenerator>,
}

// Represents an entry in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Builder, Getters)]
pub(crate) struct ScopeGenerator {
  #[get = "pub"]
  name: String,
  #[get = "pub"]
  rules: Vec<ScopeQueryGenerator>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Getters, Builder)]
pub(crate) struct ScopeQueryGenerator {
  #[get = "pub"]
  enclosing_node: CGPattern, // a tree-sitter query matching some enclosing AST pattern (like method or class)
  #[get = "pub"]
  scope: CGPattern, // a tree-sitter query that will match the same node that matched `enclosing_node`
}

// Implements instance methods related to getting the scope
impl SourceCodeUnit {
  /// Generate a tree-sitter based query representing the scope of the previous edit.
  /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
  pub(crate) fn get_scope_query(
    &self, scope_level: &str, start_byte: usize, end_byte: usize, rules_store: &mut RuleStore,
  ) -> CGPattern {
    let root_node = self.root_node();
    let mut changed_node = get_node_for_range(root_node, start_byte, end_byte);
    // Get the scope enclosing_nodes for `scope_level` from the `scope_config.toml`.
    let scope_enclosing_nodes = rules_store.get_scope_query_generators(scope_level);

    // Match the `scope_enclosing_node.enclosing_node` to the parent
    loop {
      trace!(
        "Getting scope {} for node kind {}",
        scope_level,
        changed_node.kind()
      );
      for m in &scope_enclosing_nodes {
        let pattern = rules_store.query(m.enclosing_node());
        if let Some(p_match) = pattern.get_match(&changed_node, self.code(), false) {
          // Generate the scope query for the specific context by substituting the
          // the tags with code snippets appropriately in the `generator` query.
          return m.scope().instantiate(p_match.matches());
        }
      }
      if let Some(parent) = changed_node.parent() {
        changed_node = parent;
      } else {
        break;
      }
    }
    panic!("Could not create scope query for {scope_level:?}");
  }
}

#[cfg(test)]
#[path = "unit_tests/scopes_test.rs"]
mod scopes_test;
