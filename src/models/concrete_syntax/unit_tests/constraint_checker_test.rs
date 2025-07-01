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

use crate::models::concrete_syntax::constraint_checker::*;
use crate::models::concrete_syntax::parser::CsConstraint;
use crate::models::concrete_syntax::types::CapturedNode;
use crate::models::matches::{Point, Range};

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_node(text: &str) -> CapturedNode {
    CapturedNode {
      range: Range {
        start_byte: 0,
        end_byte: text.len(),
        start_point: Point { row: 0, column: 0 },
        end_point: Point {
          row: 0,
          column: text.len(),
        },
      },
      text: text.to_string(),
    }
  }

  #[test]
  fn test_regex_constraint_validation() {
    let constraint = CsConstraint::Regex {
      capture: "var".to_string(),
      pattern: "^test.*".to_string(),
    };

    // Should match
    let matching_node = create_test_node("testing123");
    assert!(check_constraint(&matching_node, &constraint));

    // Should not match
    let non_matching_node = create_test_node("nottest");
    assert!(!check_constraint(&non_matching_node, &constraint));
  }

  #[test]
  fn test_regex_constraint_invalid_pattern() {
    let constraint = CsConstraint::Regex {
      capture: "var".to_string(),
      pattern: "[invalid".to_string(), // Invalid regex
    };

    let node = create_test_node("anything");
    // Should return false for invalid regex patterns
    assert!(!check_constraint(&node, &constraint));
  }
}
