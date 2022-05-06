use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
// Represents the `edges.toml` file
pub(crate) struct Edges {
  pub edges: Vec<OutgoingEdges>,
}

// Captures an entry from the `edges.toml` file.
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct OutgoingEdges {
  from: String,
  to: Vec<String>,
  scope: String,
}

impl OutgoingEdges {
  /// Get a reference to the edge's from.
  #[must_use]
  pub(crate) fn from_rule(&self) -> String {
    String::from(&self.from)
  }

  /// Get a reference to the edge's to.
  #[must_use]
  pub(crate) fn to_rules(&self) -> Vec<String> {
    self.to.clone()
  }

  /// Get a reference to the edge's scope.
  #[must_use]
  pub(crate) fn scope(&self) -> &str {
    self.scope.as_ref()
  }
}
