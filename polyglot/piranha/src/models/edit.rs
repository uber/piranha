use std::collections::HashMap;

use serde_derive::Serialize;
use tree_sitter::Range;

use super::matches::Match;
use pyo3::prelude::pyclass;

#[derive(Serialize, Debug, Clone)]
#[pyclass]
pub(crate) struct Edit {
  // The match representing the target site of the edit
  #[pyo3(get)]
  p_match: Match,
  // The string to replace the substring encompassed by the match
  #[pyo3(get)]
  replacement_string: String,
  // The rule used for creating this match-replace
  #[pyo3(get)]
  matched_rule: String,
}

impl Edit {
  pub(crate) fn new(p_match: Match, replacement_string: String, matched_rule: String) -> Self {
    Self {
      p_match,
      replacement_string,
      matched_rule,
    }
  }

  #[cfg(test)]
  pub(crate) fn dummy_edit(replacement_range: Range, replacement_string: String) -> Self {
    Self::new(
      Match::new(replacement_range, HashMap::new()),
      replacement_string,
      String::new(),
    )
  }

  /// Get the edit's replacement range.
  pub(crate) fn replacement_range(&self) -> Range {
    self.p_match.range()
  }

  pub(crate) fn replacement_string(&self) -> &str {
    self.replacement_string.as_ref()
  }

  pub(crate) fn matched_rule(&self) -> String {
    self.matched_rule.clone()
  }

  pub(crate) fn matches(&self) -> &HashMap<String, String> {
    self.p_match.matches()
  }
}
