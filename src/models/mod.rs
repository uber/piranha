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

pub(crate) mod capture_group_patterns;
pub(crate) mod concrete_syntax;
pub(crate) mod default_configs;
pub mod edit;
pub mod filter;
pub mod language;
pub(crate) mod matches;
pub mod outgoing_edges;
pub mod piranha_arguments;
pub mod piranha_output;
pub mod rule;
pub mod rule_graph;
pub(crate) mod rule_store;
pub(crate) mod scopes;
pub(crate) mod source_code_unit;

pub(crate) trait Validator {
  fn validate(&self) -> Result<(), String>;
}
