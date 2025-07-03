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

use crate::models::concrete_syntax::parser::CsConstraint;
use crate::models::concrete_syntax::types::CapturedNode;
use regex::Regex;

/// Check if a captured node satisfies a single constraint
pub(crate) fn check_constraint(node: &CapturedNode, ctr: &CsConstraint) -> bool {
  match ctr {
    CsConstraint::In { items, .. } => items.contains(&node.text.to_string()),
    CsConstraint::Regex { pattern, .. } => {
      match Regex::new(pattern) {
        Ok(regex) => regex.is_match(&node.text),
        Err(_) => false, // Invalid regex patterns don't match
      }
    }
  }
}

/// Check if a captured node satisfies all constraints
pub(crate) fn satisfies_constraints(node: &CapturedNode, constraints: &[CsConstraint]) -> bool {
  constraints
    .iter()
    .all(|constraint| check_constraint(node, constraint))
}

#[cfg(test)]
#[path = "unit_tests/constraint_checker_test.rs"]
mod constraint_checker_test;
