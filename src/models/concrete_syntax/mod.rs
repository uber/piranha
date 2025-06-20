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

//! Concrete Syntax Module
//!
//! This module provides concrete syntax pattern matching capabilities for Piranha.
//! It includes a parser for concrete syntax patterns and an interpreter that can
//! match these patterns against tree-sitter ASTs.

pub mod interpreter;
pub mod parser;

// Re-export main functions for backward compatibility
pub use interpreter::parse_and_match;

use crate::models::capture_group_patterns::ConcreteSyntax as ConcreteSyntaxWrapper;
use crate::models::matches::{Match, Range};
use tree_sitter::Node;

// Legacy types kept for compatibility with existing test infrastructure
#[derive(Clone, PartialEq, Eq)]
pub struct CapturedNode {
  pub range: Range,
  pub text: String,
}

/// Main entry point for concrete syntax matching - delegates to v3 implementation
pub(crate) fn get_all_matches_for_concrete_syntax(
  node: &Node, code_str: &[u8], cs: &ConcreteSyntaxWrapper, recursive: bool,
  replace_node: Option<String>,
) -> (Vec<Match>, bool) {
  // Use v3 implementation for enhanced parsing with alternations and constraints
  match parse_and_match(&cs.0, node, code_str, recursive, replace_node) {
    Ok(result) => result,
    Err(_) => (Vec::new(), false), // Return empty on parse error
  }
}

#[cfg(test)]
#[path = "concrete_syntax_test.rs"]
mod concrete_syntax_test;

#[cfg(test)]
#[path = "complex_features_test.rs"]
mod complex_features_test;
