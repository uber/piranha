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

use std::collections::HashMap;

use derive_builder::Builder;
use getset::Getters;
use itertools::Itertools;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::Deserialize;

use crate::utilities::{tree_sitter_utilities::TSQuery, Instantiate};

use super::default_configs::{default_matcher, default_queries};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Getters, Builder)]
#[pyclass]
pub struct Constraint {
  /// Scope in which the constraint query has to be applied
  #[builder(default = "default_matcher()")]
  #[get = "pub"]
  #[pyo3(get)]
  matcher: TSQuery,
  /// The Tree-sitter queries that need to be applied in the `matcher` scope
  #[builder(default = "default_queries()")]
  #[get = "pub"]
  #[serde(default)]
  #[pyo3(get)]
  queries: Vec<TSQuery>,
}

#[pymethods]
impl Constraint {
  #[new]
  fn py_new(matcher: String, queries: Option<Vec<String>>) -> Self {
    ConstraintBuilder::default()
      .matcher(TSQuery::new(matcher))
      .queries(
        queries
          .unwrap_or_default()
          .iter()
          .map(|x| TSQuery::new(x.to_string()))
          .collect_vec(),
      )
      .build()
      .unwrap()
  }
}

#[macro_export]
/// This macro can be used to construct a Constraint (via the builder)'
/// Allows to use builder pattern more "dynamically"
///
/// Usage:
///
/// ```ignore
/// constraint! {
///   matcher = "(method_declaration) @md".to_string(),
///   queries=  ["(method_invocation name: (_) @name) @mi".to_string()]
/// }
/// ```
///
/// expands to
///
/// ```ignore
/// ConstraintBuilder::default()
///      .matcher("(method_declaration) @md".to_string())
///      .queries(vec!["(method_invocation name: (_) @name) @mi".to_string()])
///      .build()
/// ```
///
macro_rules! constraint {
  (matcher = $matcher:expr, queries= [$($q:expr,)*]) => {
    $crate::models::constraint::ConstraintBuilder::default()
      .matcher($crate::utilities::tree_sitter_utilities::TSQuery::new($matcher.to_string()))
      .queries(vec![$($crate::utilities::tree_sitter_utilities::TSQuery::new($q.to_string()),)*])
      .build().unwrap()
  };
}

pub use constraint;

impl Instantiate for Constraint {
  /// Create a new query from `self` by updating the `query` and `replace` based on the substitutions.
  fn instantiate(&self, substitutions_for_holes: &HashMap<String, String>) -> Constraint {
    Constraint {
      matcher: self.matcher().instantiate(substitutions_for_holes),
      queries: self
        .queries()
        .iter()
        .map(|x| x.instantiate(substitutions_for_holes))
        .collect_vec(),
    }
  }
}
