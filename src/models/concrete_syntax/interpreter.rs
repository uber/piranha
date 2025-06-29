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

use regex::Regex;

use crate::models::concrete_syntax::cursor_utils::CursorNavigator;
use crate::models::concrete_syntax::parser::CsConstraint;
use crate::models::concrete_syntax::resolver::{ResolvedConcreteSyntax, ResolvedCsElement};
use crate::models::concrete_syntax::types::{CapturedNode, MatchingContext, PatternMatchResult};
use crate::models::matches::Match;
use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};
use tree_sitter_traversal::Cursor;

// Precompile the regex outside the function
lazy_static! {
  static ref RE_VAR: Regex = Regex::new(r"^:\[(?P<var_name>\w+)\]").unwrap();
  static ref RE_VAR_PLUS: Regex = Regex::new(r"^:\[(?P<var_name>\w+)\+\]").unwrap();
}

#[derive(Clone, PartialEq, Eq)]
struct MatchResult {
  mapping: HashMap<String, CapturedNode>,
  range: Range,
}

pub(crate) fn get_all_matches_for_concrete_syntax(
  node: &Node, code_str: &[u8], cs: &ResolvedConcreteSyntax, recursive: bool,
  replace_node: Option<String>,
) -> (Vec<Match>, bool) {
  let mut matches: Vec<Match> = Vec::new();

  if let Some(match_result) =
    match_sequential_siblings(&mut node.walk(), code_str, &cs.pattern.sequence)
  {
    let replace_node_key = replace_node.clone().unwrap_or("*".to_string());
    let mut match_map = match_result.mapping;
    let range = match_result.range;
    let replace_node_match = if replace_node_key != "*" {
      match_map
        .get(&replace_node_key)
        .cloned()
        .unwrap_or_else(|| {
          panic!("The tag {replace_node_key} provided in the replace node is not present")
        })
    } else {
      CapturedNode {
        range,
        text: CursorNavigator::get_text_from_range(range.start_byte, range.end_byte, code_str),
      }
    };

    match_map.insert(replace_node_key, replace_node_match.clone());
    matches.push(Match {
      matched_string: replace_node_match.text,
      range: replace_node_match.range,
      matches: match_map.into_iter().map(|(k, v)| (k, v.text)).collect(),
      associated_comma: None,
      associated_comments: Vec::new(),
      associated_leading_empty_lines: Vec::new(),
    });
  }
  if recursive {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
      if let (mut inner_matches, true) =
        get_all_matches_for_concrete_syntax(&child, code_str, cs, recursive, replace_node.clone())
      {
        matches.append(&mut inner_matches);
      }
    }
  }

  let is_empty = matches.is_empty();
  (matches, !is_empty)
}

