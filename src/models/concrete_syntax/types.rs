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

use crate::models::matches::Range;
use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};

/// Represents a captured node during pattern matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedNode {
  pub range: Range,
  pub text: String,
}

/// Result of a pattern matching operation with clear success/failure semantics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternMatchResult {
  /// Successful match with captured variables and number of consumed nodes
  Success {
    captures: HashMap<String, CapturedNode>,
    consumed_nodes: usize,
    range: Option<Range>,
  },
  /// Failed to match with optional reason for debugging
  Failure { reason: Option<String> },
}

/// Context passed to matching functions to reduce parameter repetition
/// The lifetime 'tree represents the lifetime of the tree data, not the context itself
pub struct MatchingContext<'a> {
  pub cursor: TreeCursor<'a>,
  pub source_code: &'a [u8],
  pub top_node: &'a Node<'a>,
}

impl PatternMatchResult {
  /// Convenience constructor for successful matches
  pub fn success(captures: HashMap<String, CapturedNode>, consumed_nodes: usize) -> Self {
    PatternMatchResult::Success {
      captures,
      consumed_nodes,
      range: None,
    }
  }

  /// Convenience constructor for failures with reason
  pub fn failure(reason: impl Into<String>) -> Self {
    PatternMatchResult::Failure {
      reason: Some(reason.into()),
    }
  }

  /// Convenience constructor for failures without specific reason
  pub fn failed() -> Self {
    PatternMatchResult::Failure { reason: None }
  }

  /// Check if the result represents a successful match
  pub fn is_success(&self) -> bool {
    matches!(self, PatternMatchResult::Success { .. })
  }

  /// Extract captures from successful result, panics if called on failure
  pub fn captures(&self) -> &HashMap<String, CapturedNode> {
    match self {
      PatternMatchResult::Success { captures, .. } => captures,
      PatternMatchResult::Failure { .. } => panic!("Called captures() on failed match result"),
    }
  }

  /// Extract consumed node count from successful result, panics if called on failure
  pub fn consumed_nodes(&self) -> usize {
    match self {
      PatternMatchResult::Success { consumed_nodes, .. } => *consumed_nodes,
      PatternMatchResult::Failure { .. } => {
        panic!("Called consumed_nodes() on failed match result")
      }
    }
  }
}

/// Helper function to convert from legacy tuple format to new PatternMatchResult
/// This allows gradual migration of existing functions
pub fn convert_legacy_result(
  legacy_result: (HashMap<String, CapturedNode>, Option<usize>),
) -> PatternMatchResult {
  let (captures, consumed_nodes_opt) = legacy_result;

  match consumed_nodes_opt {
    Some(consumed_nodes) => PatternMatchResult::success(captures, consumed_nodes),
    None => PatternMatchResult::failed(),
  }
}
