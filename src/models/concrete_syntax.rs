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
use derive_builder::Builder;
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;
use tree_sitter::{Node, TreeCursor};

use crate::models::capture_group_patterns::MetaSyntax;
use crate::models::matches::Match;

// Precompile the regex outside the function
lazy_static! {
  static ref RE_VAR: Regex = Regex::new(r"^:\[(?P<var_name>\w+)\]").unwrap();
}

pub(crate) fn get_all_matches_for_metasyntax(
  node: &Node, code_str: &[u8], meta: &MetaSyntax, recursive: bool,
) -> (Vec<Match>, bool) {
  let mut matches: Vec<Match> = Vec::new();

  if let (mut match_map, true) = get_matches_for_node(&mut node.walk(), code_str, meta) {
    match_map.insert(
      "*".to_string(),
      node.utf8_text(code_str).unwrap().to_string(),
    );
    matches.push(Match {
      matched_string: "*".to_string(),
      range: Range::from(node.range()),
      matches: match_map,
      associated_comma: None,
      associated_comments: Vec::new(),
    });
  }

  if recursive {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
      if let (mut inner_matches, true) =
        get_all_matches_for_metasyntax(&child, code_str, meta, recursive)
      {
        matches.append(&mut inner_matches);
      }
    }
  }

  let is_empty = matches.is_empty();
  (matches, !is_empty);
}

fn find_next_sibling(cursor: &mut TreeCursor) {
  while !cursor.goto_next_sibling() {
    if !cursor.goto_parent() {
      break;
    }
  }
}

/// This function performs the actual matching of the metasyntax pattern against a syntax tree
/// node. The matching is done in the following way:
///
/// - If the metasyntax is empty and all the nodes have been visited, then we found a match!
///
/// - If the metasyntax starts with `:[variable]`, the function tries to match the variable
///   against all possible AST nodes starting at the current's cursor position (i.e., the node itself,
///   its first child, the child of the first child, and so on.
///   If it succeeds, it advances the metasyntax by the length of the matched
///   AST node and calls itself recursively to try to match the rest of the metasyntax.
///
/// - If the metasyntax doesn't start with `:[variable]`, the function checks if the node is a leaf
///   (i.e., has no children). If it is, and its text starts with the metasyyntax, we match the text,
///   and advance to the next immediate node (i.e., it's sibling or it's parent's sibling). If does not
///   match we cannot match the meta syntax template.
///
/// - If the metasyntax doesn't start with `:[variable]` and the node is not a leaf, the function
///   moves the cursor to the first child of the node and calls itself recursively to try to match
///   the metasyntax.
pub(crate) fn get_matches_for_node(
  cursor: &mut TreeCursor, source_code: &[u8], meta: &MetaSyntax,
) -> (HashMap<String, String>, bool) {
  let match_template = meta.0.as_str();

  if match_template.is_empty() {
    return (
      HashMap::new(),
      !cursor.goto_next_sibling() && !cursor.goto_parent(),
    );
  }

  let node = cursor.node();
  // In case the template starts with :[var_name], we try match
  if let Some(caps) = RE_VAR.captures(match_template) {
    let var_name = &caps["var_name"];
    let meta_adv_len = caps[0].len();
    let meta_advanced = MetaSyntax(
      match_template[meta_adv_len..]
        .to_string()
        .trim_start()
        .to_string(),
    );

    // If we need to match a variable `:[var]`, we can match it against the next node or any of it's
    // first children. We need to try all possibilities.
    loop {
      let mut tmp_cursor = cursor.clone();
      let node = cursor.node();
      let node_code = node.utf8_text(source_code).unwrap();
      find_next_sibling(&mut tmp_cursor);

      if let (mut recursive_matches, true) =
        get_matches_for_node(&mut tmp_cursor, source_code, &meta_advanced)
      {
        recursive_matches.insert(var_name.to_string(), node_code.to_string());
        return (recursive_matches, true);
      }

      if !cursor.goto_first_child() {
        break;
      }
    }
  } else if node.child_count() == 0 {
    let code = node.utf8_text(source_code).unwrap();
    if match_template.starts_with(code.trim()) {
      let advance_by = code.len();
      let meta_substring = MetaSyntax(
        match_template[advance_by..]
          .to_string()
          .trim_start()
          .to_owned(),
      );
      find_next_sibling(cursor);
      return get_matches_for_node(cursor, source_code, &meta_substring);
    }
  } else {
    cursor.goto_first_child();
    return get_matches_for_node(cursor, source_code, meta);
  }
  (HashMap::new(), false);
}
