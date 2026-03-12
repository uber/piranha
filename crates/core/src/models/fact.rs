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

use std::collections::HashMap;

use getset::Getters;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::{Deserialize, Serialize};

use super::matches::Range;

/// A fact recorded by a fact rule: a `String -> String` map associated to a range in source code.
/// When a rewrite overlaps the fact's range, the fact is marked `voided`.
/// When a rewrite precedes the fact's range, the fact's byte offsets are shifted.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Getters)]
#[pyclass]
pub struct Fact {
  /// The range in source code this fact is associated with
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) range: Range,
  /// The fact data
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) data: HashMap<String, String>,
  /// Whether this fact was voided by a subsequent rewrite that overlapped its range
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) voided: bool,
}

impl Fact {
  pub(crate) fn new(range: Range, data: HashMap<String, String>) -> Self {
    Fact { range, data, voided: false }
  }

  /// Shift byte offsets by `delta` after a preceding edit.
  /// Point (row/col) offsets are not adjusted since we don't have source code available here.
  pub(crate) fn shift_range(&mut self, delta: isize) {
    self.range.start_byte = (self.range.start_byte as isize + delta) as usize;
    self.range.end_byte = (self.range.end_byte as isize + delta) as usize;
  }
}

#[pymethods]
impl Fact {
  fn __repr__(&self) -> String {
    format!("{self:?}")
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}
