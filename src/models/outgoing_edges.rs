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

use derive_builder::Builder;
use getset::Getters;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
// Represents the `edges.toml` file
pub(crate) struct Edges {
  pub(crate) edges: Vec<OutgoingEdges>,
}

// Captures an entry from the `edges.toml` file.
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Getters, Builder)]
pub struct OutgoingEdges {
  #[get = "pub with_prefix"]
  from: String,
  #[get = "pub with_prefix"]
  to: Vec<String>,
  #[get = "pub with_prefix"]
  scope: String,
}

#[macro_export]
macro_rules! edges {
  (from = $from: expr, to = [$($to: expr)*], scope = $scope: expr) => {
    $crate::models::outgoing_edges::OutgoingEdgesBuilder::default()
    .from($from.to_string())
    .to(vec![$($to.to_string())*])
    .scope($scope.to_string())
    .build().unwrap()
  };
}
