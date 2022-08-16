use itertools::Itertools;
use serde_derive::Serialize;

use super::{edit::Edit, matches::Match, source_code_unit::SourceCodeUnit};

#[derive(Serialize, Debug, Clone, Default)]
pub(crate) struct PiranhaOutputSummary {
  path: String,
  content: String,
  matches: Vec<(String, Match)>,
  rewrites: Vec<Edit>,
}

impl PiranhaOutputSummary {
  pub(crate) fn new(source_code_unit: &SourceCodeUnit) -> PiranhaOutputSummary {
    return PiranhaOutputSummary {
      path: String::from(
        source_code_unit
          .path()
          .as_os_str()
          .to_str()
          .unwrap(),
      ),
      content: source_code_unit.code(),
      matches: source_code_unit.matches().iter().cloned().collect_vec(),
      rewrites: source_code_unit.rewrites().iter().cloned().collect_vec(),
    };
  }
  
  #[cfg(test)]
  pub(crate) fn matches(&self) -> &[(String, Match)] {
    self.matches.as_ref()
  }

  #[cfg(test)]
  pub(crate) fn rewrites(&self) -> &[Edit] {
    self.rewrites.as_ref()
  }
}
