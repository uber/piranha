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

use std::path::PathBuf;

use itertools::Itertools;
use serde_derive::Serialize;

use super::{edit::Edit, matches::Match, source_code_unit::SourceCodeUnit};
use pyo3::prelude::pyclass;

#[derive(Serialize, Debug, Clone)]
#[pyclass]
pub struct MatchDto {
  #[pyo3(get)]
  name: String,
  #[pyo3(get)]
  match_: Match,
}

#[derive(Serialize, Debug, Clone, Default)]
#[pyclass]
pub struct PiranhaOutputSummary {
  #[pyo3(get)]
  path: String,
  #[pyo3(get)]
  content: String,
  #[pyo3(get)]
  matches: Vec<MatchDto>,
  #[pyo3(get)]
  rewrites: Vec<Edit>,
}

impl PiranhaOutputSummary {
  pub(crate) fn new(source_code_unit: &SourceCodeUnit) -> PiranhaOutputSummary {
    let mut match_dtos = Vec::<MatchDto>::new();

    for r in source_code_unit.matches().iter() {
      let match_dto = MatchDto {
        name: r.0.clone(),
        match_: r.1.clone(),
      };

      match_dtos.push(match_dto)
    }

    return PiranhaOutputSummary {
      path: String::from(source_code_unit.path().as_os_str().to_str().unwrap()),
      content: source_code_unit.code(),
      matches: match_dtos,
      rewrites: source_code_unit.rewrites().iter().cloned().collect_vec(),
    };
  }

  pub(crate) fn matches(&self) -> &Vec<MatchDto> {
    self.matches.as_ref()
  }

  pub(crate) fn rewrites(&self) -> &[Edit] {
    self.rewrites.as_ref()
  }

  pub fn path(&self) -> PathBuf {
    PathBuf::from(self.path.as_str())
  }

  pub fn content(&self) -> &str {
    self.content.as_ref()
  }
}
