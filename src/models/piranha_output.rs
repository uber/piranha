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

use getset::Getters;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use crate::utilities::gen_py_str_methods;

use super::{edit::Edit, matches::Match, source_code_unit::SourceCodeUnit};
use pyo3::{prelude::pyclass, pymethods};

/// A class to represent Piranha's output
#[derive(Serialize, Debug, Clone, Default, Deserialize, Getters)]
#[pyclass]
pub struct PiranhaOutputSummary {
  /// Path to the file
  #[pyo3(get)]
  #[get = "pub(crate)"]
  path: String,
  /// Original content of the file after all the rewrites
  #[pyo3(get)]
  #[get = "pub(crate)"]
  #[serde(skip)]
  original_content: String,
  /// Final content of the file after all the rewrites
  #[pyo3(get)]
  #[get = "pub(crate)"]
  content: String,
  /// All the occurrences of "match-only" rules
  #[pyo3(get)]
  #[get = "pub(crate)"]
  matches: Vec<(String, Match)>,
  /// All the applied edits
  #[pyo3(get)]
  #[get = "pub(crate)"]
  rewrites: Vec<Edit>,
}

gen_py_str_methods!(PiranhaOutputSummary);

impl PiranhaOutputSummary {
  pub(crate) fn new(source_code_unit: &SourceCodeUnit) -> PiranhaOutputSummary {
    return PiranhaOutputSummary {
      path: String::from(source_code_unit.path().as_os_str().to_str().unwrap()),
      original_content: source_code_unit.original_content().to_string(),
      content: source_code_unit.code().to_string(),
      matches: source_code_unit.matches().iter().cloned().collect_vec(),
      rewrites: source_code_unit.rewrites().iter().cloned().collect_vec(),
    };
  }
}
