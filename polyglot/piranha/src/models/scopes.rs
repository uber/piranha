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

use itertools::Itertools;
use serde_derive::Deserialize;

use crate::utilities::tree_sitter_utilities::{
  get_node_for_range, substitute_tags, PiranhaHelpers,
};

use super::{rule_store::RuleStore, source_code_unit::SourceCodeUnit};

// Represents the content in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeConfig {
  scopes: Vec<ScopeGenerator>,
}

impl ScopeConfig {
  /// Get a reference to the scope `config's` scopes.
  #[must_use]
  pub(crate) fn scopes(&self) -> Vec<ScopeGenerator> {
    self.scopes.iter().cloned().collect_vec()
  }
}

// Represents an entry in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeGenerator {
  name: String,
  rules: Vec<ScopeQueryGenerator>,
}

impl ScopeGenerator {
  pub(crate) fn name(&self) -> &str {
    self.name.as_ref()
  }

  pub(crate) fn rules(&self) -> Vec<ScopeQueryGenerator> {
    self.rules.iter().cloned().collect_vec()
  }

  /// Generate a tree-sitter based query representing the scope of the previous edit.
  /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
  pub(crate) fn get_scope_query(
    source_code_unit: SourceCodeUnit, scope_level: &str, start_byte: usize, end_byte: usize,
    rules_store: &mut RuleStore,
  ) -> String {
    let root_node = source_code_unit.root_node();
    let mut changed_node = get_node_for_range(root_node, start_byte, end_byte);

    // Get the scope matchers for `scope_level` from the `scope_config.toml`.
    let scope_matchers = rules_store.get_scope_query_generators(scope_level);

    // Match the `scope_matcher.matcher` to the parent
    while let Some(parent) = changed_node.parent() {
      for m in &scope_matchers {
        if let Some((_, substitutions)) = parent.get_match_for_query(
          &source_code_unit.code(),
          rules_store.get_query(&m.matcher()),
          false,
        ) {
          // Generate the scope query for the specific context by substituting the
          // the tags with code snippets appropriately in the `generator` query.
          return substitute_tags(m.generator(), &substitutions);
        } else {
          changed_node = parent;
        }
      }
    }
    panic!("Could not create scope query for {:?}", scope_level);
  }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeQueryGenerator {
  matcher: String, // a tree-sitter query matching some enclosing AST pattern (like method or class)
  generator: String, // a tree-sitter query matching the exact AST node
}

impl ScopeQueryGenerator {
  pub(crate) fn matcher(&self) -> String {
    String::from(&self.matcher)
  }

  pub(crate) fn generator(&self) -> String {
    String::from(&self.generator)
  }
}

#[cfg(test)]
impl ScopeQueryGenerator {
  fn new(matcher: &str, generator: &str) -> ScopeQueryGenerator {
    ScopeQueryGenerator {
      matcher: matcher.to_string(),
      generator: generator.to_string(),
    }
  }
}
#[cfg(test)]
impl ScopeGenerator {
  fn new(name: &str, rules: Vec<ScopeQueryGenerator>) -> ScopeGenerator {
    ScopeGenerator {
      name: name.to_string(),
      rules,
    }
  }
}

#[cfg(test)]
#[path = "unit_tests/scopes_test.rs"]
mod scopes_test;
