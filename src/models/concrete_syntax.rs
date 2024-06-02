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

use itertools::Itertools;
use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};
use tree_sitter_traversal::Cursor;

use crate::models::capture_group_patterns::ConcreteSyntax;
use crate::models::matches::Match;

// Precompile the regex outside the function
lazy_static! {
  static ref RE_VAR: Regex = Regex::new(r"^:\[(?P<var_name>\w+)\]").unwrap();
}

// Struct to avoid dealing with lifetimes
#[derive(Clone, PartialEq, Eq)]
pub struct CapturedNode {
  range: Range,
  text: String,
}

pub(crate) fn get_all_matches_for_concrete_syntax(
  node: &Node, code_str: &[u8], meta: &ConcreteSyntax, recursive: bool,
  replace_node: Option<String>,
) -> (Vec<Match>, bool) {
  let mut matches: Vec<Match> = Vec::new();

  if let (mut match_map, true, Some(range)) =
    get_matched_for_sequential_siblings(&mut node.walk(), code_str, meta)
  {
    let replace_node_key = replace_node.clone().unwrap_or("*".to_string());

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
        text: get_code_from_range(range.start_byte, range.end_byte, code_str),
      }
    };

    match_map.insert(replace_node_key, replace_node_match.clone());

    matches.push(Match {
      matched_string: replace_node_match.text,
      range: replace_node_match.range,
      matches: match_map.into_iter().map(|(k, v)| (k, v.text)).collect(),
      associated_comma: None,
      associated_comments: Vec::new(),
    });
  }
  if recursive {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
      if let (mut inner_matches, true) =
        get_all_matches_for_concrete_syntax(&child, code_str, meta, recursive, replace_node.clone())
      {
        matches.append(&mut inner_matches);
      }
    }
  }

  let is_empty = matches.is_empty();
  (matches, !is_empty)
}

/// `find_next_sibling` navigates the cursor through the tree to the next sibling.
/// If no sibling exists, it ascends the tree until it can move laterally or until it reaches the root.
///
/// # Arguments
///
/// * `cursor` - A mutable reference to a `TreeCursor` used to navigate the tree.
///
/// The function mutates the cursor position. If no further siblings exist, the cursor ends at the root.
fn find_next_sibling(cursor: &mut TreeCursor) -> bool {
  while !cursor.goto_next_sibling() {
    if !cursor.goto_parent() {
      return false;
    }
  }
  true
}

pub(crate) fn get_matched_for_sequential_siblings(
  cursor: &mut TreeCursor, source_code: &[u8], meta: &ConcreteSyntax,
) -> (HashMap<String, CapturedNode>, bool, Option<Range>) {
  let the_node = cursor.node();
  let mut child_incr = 0;
  cursor.goto_first_child();
  while {
    let mut tmp_cursor = cursor.clone();
    let (mapping, mut matched, indx) =
      get_matches_for_node(&mut tmp_cursor, source_code, meta, true, the_node);

    if matched {
      let mut last_node = the_node.child(the_node.child_count() - 1);
      if let Some(last_node_index) = indx {
        last_node = the_node.child(last_node_index);
        matched = matched && (last_node_index != child_incr) // Makes no sense to match just 1 sibling
      }
      let range = Range::from_siblings(cursor.node().range(), last_node.unwrap().range());
      return (mapping, matched, Some(range));
    }

    child_incr += 1;
    cursor.goto_next_sibling()
  } {}

  (HashMap::new(), false, None)
}

