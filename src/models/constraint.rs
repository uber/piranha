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

use getset::Getters;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Getters)]
pub(crate) struct Constraint {
  /// Scope in which the constraint query has to be applied
  #[get = "pub"]
  matcher: String,
  /// The Tree-sitter queries that need to be applied in the `matcher` scope
  #[get = "pub"]
  #[serde(default)]
  queries: Vec<String>,
}

impl Constraint {
  #[cfg(test)]
  pub(crate) fn new(matcher: String, queries: Vec<String>) -> Self {
    Self { matcher, queries }
  }
}
