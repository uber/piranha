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

use crate::utilities::tree_sitter_utilities::TSQuery;
// Represents the content in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Getters)]
pub(crate) struct ScopeConfig {
  #[get = "pub"]
  scopes: Vec<ScopeGenerator>,
}

// Represents an entry in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Builder, Getters)]
pub(crate) struct ScopeGenerator {
  #[get = "pub"]
  name: String,
  #[get = "pub"]
  rules: Vec<ScopeQueryGenerator>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default, Getters, Builder)]
pub(crate) struct ScopeQueryGenerator {
  #[get = "pub"]
  matcher: TSQuery, // a tree-sitter query matching some enclosing AST pattern (like method or class)
  #[get = "pub"]
  generator: TSQuery, // a tree-sitter query matching the exact AST node
}

#[cfg(test)]
#[path = "unit_tests/scopes_test.rs"]
mod scopes_test;
