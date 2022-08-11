use std::collections::HashMap;

use tree_sitter::Range;

use super::{rule::Rule, matches::Match};

pub(crate) struct Edit {
  p_match: Match,
  replacement_string: String,
  matched_rule: Rule,
  
}

impl Edit {
  pub(crate) fn new(
    p_match: Match, replacement_string: String, matched_rule: Rule,
  ) -> Self {
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
      Rule::dummy(),
    )
  }

  /// Get the edit's replacement range.
  pub(crate) fn replacement_range(&self) -> Range {
    self.p_match.range()
  }

  pub(crate) fn replacement_string(&self) -> &str {
    self.replacement_string.as_ref()
  }

  pub(crate) fn matched_rule(&self) -> Rule {
    self.matched_rule.clone()
  }

  pub(crate) fn matches(&self) -> &HashMap<String, String> {
    &self.p_match.matches()
  }
}
