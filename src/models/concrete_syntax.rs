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
    match_sequential_siblings(&mut node.walk(), code_str, meta)
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

fn find_next_sibling(cursor: &mut TreeCursor) -> bool {
  while !cursor.goto_next_sibling() {
    if !cursor.goto_parent() {
      return false;
    }
  }
  true
}

/// Attempts to match a given ConcreteSyntax template against a sequence of sibling nodes
/// in an Abstract Syntax Tree (AST).
///
/// # Arguments
///
/// * `cursor` - A mutable reference to the TreeCursor, which is used to navigate the AST.
/// * `source_code` - A slice of bytes representing the source code being analyzed.
/// * `meta` - A reference to the ConcreteSyntax template used for matching.
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
/// 2. Use `get_matches_for_node` to attempt matching the template against the subtree starting at each sibling.
/// 3. If a match is found, determine the range of matched nodes and return the match mapping, status, and range.
/// 4. If no match is found, return an empty mapping, false status, and None for range.
pub(crate) fn match_sequential_siblings(
  cursor: &mut TreeCursor, source_code: &[u8], meta: &ConcreteSyntax,
) -> (HashMap<String, CapturedNode>, bool, Option<Range>) {
  let parent_node = cursor.node();
  let mut child_incr = 0;

  if cursor.goto_first_child() {
    // Iterate through siblings to find a match
    while {
      let mut tmp_cursor = cursor.clone();
      let (mapping, indx) = get_matches_for_subsequence_of_nodes(
        &mut tmp_cursor,
        source_code,
        meta,
        true,
        &parent_node,
      );

      if let Some(last_node_index) = indx {
        // Determine the last matched node. Remember, we are matching subsequences of children [n ... k]
        let last_node = parent_node.child(last_node_index);
        let range = Range::from_siblings(cursor.node().range(), last_node.unwrap().range());
        return (
          mapping,
          last_node_index != child_incr || parent_node.child_count() == 1,
          Some(range),
        );
      }

      child_incr += 1;
      cursor.goto_next_sibling()
    } {}
  }

  // Return no match if none found
  (HashMap::new(), false, None)
}

/// This function performs the actual matching of the ConcreteSyntax pattern against a syntax tree
/// node. The matching is done in the following way:
///
///
/// - If the ConcreteSyntax is empty and all the nodes have been visited, then we found a match!
///   Otherwise, if we ran out of nodes to match, and the template is not empty, then we failed
///
/// - If the ConcreteSyntax starts with `:[variable]`, the function tries to match the variable
///   against all possible AST nodes starting at the current's cursor position (i.e., the node itself
///   and all of its siblings, its first child and respective siblings, the child of the first child, and so on.)
///   If it succeeds, it advances the ConcreteSyntax by the length of the matched sequence of
///   AST nodes, and calls itself recursively to try to match the rest of the ConcreteSyntax.
///
/// - If the ConcreteSyntax doesn't start with `:[variable]`, the function checks if the node is a leaf
///   (i.e., has no children). If it is, and its text starts with the metasyyntax, we match the text,
///   and advance to the next immediate node (i.e., it's sibling or it's parent's sibling). If does not
///   match we cannot match the meta syntax template.
///
/// - If the ConcreteSyntax doesn't start with `:[variable]` and the node is not a leaf, the function
///   moves the cursor to the first child of the node and calls itself recursively to try to match
///   the ConcreteSyntax.
pub(crate) fn get_matches_for_subsequence_of_nodes(
  cursor: &mut TreeCursor, source_code: &[u8], meta: &ConcreteSyntax, nodes_left_to_match: bool,
  top_node: &Node,
) -> (HashMap<String, CapturedNode>, Option<usize>) {
  let match_template = meta.0.as_str();

  if match_template.is_empty() {
    if !nodes_left_to_match {
      return (HashMap::new(), Some(top_node.child_count() - 1));
    }

    let index = find_last_matched_node(cursor, top_node);
    return (HashMap::new(), index);
  } else if !nodes_left_to_match {
    return (HashMap::new(), None);
  }

  let mut node = cursor.node();
  // Skip comment nodes always
  while node.kind().contains("comment") && cursor.goto_next_sibling() {
    node = cursor.node();
  }

  if let Some(caps) = RE_VAR.captures(match_template) {
    // If template starts with a template variable
    handle_template_variable_matching(cursor, source_code, top_node, caps, match_template)
  } else if node.child_count() == 0 {
    // If the current node if a leaf
    return handle_leaf_node(cursor, source_code, match_template, top_node);
  } else {
    // If the current node is an intermediate node
    cursor.goto_first_child();
    return get_matches_for_subsequence_of_nodes(cursor, source_code, meta, true, top_node);
  }
}

