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

use regex::Regex;

use super::interpreter::get_all_matches_for_concrete_syntax;
use super::tree_sitter_adapter::{Node, SyntaxNode};
use crate::models::concrete_syntax::parser::CsConstraint;
use crate::models::matches::CapturedNode;
use crate::tree_sitter_adapter::SyntaxCursor;

/// Simple context for contains constraint checking
pub struct ConstraintContext<'a> {
  pub captured_node: &'a CapturedNode,
  pub source_code: &'a [u8],
  pub ast_root: &'a Node<'a>,
}

/// Check if a captured node satisfies a single constraint
pub(crate) fn check_constraint(
  node: &CapturedNode, ctr: &CsConstraint, context: Option<&ConstraintContext>,
) -> bool {
  match ctr {
    CsConstraint::In { items, .. } => items.contains(&node.text.to_string()),
    CsConstraint::Regex { pattern, .. } => {
      match Regex::new(pattern) {
        Ok(regex) => regex.is_match(&node.text),
        Err(_) => false, // Invalid regex patterns don't match
      }
    }
    CsConstraint::Type { types, .. } => {
      // Check if the captured node's type is in the allowed types list
      types.contains(&node.node_type)
    }
    CsConstraint::Contains {
      resolved_pattern, ..
    } => {
      // Use the resolved pattern for proper AST matching
      if let (Some(resolved), Some(ctx)) = (resolved_pattern, context) {
        check_contains_recursive(resolved, ctx)
      } else {
        false // No context or resolved pattern available
      }
    }
    CsConstraint::Not(inner_constraint) => {
      // Negation: return opposite of the inner constraint result
      !check_constraint(node, inner_constraint, context)
    }
  }
}

/// Check contains constraint using recursive pattern matching
fn check_contains_recursive(
  resolved_pattern: &super::resolver::ResolvedConcreteSyntax, context: &ConstraintContext,
) -> bool {
  // Run the resolved pattern against the entire AST
  let matches = get_all_matches_for_concrete_syntax(
    &context.ast_root.walk().node(),
    context.source_code,
    resolved_pattern,
    true, // recursive
    None, // replace_node
  );
  // If we found any matches, the contains constraint is satisfied
  !matches.is_empty()
}

/// Check if a captured node satisfies all constraints
pub(crate) fn satisfies_constraints(
  node: &CapturedNode, constraints: &[CsConstraint], context: Option<&ConstraintContext>,
) -> bool {
  constraints
    .iter()
    .all(|constraint| check_constraint(node, constraint, context))
}

/// Check if a node satisfies root type constraints before attempting pattern matching
pub(crate) fn satisfies_root_constraints(node: &Node, constraints: &[CsConstraint]) -> bool {
  for constraint in constraints {
    if let CsConstraint::Type { target, types } = constraint {
      if target == "root" {
        let node_type = node.kind();
        if !types.contains(&node_type.to_string()) {
          return false;
        }
      }
    }
  }
  true
}

#[cfg(test)]
#[path = "unit_tests/constraint_checker_test.rs"]
mod constraint_checker_test;
