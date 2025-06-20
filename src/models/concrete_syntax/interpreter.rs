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

//! Concrete Syntax Interpreter
//!
//! This module provides the matching algorithm that applies concrete syntax patterns
//! against tree-sitter ASTs. It handles the sophisticated pattern matching logic for
//! alternations, sequences, and constraints.

use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};

use super::parser::{
  CaptureMode, ComparisonOp, ConcreteSyntax, ConstraintKind, CsConstraint, CsElement, CsPattern,
  CsValue,
};
use crate::models::matches::{Match, Point, Range};

/// Interpreter that uses the AST to match against tree-sitter nodes
pub struct CsInterpreter;

// Captured node representation
#[derive(Clone, PartialEq, Eq)]
pub struct CapturedNode {
  pub range: Range,
  pub text: String,
}

#[derive(Clone, PartialEq, Eq)]
struct MatchResult {
  mapping: HashMap<String, CapturedNode>,
  range: Range,
}

impl CsInterpreter {
  pub fn get_all_matches(
    node: &Node, code_str: &[u8], cs: &ConcreteSyntax, recursive: bool,
    replace_node: Option<String>,
  ) -> (Vec<Match>, bool) {
    let mut matches: Vec<Match> = Vec::new();
    // Try to match the pattern against the current node using the sophisticated algorithm
    if let Some(match_result) = Self::match_sequential_siblings(&mut node.walk(), code_str, cs) {
      // Check constraints
      if Self::check_constraints(&match_result.mapping, &cs.constraints) {
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
            text: Self::get_code_from_range(range.start_byte, range.end_byte, code_str),
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
    }

    // Recursively search child nodes if requested
    if recursive {
      let mut cursor = node.walk();
      for child in node.children(&mut cursor) {
        if let (mut inner_matches, true) =
          Self::get_all_matches(&child, code_str, cs, recursive, replace_node.clone())
        {
          matches.append(&mut inner_matches);
        }
      }
    }

    let is_empty = matches.is_empty();
    (matches, !is_empty)
  }

  /// Sophisticated matching algorithm adapted from the original implementation
  /// This handles the complex pattern matching logic for alternations and sequences
  fn match_sequential_siblings(
    cursor: &mut TreeCursor, source_code: &[u8], cs: &ConcreteSyntax,
  ) -> Option<MatchResult> {
    match &cs.pattern {
      CsPattern::Alternation(alternatives) => {
        // Try each alternative until one matches
        for alternative in alternatives {
          let mut temp_cursor = cursor.clone();
          let temp_cs = ConcreteSyntax {
            pattern: alternative.clone(),
            constraints: cs.constraints.clone(),
          };
          if let Some(result) =
            Self::match_sequential_siblings(&mut temp_cursor, source_code, &temp_cs)
          {
            *cursor = temp_cursor;
            return Some(result);
          }
        }
        None
      }
      CsPattern::Sequence(elements) => Self::match_sequence_pattern(cursor, source_code, elements),
    }
  }

  fn match_sequence_pattern(
    cursor: &mut TreeCursor, source_code: &[u8], elements: &[CsElement],
  ) -> Option<MatchResult> {
    let parent_node = cursor.node();
    let mut child_seq_match_start = 0;

    if cursor.goto_first_child() {
      // Iterate through siblings to find a match
      loop {
        let mut tmp_cursor = cursor.clone();
        let (mapping, indx) = Self::get_matches_for_element_sequence(
          &mut tmp_cursor,
          source_code,
          elements,
          true,
          &parent_node,
        );

        // If we got the index of the last matched sibling, the matching was successful
        if let Some(last_node_index) = indx {
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
    }
    None
  }

  /// Core matching algorithm adapted from get_matches_for_subsequence_of_nodes
  fn get_matches_for_element_sequence(
    cursor: &mut TreeCursor, source_code: &[u8], elements: &[CsElement], nodes_left_to_match: bool,
    top_node: &Node,
  ) -> (HashMap<String, CapturedNode>, Option<usize>) {
    if elements.is_empty() {
      if !nodes_left_to_match {
        return (HashMap::new(), Some(top_node.child_count() - 1));
      }
      let index = Self::find_last_matched_node(cursor, top_node);
      return (HashMap::new(), index);
    } else if !nodes_left_to_match {
      return (HashMap::new(), None);
    }

    let mut node = cursor.node();
    // Skip comment nodes always
    while node.kind().contains("comment") && cursor.goto_next_sibling() {
      node = cursor.node();
    }

    let first_element = &elements[0];
    let remaining_elements = &elements[1..];

    match first_element {
      CsElement::Capture { name, mode } => Self::handle_template_variable_matching_v3(
        cursor,
        source_code,
        top_node,
        name,
        *mode,
        remaining_elements,
      ),
      CsElement::Literal(text) => {
        if node.child_count() == 0 {
          Self::handle_leaf_node_v3(cursor, source_code, text, remaining_elements, top_node)
        } else {
          cursor.goto_first_child();
          Self::get_matches_for_element_sequence(cursor, source_code, elements, true, top_node)
        }
      }
    }
  }

  fn handle_template_variable_matching_v3(
    cursor: &mut TreeCursor, source_code: &[u8], top_node: &Node, var_name: &str,
    mode: CaptureMode, remaining_elements: &[CsElement],
  ) -> (HashMap<String, CapturedNode>, Option<usize>) {
    // For zero_plus patterns, first try to match with zero nodes
    if mode == CaptureMode::ZeroPlus {
      let mut tmp_cursor = cursor.clone();
      if let (mut recursive_matches, Some(last_matched_node_idx)) =
        Self::get_matches_for_element_sequence(
          &mut tmp_cursor,
          source_code,
          remaining_elements,
          true,
          top_node,
        )
      {
        // Successfully matched with zero nodes
        recursive_matches.insert(
          var_name.to_string(),
          CapturedNode {
            range: Range {
              start_byte: 0,
              end_byte: 0,
              start_point: Point { row: 0, column: 0 },
              end_point: Point { row: 0, column: 0 },
            },
            text: String::new(),
          },
        );
        return (recursive_matches, Some(last_matched_node_idx));
      }
    }

    // Match variable against sequence of nodes
    loop {
      let first_node = cursor.node();
      let mut last_node = first_node;

      // Determine whether a next node exists
      let mut next_node_cursor = cursor.clone();
      let mut should_match = Self::find_next_sibling_or_ancestor_sibling(&mut next_node_cursor);

      let mut is_final_sibling = false;
      loop {
        let mut tmp_cursor = next_node_cursor.clone();

        if let (mut recursive_matches, Some(last_matched_node_idx)) =
          Self::get_matches_for_element_sequence(
            &mut tmp_cursor,
            source_code,
            remaining_elements,
            should_match,
            top_node,
          )
        {
          // Continuous code range that variable is matching
          let matched_code = Self::get_code_from_range(
            first_node.range().start_byte,
            last_node.range().end_byte,
            source_code,
          );

          // Check if variable was already matched against some code range
          if recursive_matches.contains_key(var_name)
            && recursive_matches[var_name].text.trim() != matched_code.trim()
          {
            return (HashMap::new(), None);
          }

          // Insert the matched variable
          recursive_matches.insert(
            var_name.to_string(),
            CapturedNode {
              range: Range::span_ranges(first_node.range(), last_node.range()),
              text: matched_code,
            },
          );
          return (recursive_matches, Some(last_matched_node_idx));
        }

        // Append an extra node to match with the variable
        last_node = next_node_cursor.node();
        if is_final_sibling {
          break;
        }

        is_final_sibling = !next_node_cursor.goto_next_sibling();
        if is_final_sibling {
          should_match = Self::find_next_sibling_or_ancestor_sibling(&mut next_node_cursor);
        }

        if mode == CaptureMode::Single {
          break;
        }
      }

      // Move one level down to attempt matching against smaller nodes
      if !cursor.goto_first_child() {
        break;
      }
    }
    (HashMap::new(), None)
  }

  fn handle_leaf_node_v3(
    cursor: &mut TreeCursor, source_code: &[u8], literal_text: &str,
    remaining_elements: &[CsElement], top_node: &Node,
  ) -> (HashMap<String, CapturedNode>, Option<usize>) {
    let code = cursor.node().utf8_text(source_code).unwrap().trim();
    if literal_text.starts_with(code) && !code.is_empty() {
      let advance_by = code.len();
      if advance_by > literal_text.len() {
        return (HashMap::new(), None);
      }

      // Create remaining tokens with updated literal
      let mut updated_elements = Vec::new();
      if advance_by < literal_text.len() {
        updated_elements.push(CsElement::Literal(
          literal_text[advance_by..].trim_start().to_string(),
        ));
      }
      updated_elements.extend_from_slice(remaining_elements);

      let should_match = Self::find_next_sibling_or_ancestor_sibling(cursor);
      return Self::get_matches_for_element_sequence(
        cursor,
        source_code,
        &updated_elements,
        should_match,
        top_node,
      );
    }
    (HashMap::new(), None)
  }

  fn find_next_sibling_or_ancestor_sibling(cursor: &mut TreeCursor) -> bool {
    while !cursor.goto_next_sibling() {
      if !cursor.goto_parent() {
        return false;
      }
    }
    true
  }

  fn find_last_matched_node(cursor: &mut TreeCursor, parent_node: &Node) -> Option<usize> {
    parent_node
      .children(&mut parent_node.walk())
      .enumerate()
      .filter(|&(_i, child)| child == cursor.node())
      .map(|(i, _child)| i.saturating_sub(1))
      .next()
  }

  fn check_constraints(
    mapping: &HashMap<String, CapturedNode>, constraints: &[CsConstraint],
  ) -> bool {
    for constraint in constraints {
      if !Self::check_constraint(mapping, constraint) {
        return false;
      }
    }
    true
  }

  fn check_constraint(mapping: &HashMap<String, CapturedNode>, constraint: &CsConstraint) -> bool {
    match &constraint.kind {
      ConstraintKind::Matches { capture, regex } => {
        if let Some(captured) = mapping.get(capture) {
          regex.is_match(&captured.text)
        } else {
          false
        }
      }
      ConstraintKind::Length { capture, op, value } => {
        if let Some(captured) = mapping.get(capture) {
          let len = captured.text.split(',').count();
          Self::compare_numbers(len, *op, *value)
        } else {
          false
        }
      }
      ConstraintKind::Comparison { capture, op, value } => {
        if let Some(captured) = mapping.get(capture) {
          Self::compare_value(&captured.text, *op, value)
        } else {
          false
        }
      }
    }
  }

  fn compare_numbers(left: usize, op: ComparisonOp, right: usize) -> bool {
    match op {
      ComparisonOp::Eq => left == right,
      ComparisonOp::NotEq => left != right,
      ComparisonOp::Lt => left < right,
      ComparisonOp::Le => left <= right,
      ComparisonOp::Gt => left > right,
      ComparisonOp::Ge => left >= right,
    }
  }

  fn compare_value(left: &str, op: ComparisonOp, right: &CsValue) -> bool {
    match right {
      CsValue::String(s) => match op {
        ComparisonOp::Eq => left == s,
        ComparisonOp::NotEq => left != s,
        _ => false, // Other comparisons don't make sense for strings
      },
      CsValue::Number(n) => {
        if let Ok(left_num) = left.parse::<f64>() {
          match op {
            ComparisonOp::Eq => (left_num - n).abs() < f64::EPSILON,
            ComparisonOp::NotEq => (left_num - n).abs() >= f64::EPSILON,
            ComparisonOp::Lt => left_num < *n,
            ComparisonOp::Le => left_num <= *n,
            ComparisonOp::Gt => left_num > *n,
            ComparisonOp::Ge => left_num >= *n,
          }
        } else {
          false
        }
      }
      CsValue::Boolean(b) => {
        let left_bool = left == "true";
        match op {
          ComparisonOp::Eq => left_bool == *b,
          ComparisonOp::NotEq => left_bool != *b,
          _ => false,
        }
      }
    }
  }

  fn get_code_from_range(start_byte: usize, end_byte: usize, source_code: &[u8]) -> String {
    let text_slice = &source_code[start_byte..end_byte];
    String::from_utf8_lossy(text_slice).to_string()
  }
}

/// Convenience function to parse and match concrete syntax
pub fn parse_and_match(
  cs_string: &str, node: &Node, code_str: &[u8], recursive: bool, replace_node: Option<String>,
) -> Result<(Vec<Match>, bool), String> {
  let cs = ConcreteSyntax::parse(cs_string)?;
  Ok(CsInterpreter::get_all_matches(
    node,
    code_str,
    &cs,
    recursive,
    replace_node,
  ))
}