/// This function is a bit convoluted because I have failed to simplify it further.
/// It basically matches a template variable against a subsequence of nodes.
/// Comments inline explaining what's going on during the matching process
fn handle_template_variable_matching(
  cursor: &mut TreeCursor, source_code: &[u8], top_node: &Node, caps: regex::Captures,
  match_template: &str,
) -> (HashMap<String, CapturedNode>, Option<usize>) {
  let var_name = &caps["var_name"];
  let meta_adv_len = caps[0].len();
  let meta_advanced = ConcreteSyntax(
    match_template[meta_adv_len..]
      .to_string()
      .trim_start()
      .to_string(),
  );

  // Matching :[var] against a sequence of nodes [first_node, last_node]
  loop {
    let first_node = cursor.node();
    let mut last_node = first_node;

    let mut tmp_cursor = cursor.clone();
    let mut should_match = find_next_sibling(&mut tmp_cursor); // Advance the cursor to match the rest of the template
    let mut is_final_sibling = false;
    loop {
      let mut walkable_cursor = tmp_cursor.clone();
      if let (mut recursive_matches, Some(last_matched_node_idx)) =
        get_matches_for_subsequence_of_nodes(
          &mut walkable_cursor,
          source_code,
          &meta_advanced,
          should_match,
          top_node,
        )
      {
        // Check if the variable was already matched and if the match is the same, otherwise return unsuccessful
        let matched_code = get_code_from_range(
          first_node.range().start_byte,
          last_node.range().end_byte,
          source_code,
        );

        if recursive_matches.contains_key(var_name)
          && recursive_matches[var_name].text.trim() != matched_code.trim()
        {
          return (HashMap::new(), None);
        }

        recursive_matches.insert(
          var_name.to_string(),
          CapturedNode {
            range: Range::from_siblings(first_node.range(), last_node.range()),
            text: matched_code,
          },
        );
        return (recursive_matches, Some(last_matched_node_idx));
      }

      // Append an extra node to match with :[var]. Remember we had advanced tmp_cursor before,
      // therefore we cannot advance it again, otherwise we would skip nodes.
      last_node = tmp_cursor.node();
      if is_final_sibling {
        break;
      }

      // This is used for the final iteration. We need to determine if there are any other nodes
      // left to match, to inform our next recursive call. We do this by calling find_next_sibling
      // to move the cursor to the parent and find the next sibling at another level
      // since at this level we already matched everything
      is_final_sibling = !tmp_cursor.goto_next_sibling();
      if is_final_sibling {
        should_match = find_next_sibling(&mut tmp_cursor);
      }
    }

    // Move one level down, to attempt to match the template variable :[var] against smaller nodes.
    if !cursor.goto_first_child() {
      break;
    }
  }
  (HashMap::new(), None)
}

fn handle_leaf_node(
  cursor: &mut TreeCursor, source_code: &[u8], match_template: &str, top_node: &Node,
) -> (HashMap<String, CapturedNode>, Option<usize>) {
  let code = cursor.node().utf8_text(source_code).unwrap().trim();
  if match_template.starts_with(code) && !code.is_empty() {
    let advance_by = code.len();
    // Can only advance if there is still enough chars to consume
    if advance_by > match_template.len() {
      return (HashMap::new(), None);
    }
    let meta_substring = ConcreteSyntax(
      match_template[advance_by..]
        .to_string()
        .trim_start()
        .to_owned(),
    );
    let should_match = find_next_sibling(cursor);
    return get_matches_for_subsequence_of_nodes(
      cursor,
      source_code,
      &meta_substring,
      should_match,
      top_node,
    );
  }
  (HashMap::new(), None)
}

fn find_last_matched_node(cursor: &mut TreeCursor, parent_node: &Node) -> Option<usize> {
  parent_node
    .children(&mut parent_node.walk())
    .enumerate()
    .find_map(|(i, child)| {
      if child == cursor.node() {
        Some(i - 1)
      } else {
        None
      }
    })
}

fn get_code_from_range(start_byte: usize, end_byte: usize, source_code: &[u8]) -> String {
  let text_slice = &source_code[start_byte..end_byte];
  String::from_utf8_lossy(text_slice).to_string()
}

#[cfg(test)]
#[path = "unit_tests/concrete_syntax_test.rs"]
mod concrete_syntax_test;
