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

use crate::models::concrete_syntax::constraint_checker::satisfies_constraints;
use crate::models::concrete_syntax::cursor_utils::CursorNavigator;
use crate::models::concrete_syntax::parser::CaptureMode;
use crate::models::concrete_syntax::parser::CsConstraint;
use crate::models::concrete_syntax::resolver::{ResolvedConcreteSyntax, ResolvedCsElement};
use crate::models::concrete_syntax::types::{CapturedNode, MatchingContext, PatternMatchResult};
use crate::models::matches::Match;
use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};
use tree_sitter_traversal::Cursor;

// =============================================================================
// PUBLIC API
// =============================================================================

pub(crate) fn get_all_matches_for_concrete_syntax(
  node: &Node, source_code_ref: &[u8], cs: &ResolvedConcreteSyntax, recursive: bool,
  replace_node: Option<String>,
) -> Vec<Match> {
  let mut matches: Vec<Match> = Vec::new();

  if let PatternMatchResult::Success {
    captures: mut mapping,
    consumed_nodes: _last_node_index,
    range: Some(range),
  } = match_sequential_siblings(&mut node.walk(), source_code_ref, &cs.pattern.sequence)
  {
    // If the user doesn't specify a replace node, we use '*' as the default (everything)
    let replace_node_key = replace_node.clone().unwrap_or("*".to_string());

    if replace_node_key == "*" {
      let replace_node_match = CapturedNode {
        range,
        text: CursorNavigator::get_text_from_range(
          range.start_byte,
          range.end_byte,
          source_code_ref,
        ),
      };
      mapping.insert("*".to_string(), replace_node_match);
    }

    let match_item = create_match_from_capture(&replace_node_key, mapping, range);
    matches.push(match_item);
  }

  if recursive {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
      let mut inner_matches = get_all_matches_for_concrete_syntax(
        &child,
        source_code_ref,
        cs,
        recursive,
        replace_node.clone(),
      );
      matches.append(&mut inner_matches)
    }
  }
  matches
}

// =============================================================================
// CORE MATCHING ENGINE
// =============================================================================

/// Attempts to match a pattern against sequential sibling nodes in an AST.
///
/// ## How it works:
/// Given a function body with multiple statements, try to match a pattern starting
/// at each statement position until we find a match.
///
/// ```
/// Pattern: int :[x] = 2; :[x]++;
///
/// Function Body AST:
/// ├── [0] char a = 'c' ; ← Start matching here: ❌ No match
/// ├── [1] a++;           ← Start matching here: ❌ No match
/// ├── [2] int b = 2;     ← Start matching here: ✅ MATCH! (spans [2] and [3]) -> Returns
/// ├── [3] b++;           ←
/// ├── [4] return a;      ←
/// └── [5] }              ←
/// ```
///
/// The algorithm slides a "matching window" across all possible starting positions,
/// trying to match consecutive siblings against the pattern elements.
fn match_sequential_siblings(
  cursor: &mut TreeCursor, source_code_ref: &[u8], cs_elements: &[ResolvedCsElement],
) -> PatternMatchResult {
  let parent_node = cursor.node();
  let mut child_seq_match_start = 0;
  if cursor.goto_first_child() {
    // Iterate through siblings to find a match
    loop {
      // Clone the cursor in order to attempt matching the sequence starting at cursor.node
      // Cloning here is necessary other we won't be able to advance to the next sibling if the matching fails
      let result = {
        match_cs_pattern(
          &mut MatchingContext {
            cursor: cursor.clone(),
            source_code: source_code_ref,
            top_node: &parent_node,
          },
          cs_elements,
          true,
        )
      };

      // If we got a successful match, extract the mapping and index
      if let PatternMatchResult::Success {
        captures: mapping,
        consumed_nodes: last_node_index,
        range: None,
      } = result
      {
        // Determine the last matched node. Remember, we are matching subsequences of children [n ... k]
        let last_node = parent_node.child(last_node_index);
        let range = Range::span_ranges(cursor.node().range(), last_node.unwrap().range());
        if last_node_index != child_seq_match_start || parent_node.child_count() == 1 {
          return PatternMatchResult::Success {
            captures: mapping,
            consumed_nodes: last_node_index,
            range: Some(range),
          };
        }
        // This is to prevent double matches when unrolling a node. i.e., matching the statement in a function body,
        // as well as the statement itself when unrolled.
        return PatternMatchResult::failed();
      }

      child_seq_match_start += 1;
      if !cursor.goto_next_sibling() {
        break;
      }
    }
  } // Not currently handing matching of leaf nodes. Current semantics would never match it anyway.
  PatternMatchResult::failed()
}

