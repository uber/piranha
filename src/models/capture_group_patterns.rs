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

use crate::{
  models::Validator,
  utilities::{
    tree_sitter_utilities::{get_ts_query_parser, number_of_errors},
    Instantiate,
  },
};
use pyo3::prelude::pyclass;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[pyclass]
#[derive(Deserialize, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct CGPattern(pub String);

impl CGPattern {
  pub(crate) fn new(query: String) -> Self {
    Self(query)
  }

  pub(crate) fn pattern(&self) -> String {
    self.0.to_string()
  }
}

impl Validator for CGPattern {
  fn validate(&self) -> Result<(), String> {
    let mut parser = get_ts_query_parser();
    parser
      .parse(self.pattern(), None)
      .filter(|x| number_of_errors(&x.root_node()) == 0)
      .map(|_| Ok(()))
      .unwrap_or(Err(format!("Cannot parse - {}", self.pattern())))
  }
}

impl Instantiate for CGPattern {
  fn instantiate(&self, substitutions: &HashMap<String, String>) -> Self {
    let substitutions = substitutions
      .iter()
      .map(|(k, v)| (k.to_string(), v.replace('\n', "\\n")))
      .collect();
    CGPattern::new(self.pattern().instantiate(&substitutions))
  }
}
