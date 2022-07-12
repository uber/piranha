use std::collections::HashMap;

use tree_sitter::Range;

use super::rule::Rule;

pub(crate) struct Edit {
  replacement_range: Range,
  replacement_string: String,
  matched_rule: Rule,
  matches: HashMap<String, String>,
}

impl Edit {
  pub(crate) fn new(
    replacement_range: Range, replacement_string: String, matched_rule: Rule,
    matches: HashMap<String, String>,
  ) -> Self {
    Self {
      replacement_range,
      replacement_string,
      matched_rule,
      matches,
    }
  }

  #[cfg(test)]
  pub(crate) fn dummy_edit(replacement_range: Range, replacement_string: String) -> Self {
    Self::new(
      replacement_range,
      replacement_string,
      Rule::dummy(),
      HashMap::new(),
    )
  }

  /// Get the edit's replacement range.
  pub(crate) fn replacement_range(&self) -> Range {
    self.replacement_range
  }

  pub(crate) fn replacement_string(&self) -> &str {
    self.replacement_string.as_ref()
  }

  pub(crate) fn matched_rule(&self) -> Rule {
    self.matched_rule.clone()
  }

  pub(crate) fn matches(&self) -> &HashMap<String, String> {
    &self.matches
  }
}