/// Attempts to match a given ResolvedConcreteSyntax template against a sequence of sibling nodes
/// in an Abstract Syntax Tree (AST).
///
/// # Arguments
///
/// * `cursor` - A mutable reference to the TreeCursor, which is used to navigate the AST.
/// * `source_code` - A slice of bytes representing the source code being analyzed.
/// * `cs` - A reference to the ResolvedConcreteSyntax template used for matching.
///
/// # Returns
///
/// A tuple containing:
/// * A HashMap where keys are variable names from the template and values are CapturedNode instances representing matched AST nodes.
/// * A boolean indicating whether a match was found.
/// * An Option containing the range of matched nodes if a match was found.
///
/// # Algorithm
///
/// 1. Initialize cursor to the first child and iterate through siblings.
/// 2. Use `get_matches_for_subsequence_of_nodes` to attempt matching the template against a sequence of subtree starting at each sibling.
/// 3. If a match is found, determine the range of matched nodes subtrees (i.e., [2nd,..., 4th], and return the match mapping, and range.
/// 4. If no match is found, return an empty mapping, and None for range.
fn match_sequential_siblings(
  cursor: &mut TreeCursor, source_code: &[u8], cs_elements: &Vec<ResolvedCsElement>,
) -> Option<MatchResult> {
  let parent_node = cursor.node();
  let mut child_seq_match_start = 0;
  if cursor.goto_first_child() {
    // Iterate through siblings to find a match
    loop {
      // Clone the cursor in order to attempt matching the sequence starting at cursor.node
      // Cloning here is necessary other we won't be able to advance to the next sibling if the matching fails
      let result = {
        let mut ctx = MatchingContext {
          cursor: cursor.clone(),
          source_code,
          top_node: &parent_node,
        };
        get_matches_for_subsequence_of_nodes(&mut ctx, cs_elements, true)
      };

      // If we got a successful match, extract the mapping and index
      if let PatternMatchResult::Success {
        captures: mapping,
        consumed_nodes: last_node_index,
      } = result
      {
        // Determine the last matched node. Remember, we are matching subsequences of children [n ... k]
        let last_node = parent_node.child(last_node_index);
        let range = Range::span_ranges(cursor.node().range(), last_node.unwrap().range());
        if last_node_index != child_seq_match_start || parent_node.child_count() == 1 {
          return Some(MatchResult { mapping, range });
        }
        return None;
      }

      child_seq_match_start += 1;
      if !cursor.goto_next_sibling() {
        break;
      }
    }
  } // Not currently handing matching of leaf nodes. Current semantics would never match it anyway.
  None
}

/// This function performs the actual matching of the ResolvedConcreteSyntax pattern against a syntax tree
/// node. The matching is done in the following way:
///
///
/// - If the ResolvedConcreteSyntax is empty and all the nodes have been visited, then we found a match!
///   Otherwise, if we ran out of nodes to match, and the template is not empty, then we failed
///
/// - If the ResolvedConcreteSyntax starts with `:[variable]`, the function tries to match the variable
///   against all possible AST nodes starting at the current's cursor position (i.e., the node itself
///   and all of its siblings, its first child and respective siblings, the child of the first child, and so on.)
///   If it succeeds, it advances the ResolvedConcreteSyntax by the length of the matched sequence of
///   AST nodes, and calls itself recursively to try to match the rest of the ResolvedConcreteSyntax.
///
/// - If the ResolvedConcreteSyntax doesn't start with `:[variable]`, the function checks if the node **is a leaf**
///   (i.e., has no children). If it is, and the leaf node matches the concrete syntax, we match it against
///   the concrete syntax and advance to the next immediate node. If the leaf does not match the concrete syntax,
///   then our matching has failed.
///
/// - If the ResolvedConcreteSyntax doesn't start with `:[variable]` and the node **is not a leaf**, the function
///   moves the cursor to the first child of the node and calls itself recursively to try to match
///   the ResolvedConcreteSyntax.
pub(crate) fn get_matches_for_subsequence_of_nodes(
  ctx: &mut MatchingContext<'_>, cs_elements: &Vec<ResolvedCsElement>, nodes_left_to_match: bool,
) -> PatternMatchResult {
  if cs_elements.is_empty() {
    if !nodes_left_to_match {
      return PatternMatchResult::success(HashMap::new(), ctx.top_node.child_count() - 1);
    }
    let index = find_last_matched_node(&mut ctx.cursor, ctx.top_node);
    return match index {
      Some(consumed_nodes) => PatternMatchResult::success(HashMap::new(), consumed_nodes),
      None => PatternMatchResult::failed(),
    };
  } else if !nodes_left_to_match {
    return PatternMatchResult::failed();
  }

  // Skip comment nodes always
  CursorNavigator::skip_comment_nodes(&mut ctx.cursor);
  let node = ctx.cursor.node();

  // Get the first element and remaining elements
  let first_element = &cs_elements[0];
  let remaining_elements = &cs_elements[1..].to_vec();

  match first_element {
    ResolvedCsElement::Capture { .. } => {
      // Handle template variable matching
      handle_template_variable_matching_elements(ctx, first_element, remaining_elements)
    }
    ResolvedCsElement::Literal(literal_text) => {
      if node.child_count() == 0 {
        // If the current node is a leaf
        handle_leaf_node_elements(ctx, literal_text, remaining_elements)
      } else {
        // If the current node is an intermediate node
        ctx.cursor.goto_first_child();
        get_matches_for_subsequence_of_nodes(ctx, cs_elements, true)
      }
    }
  }
}

