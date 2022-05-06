use itertools::Itertools;
use serde_derive::Deserialize;

// Represents the content in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeConfig {
  scopes: Vec<ScopeGenerator>,
}

impl ScopeConfig {
  /// Get a reference to the scope config's scopes.
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
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct ScopeQueryGenerator {
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