/// Top-level entry point for concrete-syntax matching.
///
/// Dispatches each element of the resolved pattern—capture groups or literal text—against the AST
/// at the current cursor position, recursing until the pattern is fully applied.
pub(crate) fn match_cs_pattern(
  ctx: &mut MatchingContext<'_>, cs_elements: &[ResolvedCsElement], can_continue: bool,
) -> PatternMatchResult {
  // Check if we finished matching! In case, we haven't `check_match_completion` returns None
  if let Some(result) = check_match_completion(ctx, cs_elements, can_continue) {
    return result;
  }

  // Skip comment nodes always
  CursorNavigator::skip_comment_nodes(&mut ctx.cursor);

  // Get the first element and remaining elements
  let (first, rest) = cs_elements.split_first().unwrap();

  match first {
    ResolvedCsElement::Capture {
      name,
      mode,
      constraints,
    } => match mode {
      CaptureMode::Single => match_single_capture(ctx, name, constraints, rest),
      CaptureMode::OnePlus => match_one_plus_capture(ctx, name, constraints, rest),
      CaptureMode::ZeroPlus => match_zero_plus_capture(ctx, name, constraints, rest),
    },
    ResolvedCsElement::Literal(literal_text) => match_literal(ctx, literal_text, rest),
  }
}

fn match_literal(
  ctx: &mut MatchingContext<'_>, literal_text: &str, remaining_elements: &[ResolvedCsElement],
) -> PatternMatchResult {
  // We match literals against leaves
  while ctx.cursor.node().child_count() != 0 {
    ctx.cursor.goto_first_child();
  }

  let node_code = ctx.cursor.node().utf8_text(ctx.source_code).unwrap().trim();
  if literal_text.starts_with(node_code) && !node_code.is_empty() {
    let advance_by = node_code.len();
    // Can only advance if there is still enough chars to consume
    if advance_by > literal_text.len() {
      return PatternMatchResult::failed();
    }

    let should_match = CursorNavigator::find_next_sibling_or_ancestor_sibling(&mut ctx.cursor);

    // If we consumed the entire literal, continue with remaining elements
    return if advance_by == literal_text.len() {
      match_cs_pattern(ctx, remaining_elements, should_match)
    } else {
      // If we only consumed part of the literal, create a new literal with the remaining text
      let remaining_literal = &literal_text[advance_by..];
      let mut new_elements = vec![ResolvedCsElement::Literal(remaining_literal.to_string())];
      new_elements.extend_from_slice(remaining_elements);
      match_cs_pattern(ctx, &new_elements, should_match)
    };
  }
  PatternMatchResult::failed()
}

// =============================================================================
// CAPTURE MODE HANDLERS
// =============================================================================

/// Handle single capture: :[var] - must match exactly one node
fn match_single_capture(
  ctx: &mut MatchingContext<'_>, var_name: &str, constraints: &[CsConstraint],
  remaining_pattern: &[ResolvedCsElement],
) -> PatternMatchResult {
  match_at_all_tree_levels(ctx, var_name, constraints, remaining_pattern, false)
}