/// This function does the template variable matching against entire tree nodes.
/// It handles different matching modes:
/// - Single: Match exactly one node :[var]
/// - OnePlus: Match one or more nodes :[var+]
/// - ZeroPlus: Match zero or more nodes :[var*]
///
/// For successful matches, it returns the assignment of each template variable against a
/// particular range. The Option<usize> indicates whether a match was successful, and keeps
fn handle_template_variable_matching_elements(
  ctx: &mut MatchingContext<'_>, capture_element: &ResolvedCsElement,
  remaining_elements: &Vec<ResolvedCsElement>,
) -> PatternMatchResult {
  use crate::models::concrete_syntax::parser::CaptureMode;

  // Extract capture details
  let (var_name, mode, constraints) = match capture_element {
    ResolvedCsElement::Capture {
      name,
      mode,
      constraints,
    } => (name, *mode, constraints),
    _ => panic!("Expected capture element"),
  };

  // For zero_plus patterns, first try to match with zero nodes
  if mode == CaptureMode::ZeroPlus {
    let mut temp_ctx = MatchingContext {
      cursor: ctx.cursor.clone(),
      source_code: ctx.source_code,
      top_node: ctx.top_node,
    };
    let result = get_matches_for_subsequence_of_nodes(&mut temp_ctx, remaining_elements, true);
    if let PatternMatchResult::Success {
      captures: mut recursive_matches,
      consumed_nodes: last_matched_node_idx,
    } = result
    {
      // Successfully matched with zero nodes
      let captured_node = CapturedNode {
        range: Range {
          start_byte: 0,
          end_byte: 0,
          start_point: crate::models::matches::Point { row: 0, column: 0 },
          end_point: crate::models::matches::Point { row: 0, column: 0 },
        },
        text: String::new(),
      };

      // Check all constraints for this capture (empty string case)
      let mut constraints_satisfied = true;
      for constraint in constraints {
        if !check_constraint(&captured_node, constraint) {
          constraints_satisfied = false;
          break;
        }
      }

      if constraints_satisfied {
        recursive_matches.insert(var_name.to_string(), captured_node);
        return PatternMatchResult::success(recursive_matches, last_matched_node_idx);
      }
    }
  }
  // Matching :[var] against a sequence of nodes [first_node, ... last_node]
  loop {
    let first_node = ctx.cursor.node();
    let mut last_node = first_node;

    // Determine whether a next node exists:
    let mut next_node_cursor = ctx.cursor.clone();
    let mut should_match =
      CursorNavigator::find_next_sibling_or_ancestor_sibling(&mut next_node_cursor);
    // At this point next_node_cursor either points to the first sibling of the first node,
    // or the first node itself, if such sibling no longer exists

    // Intentionally setting is_final_sibling to false regardless of should_match, due to the logic of handling the last iteration
    let mut is_final_sibling = false;
    loop {
      let remaining_elements_tmp = remaining_elements.clone();
      let mut temp_ctx = MatchingContext {
        cursor: next_node_cursor.clone(),
        source_code: ctx.source_code,
        top_node: ctx.top_node,
      };
      let result =
        get_matches_for_subsequence_of_nodes(&mut temp_ctx, &remaining_elements_tmp, should_match);

      if let PatternMatchResult::Success {
        captures: mut recursive_matches,
        consumed_nodes: last_matched_node_idx,
      } = result
      {
        // Continuous code range that :[var] is matching from [first, ..., last]
        let matched_code = CursorNavigator::get_text_from_range(
          first_node.range().start_byte,
          last_node.range().end_byte,
          ctx.source_code,
        );

        // Check if :[var] was already matched against some code range
        // If it did, and it is not the same, we return unsuccessful
        if recursive_matches.contains_key(var_name)
          && recursive_matches[var_name].text.trim() != matched_code.trim()
        {
          return PatternMatchResult::failed();
        }

        let captured_node = CapturedNode {
          range: Range::span_ranges(first_node.range(), last_node.range()),
          text: matched_code,
        };

        // Check all constraints for this capture
        let mut constraints_satisfied = true;
        for constraint in constraints {
          if !check_constraint(&captured_node, constraint) {
            constraints_satisfied = false;
            break;
          }
        }

        if !constraints_satisfied {
          // Continue to try matching with more nodes or different positions
          // The continue here will skip to the next iteration of the inner matching loop
          break;
        }

        // Otherwise insert it
        recursive_matches.insert(var_name.to_string(), captured_node);
        return PatternMatchResult::success(recursive_matches, last_matched_node_idx);
      }

      // Append an extra node to match with :[var]. Remember we had advanced next_node_cursor before,
      // therefore we cannot advance it again, otherwise we would skip nodes.
      // We only attempt to append an extra code if we are in one_plus matching mode.
      last_node = next_node_cursor.node();
      if is_final_sibling {
        break;
      }

      // This is used for the final iteration. We need to determine if there are any other nodes
      // left to match, to inform our next recursive call. We do this by calling find_next_sibling_or_ancestor_sibling
      // to move the cursor to the parent and find the next sibling at another level,
      // since at this level we already matched everything
      is_final_sibling = !next_node_cursor.goto_next_sibling();
      if is_final_sibling {
        should_match =
          CursorNavigator::find_next_sibling_or_ancestor_sibling(&mut next_node_cursor);
      }

      if mode == CaptureMode::Single {
        break;
      }
    }

    // Move one level down, to attempt to match the template variable :[var] against smaller nodes.
    if !ctx.cursor.goto_first_child() {
      break;
    }
  }
  PatternMatchResult::failed()
}

