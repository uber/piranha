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
  pub root_constraints: Vec<CsConstraint>,
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

/// Convenience function to resolve concrete syntax
pub fn resolve_concrete_syntax(cs: &ConcreteSyntax) -> ResolvedConcreteSyntax {
  cs.clone()
    .resolve()
    .expect("Failed to resolve concrete syntax")
}

impl ConcreteSyntax {
  /// Extract capture name from any constraint type
  fn extract_capture_name_from_constraint(constraint: &CsConstraint) -> String {
    match constraint {
      CsConstraint::In { capture, .. } => capture.clone(),
      CsConstraint::Regex { capture, .. } => capture.clone(),
      CsConstraint::Type { target, .. } => target.clone(),
      CsConstraint::Contains { target, .. } => target.clone(),
      CsConstraint::Not(inner_constraint) => {
        Self::extract_capture_name_from_constraint(inner_constraint)
      }
    }
  }

  /// Resolve Contains patterns in constraints
  fn resolve_contains_in_constraints(
    constraints: Vec<CsConstraint>,
  ) -> Result<Vec<CsConstraint>, String> {
    let mut resolved_constraints = Vec::new();

    for constraint in constraints {
      let resolved_constraint = match constraint {
        CsConstraint::Contains {
          target,
          pattern,
          resolved_pattern: _,
        } => {
          // Create concrete syntax from the pattern elements
          let cs_pattern = super::parser::CsPattern {
            sequence: pattern.clone(),
            constraints: vec![], // No additional constraints for sub-pattern
          };
          let concrete_syntax = ConcreteSyntax {
            pattern: cs_pattern,
          };

          // Recursively resolve the sub-pattern
          let resolved_pattern = concrete_syntax.resolve()?;

          CsConstraint::Contains {
            target,
            pattern,
            resolved_pattern: Some(Box::new(resolved_pattern)),
          }
        }
        CsConstraint::Not(inner) => {
          let resolved_inner = Self::resolve_contains_in_constraints(vec![*inner])?;
          CsConstraint::Not(Box::new(resolved_inner.into_iter().next().unwrap()))
        }
        other => other, // No resolution needed for other constraint types
      };
      resolved_constraints.push(resolved_constraint);
    }

    Ok(resolved_constraints)
  }

  /// Resolve constraints by attaching them to their corresponding captures
  pub fn resolve(self) -> Result<ResolvedConcreteSyntax, String> {
    // Validate constraints first
    for constraint in &self.pattern.constraints {
      match constraint {
        CsConstraint::Contains { target, .. } if target == "root" => {
          return Err(
            "'root contains' is not supported. Only 'root.node_type' constraints are allowed."
              .to_string(),
          );
        }
        _ => {}
      }
    }

    // Separate root constraints from capture constraints
    let mut root_constraints = Vec::new();
    let mut capture_constraints = Vec::new();

    for constraint in self.pattern.constraints {
      match &constraint {
        CsConstraint::Type { target, .. } if target == "root" => {
          root_constraints.push(constraint);
        }
        _ => {
          capture_constraints.push(constraint);
        }
      }
    }

    // Build a map of capture names to their constraints
    let mut constraint_map: HashMap<String, Vec<CsConstraint>> = HashMap::new();

    for constraint in capture_constraints {
      let capture_name = Self::extract_capture_name_from_constraint(&constraint);
      constraint_map
        .entry(capture_name)
        .or_default()
        .push(constraint);
    }

    // Transform elements, attaching resolved constraints to captures
    let mut resolved_sequence = Vec::new();

    for element in self.pattern.sequence {
      match element {
        CsElement::Capture { name, mode } => {
          let constraints = constraint_map.remove(&name).unwrap_or_default();
          // Resolve all constraints for this capture
          let resolved_constraints = Self::resolve_contains_in_constraints(constraints)?;

          resolved_sequence.push(ResolvedCsElement::Capture {
            name,
            mode,
            constraints: resolved_constraints,
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

    // Resolve root constraints
    let resolved_root_constraints = Self::resolve_contains_in_constraints(root_constraints)?;

    Ok(ResolvedConcreteSyntax {
      pattern: ResolvedCsPattern {
        sequence: resolved_sequence,
        root_constraints: resolved_root_constraints,
      },
    })
  }
}
