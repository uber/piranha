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

use super::parser::{CaptureMode, ConcreteSyntax, CsConstraint, CsElement};
use std::collections::HashMap;

/// Resolved structure where constraints are attached to captures
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedConcreteSyntax {
  pub pattern: ResolvedCsPattern,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCsPattern {
  pub sequence: Vec<ResolvedCsElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedCsElement {
  Capture {
    name: String,
    mode: CaptureMode,
    constraints: Vec<CsConstraint>,
  },
  Literal(String),
}

impl ConcreteSyntax {
  /// Resolve constraints by attaching them to their corresponding captures
  pub fn resolve(self) -> Result<ResolvedConcreteSyntax, String> {
    // Build a map of capture names to their constraints
    let mut constraint_map: HashMap<String, Vec<CsConstraint>> = HashMap::new();

    for constraint in self.pattern.constraints {
      match &constraint {
        CsConstraint::In { capture, .. } => {
          constraint_map
            .entry(capture.clone())
            .or_default()
            .push(constraint);
        }
      }
    }

    // Transform elements, attaching constraints to captures
    let mut resolved_sequence = Vec::new();

    for element in self.pattern.sequence {
      match element {
        CsElement::Capture { name, mode } => {
          let constraints = constraint_map.remove(&name).unwrap_or_default();
          resolved_sequence.push(ResolvedCsElement::Capture {
            name,
            mode,
            constraints,
          });
        }
        CsElement::Literal(text) => {
          resolved_sequence.push(ResolvedCsElement::Literal(text));
        }
      }
    }

    // Check for unresolved constraints (constraints that reference non-existent captures)
    if !constraint_map.is_empty() {
      let unresolved: Vec<String> = constraint_map.keys().cloned().collect();
      return Err(format!(
        "Unresolved constraints for captures: {unresolved:?}"
      ));
    }

    Ok(ResolvedConcreteSyntax {
      pattern: ResolvedCsPattern {
        sequence: resolved_sequence,
      },
    })
  }
}

#[cfg(test)]
#[path = "unit_tests/resolver_test.rs"]
mod resolver_test;