/// Handle one-plus capture: :[var+] - must match one or more nodes  
fn match_one_plus_capture(
  ctx: &mut MatchingContext<'_>, var_name: &str, constraints: &[CsConstraint],
  remaining_pattern: &[ResolvedCsElement],
) -> PatternMatchResult {
  match_at_all_tree_levels(ctx, var_name, constraints, remaining_pattern, true)
}

/// Handle zero-plus capture: :[var*] - can match zero or more nodes
fn match_zero_plus_capture(
  ctx: &mut MatchingContext<'_>, var_name: &str, constraints: &[CsConstraint],
  remaining_pattern: &[ResolvedCsElement],
) -> PatternMatchResult {
  // First try to match with zero nodes
  let mut zero_match_ctx = MatchingContext {
    cursor: ctx.cursor.clone(),
    source_code: ctx.source_code,
    top_node: ctx.top_node,
  };
  let empty_capture = create_empty_captured_node();
  if satisfies_constraints(&empty_capture, constraints) {
    let zero_match_result = match_cs_pattern(&mut zero_match_ctx, remaining_pattern, true);
    if let PatternMatchResult::Success {
      captures: mut zero_captures,
      consumed_nodes: last_matched_node_idx,
      range: None,
    } = zero_match_result
    {
      zero_captures.insert(var_name.to_string(), empty_capture);
      return PatternMatchResult::success(zero_captures, last_matched_node_idx);
    }
  }

  // If zero nodes didn't work, try one or more nodes
  match_at_all_tree_levels(ctx, var_name, constraints, remaining_pattern, true)
}

/// Attempts to match a capture by exploring different levels of the AST hierarchy.
/// Starting from the current node, it tries to match progressively smaller subtrees
/// by traversing down to child nodes. This allows flexible matching where a capture
/// can match a large statement block or just a single expression within it.
///
/// For example, given: `int x = 1; x++;`
/// The capture `:[x]` could match:
/// - The entire statement block: `int x = 1; x++;`
/// - Just the declaration: `int x = 1;`
/// - Just the type: `int`
fn match_at_all_tree_levels(
  ctx: &mut MatchingContext<'_>, var_name: &str, constraints: &[CsConstraint],
  remaining_pattern: &[ResolvedCsElement], allow_horizontal_expansion: bool,
) -> PatternMatchResult {
  // Try matching at different tree levels, going deeper each iteration
  loop {
    let result = try_match_node_range(
      ctx,
      var_name,
      constraints,
      remaining_pattern,
      allow_horizontal_expansion,
    );
    if let PatternMatchResult::Success { .. } = result {
      return result;
    }

    // Move one level down to try matching against smaller/deeper nodes
    if !ctx.cursor.goto_first_child() {
      break;
    }
  }
  PatternMatchResult::failed()
}

