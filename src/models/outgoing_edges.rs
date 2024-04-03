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

use derive_builder::Builder;
use getset::Getters;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
// Represents the `edges.toml` file
pub struct Edges {
  pub edges: Vec<OutgoingEdges>,
}

// Captures an entry from the `edges.toml` file.
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Getters, Builder)]
#[pyclass]
pub struct OutgoingEdges {
  /// The source rule or group of rules
  #[get = "pub with_prefix"]
  #[serde(alias = "from")]
  #[pyo3(get)]
  frm: String,
  /// The target edges or groups of edges
  #[get = "pub with_prefix"]
  #[pyo3(get)]
  to: Vec<String>,
  /// The scope label for the edge
  #[get = "pub with_prefix"]
  #[pyo3(get)]
  scope: String,
}

#[macro_export]
macro_rules! edges {
  (from = $from: expr, to = [$($to: expr)*], scope = $scope: expr) => {
    $crate::models::outgoing_edges::OutgoingEdgesBuilder::default()
    .frm($from.to_string())
    .to(vec![$($to.to_string())*])
    .scope($scope.to_string())
    .build().unwrap()
  };
}

#[pymethods]
impl OutgoingEdges {
  #[new]
  fn py_new(from: String, to: Vec<String>, scope: String) -> Self {
    OutgoingEdgesBuilder::default()
      .frm(from)
      .to(to)
      .scope(scope)
      .build()
      .unwrap()
  }

  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}