/// This function performs the actual matching of the ConcreteSyntax pattern against a syntax tree
/// node. The matching is done in the following way:
///
/// - If the ConcreteSyntax is empty and all the nodes have been visited, then we found a match!
///
/// - If the ConcreteSyntax starts with `:[variable]`, the function tries to match the variable
///   against all possible AST nodes starting at the current's cursor position (i.e., the node itself,
///   its first child, the child of the first child, and so on.)
///   If it succeeds, it advances the ConcreteSyntax by the length of the matched
///   AST node and calls itself recursively to try to match the rest of the ConcreteSyntax.
///
/// - If the ConcreteSyntax doesn't start with `:[variable]`, the function checks if the node is a leaf
///   (i.e., has no children). If it is, and its text starts with the metasyyntax, we match the text,
///   and advance to the next immediate node (i.e., it's sibling or it's parent's sibling). If does not
///   match we cannot match the meta syntax template.
///
/// - If the ConcreteSyntax doesn't start with `:[variable]` and the node is not a leaf, the function
///   moves the cursor to the first child of the node and calls itself recursively to try to match
///   the ConcreteSyntax.
pub(crate) fn get_matches_for_node(
  cursor: &mut TreeCursor, source_code: &[u8], meta: &ConcreteSyntax, should_match: bool,
  the_node: Node,
) -> (HashMap<String, CapturedNode>, bool, Option<usize>) {
  let match_template = meta.0.as_str();

  if match_template.is_empty() {
    let index = the_node
      .children(&mut the_node.walk())
      .enumerate()
      .find_map(|(i, child)| {
        if child == cursor.node() {
          Some(i - 1) // Last matched node
        } else {
          None
        }
      });

    let result = HashMap::new();
    let matched = !should_match || index.is_some();
    return (result, matched, index);
  }

  let mut node = cursor.node();

  // Skip comment nodes always
  while node.kind().contains("comment") && cursor.goto_next_sibling() {
    node = cursor.node();
  }

  // In case the template starts with :[var_name], we try match
  if let Some(caps) = RE_VAR.captures(match_template) {
    let var_name = &caps["var_name"];
    let meta_adv_len = caps[0].len();
    let meta_advanced = ConcreteSyntax(
      match_template[meta_adv_len..]
        .to_string()
        .trim_start()
        .to_string(),
    );

    // If we need to match a variable `:[var]`, we can match it against the next node or any of it's
    // first children. We need to try all possibilities.
    loop {
      let mut tmp_cursor = cursor.clone();
      let current_node = cursor.node();
      let current_node_code = current_node.utf8_text(source_code).unwrap();

      let mut should_match = find_next_sibling(&mut tmp_cursor);

      let first = current_node;
      let mut is_final_sibling = false;
      let mut siblings_code = current_node_code.to_string();
      loop {
        let mut walkable_cursor = tmp_cursor.clone();
        if let (mut recursive_matches, true, last_node) = get_matches_for_node(
          &mut walkable_cursor,
          source_code,
          &meta_advanced,
          should_match,
          the_node,
        ) {
          // If we already matched this variable, we need to make sure that the match is the same. Otherwise, we were unsuccessful.
          // No other way of unrolling exists.
          if recursive_matches.contains_key(var_name)
            && recursive_matches[var_name].text.trim() != current_node_code.trim()
          {
            return (HashMap::new(), false, None);
          }
          recursive_matches.insert(
            var_name.to_string(),
            CapturedNode {
              range: Range::from_siblings(first.range(), current_node.range()),
              text: siblings_code,
            },
          );
          return (recursive_matches, true, last_node);
        }
        siblings_code.push_str(tmp_cursor.node().utf8_text(source_code).unwrap());
        if is_final_sibling {
          break;
        }
        is_final_sibling = !tmp_cursor.goto_next_sibling();
        if is_final_sibling {
          should_match = find_next_sibling(&mut tmp_cursor);
        }
      }

      if !cursor.goto_first_child() {
        break;
      }
    }
  } else if node.child_count() == 0 {
    let code = node.utf8_text(source_code).unwrap().trim();
    if match_template.starts_with(code) && !code.is_empty() {
      let advance_by = code.len();
      // Can only advance if there is still enough chars to consume
      if advance_by > match_template.len() {
        return (HashMap::new(), false, None);
      }
      let meta_substring = ConcreteSyntax(
        match_template[advance_by..]
          .to_string()
          .trim_start()
          .to_owned(),
      );
      let should_match = find_next_sibling(cursor);
      return get_matches_for_node(cursor, source_code, &meta_substring, should_match, the_node);
    }
  } else {
    cursor.goto_first_child();
    return get_matches_for_node(cursor, source_code, meta, true, the_node);
  }
  (HashMap::new(), false, None)
}

fn get_code_from_range(start_byte: usize, end_byte: usize, source_code: &[u8]) -> String {
  let text_slice = &source_code[start_byte..end_byte];
  String::from_utf8_lossy(text_slice).to_string()
}

#[cfg(test)]
#[path = "unit_tests/concrete_syntax_test.rs"]
mod concrete_syntax_test;