/// Try to match a range of nodes starting at the current cursor position, expanding the range if needed and allowed
/// It assigns [range_start, range_end] to a capture group, and matches the rest of the cs pattern against remaining nodes
///
/// Per‐iteration diagram:
///
///     AST siblings: … ─ node₀ ─ node₁ ─ node₂ ─ node₃ ─ node₄ ─ …  
///                          ↑         ↑  
///                    range_start range_end
///
///   1) capture = text(node₁…node₂)  
///   2) match_cs_pattern(  
///        remaining_elements,         // CS elements still to match  
///        /* should_match */ true if node₃ exists  
///      ) starting at node₃, node₄, …
fn try_match_node_range(
  ctx: &mut MatchingContext<'_>, var_name: &str, constraints: &[CsConstraint],
  remaining_pattern: &[ResolvedCsElement], allow_horizontal_expansion: bool,
) -> PatternMatchResult {
  // 1. Initial anchors
  let range_start = ctx.cursor.node();
  let mut range_end = range_start;
  let mut next_cursor = ctx.cursor.clone();
  // 'should_match' is used to check whether there are siblings or ancestor siblings we need to match
  // after we assign the current range to the capture node
  let mut should_match = CursorNavigator::find_next_sibling_or_ancestor_sibling(&mut next_cursor);
  let mut is_last = false;

  // Helper to slice out the captured text
  let make_capture = |end: &Node| CapturedNode {
    range: Range::span_ranges(range_start.range(), end.range()),
    text: CursorNavigator::get_text_from_range(
      range_start.range().start_byte,
      end.range().end_byte,
      ctx.source_code,
    ),
  };

  loop {
    // ——— 1) try current [start…end] slice ———
    let captured = make_capture(&range_end);
    if satisfies_constraints(&captured, constraints) {
      let mut sub_ctx = MatchingContext {
        cursor: next_cursor.clone(),
        source_code: ctx.source_code,
        top_node: ctx.top_node,
      };
      if let PatternMatchResult::Success {
        mut captures,
        consumed_nodes,
        range: _,
      } = match_cs_pattern(&mut sub_ctx, remaining_pattern, should_match)
      {
        // conflict check
        if let Some(prev) = captures.get(var_name) {
          if prev.text.trim() != captured.text.trim() {
            return PatternMatchResult::failed();
          }
        }
        captures.insert(var_name.to_owned(), captured);
        return PatternMatchResult::success(captures, consumed_nodes);
      }
    }

    // ——— 2) need to expand slice? ———
    if !allow_horizontal_expansion || is_last {
      return PatternMatchResult::failed();
    }

    // grow the range to include the node under next_cursor
    range_end = next_cursor.node();
    is_last = !next_cursor.goto_next_sibling();
    if is_last {
      // once we’re at the end of the range, check if there is any ancestor left to match
      should_match = CursorNavigator::find_next_sibling_or_ancestor_sibling(&mut next_cursor);
    }
  }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Helper function to create a match from captured nodes
fn create_match_from_capture(
  replace_node_key: &str, match_map: HashMap<String, CapturedNode>, _range: Range,
) -> Match {
  let replace_node_match = match_map.get(replace_node_key).cloned().unwrap_or_else(|| {
    panic!("The tag {replace_node_key} provided in the replace node is not present")
  });

  Match {
    matched_string: replace_node_match.text,
    range: replace_node_match.range,
    matches: match_map.into_iter().map(|(k, v)| (k, v.text)).collect(),
    associated_comma: None,
    associated_comments: Vec::new(),
    associated_leading_empty_lines: Vec::new(),
  }
}

/// Handle the case where pattern is empty or nodes are exhausted
fn check_match_completion(
  ctx: &mut MatchingContext<'_>, cs_elements: &[ResolvedCsElement], can_continue: bool,
) -> Option<PatternMatchResult> {
  if cs_elements.is_empty() {
    if !can_continue {
      return Some(PatternMatchResult::success(
        HashMap::new(),
        ctx.top_node.child_count() - 1,
      ));
    }
    let index = find_last_matched_node(&mut ctx.cursor, ctx.top_node);
    return match index {
      Some(consumed_nodes) => Some(PatternMatchResult::success(HashMap::new(), consumed_nodes)),
      None => Some(PatternMatchResult::failed()),
    };
  } else if !can_continue {
    return Some(PatternMatchResult::failed());
  }
  None
}

/// Finds the index of the last matched node relative to the top level node.
/// Returns the index of the child node where matching concluded, or 0 if the cursor
/// is not positioned on a child of the parent node.
fn find_last_matched_node(cursor: &mut TreeCursor, parent_node: &Node) -> Option<usize> {
  CursorNavigator::find_child_index(&cursor.node(), parent_node)
    .map(|i| if i > 0 { i - 1 } else { 0 })
}

/// Create an empty captured node for zero-match patterns
fn create_empty_captured_node() -> CapturedNode {
  CapturedNode {
    range: Range {
      start_byte: 0,
      end_byte: 0,
      start_point: crate::models::matches::Point { row: 0, column: 0 },
      end_point: crate::models::matches::Point { row: 0, column: 0 },
    },
    text: String::new(),
  }
}

#[cfg(test)]
#[path = "unit_tests/interpreter_test.rs"]
mod interpreter_test;