fn handle_leaf_node_elements(
  ctx: &mut MatchingContext<'_>, literal_text: &str, remaining_elements: &Vec<ResolvedCsElement>,
) -> PatternMatchResult {
  let code = ctx.cursor.node().utf8_text(ctx.source_code).unwrap().trim();
  if literal_text.starts_with(code) && !code.is_empty() {
    let advance_by = code.len();
    // Can only advance if there is still enough chars to consume
    if advance_by > literal_text.len() {
      return PatternMatchResult::failed();
    }

    let should_match = CursorNavigator::find_next_sibling_or_ancestor_sibling(&mut ctx.cursor);

    // If we consumed the entire literal, continue with remaining elements
    if advance_by == literal_text.len() {
      return get_matches_for_subsequence_of_nodes(ctx, remaining_elements, should_match);
    } else {
      // If we only consumed part of the literal, create a new literal with the remaining text
      let remaining_literal = &literal_text[advance_by..];
      let mut new_elements = vec![ResolvedCsElement::Literal(remaining_literal.to_string())];
      new_elements.extend_from_slice(remaining_elements);

      return get_matches_for_subsequence_of_nodes(ctx, &new_elements, should_match);
    }
  }
  PatternMatchResult::failed()
}

/// Finds the index of the last matched node relative to the `match_sequential_siblings` function.
///
/// This function checks if the matching concluded on a child of the node where `match_sequential_siblings`
/// was invoked. If so, it returns the index of that child.
fn find_last_matched_node(cursor: &mut TreeCursor, parent_node: &Node) -> Option<usize> {
  CursorNavigator::find_child_index(&cursor.node(), parent_node)
    .map(|i| if i > 0 { i - 1 } else { 0 })
}

fn check_constraint(node: &CapturedNode, ctr: &CsConstraint) -> bool {
  match ctr {
    CsConstraint::In { items, .. } => items.contains(&node.text.to_string()),
  }
}

#[cfg(test)]
#[path = "unit_tests/interpreter_test.rs"]
mod interpreter_test;
