use std::collections::HashMap;

use serde_derive::Deserialize;

/// Captures the Piranha arguments by from the file at `path_to_feature_flag_rules`.
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct PiranhaConfig {
  language: Vec<String>,
  substitutions: Vec<Vec<String>>,
}

impl PiranhaConfig {
  pub(crate) fn substitutions(&self) -> HashMap<String, String> {
    self
      .substitutions
      .iter()
      .map(|x| (String::from(&x[0]), String::from(&x[1])))
      .collect()
  }

  pub(crate) fn language(&self) -> String {
    self.language[0].clone()
  }
}
