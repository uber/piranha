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

use std::any::Any;
use crate::models::matches::Range;
use derive_builder::Builder;
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use tree_sitter::{Node, Parser, TreeCursor};

use crate::models::capture_group_patterns::MetaSyntax;
use crate::models::language::PiranhaLanguage;
use crate::models::matches::Match;
use crate::utilities::tree_sitter_utilities::get_all_matches_for_query;

pub(crate) fn get_all_matches_for_metasyntax(
  node: &Node, source_code: String, meta: &MetaSyntax, recursive: bool, memo: &mut HashMap<(String, String), (Vec<Match>, bool)>
) -> (Vec<Match>, bool) {

  let mut matches: Vec<Match> = Vec::new();
  let mut cursor = node.walk();
  for child in node.children(&mut cursor) {

    if child.kind() == "method_invocation" { // FIXME
      if let (mut inner_matches, true) = get_matches_for_node(child.walk(), source_code.clone(), meta, false, memo) {
        matches.append(&mut inner_matches)
      }
    }

    if let (mut inner_matches, true) = get_all_matches_for_metasyntax(&child, source_code.clone(), meta, false, memo) {
      matches.append(&mut inner_matches)
    }
  }

  return (matches.clone(), !matches.is_empty());
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
  mut cursor: TreeCursor, source_code: String, meta: &MetaSyntax, recursive: bool, memo: &mut HashMap<(String, String), (Vec<Match>, bool)>
) -> (Vec<Match>, bool) {
  let mut matches: Vec<Match> = Vec::new();
  let re_var = Regex::new(r"^:\[(?P<var_name>\w+)\]").unwrap();
  let syntx = meta.0.trim_start();


  if syntx.is_empty() {
    return (matches, !cursor.goto_next_sibling() && !cursor.goto_parent());
  }

  let node = cursor.node();
  let code = node.utf8_text(source_code.as_bytes()).unwrap();
  let mut success = false;

  // In case the template starts with :[var_name], we try match
  if let Some(caps) = re_var.captures(syntx) {
    let var_name = &caps["var_name"];
    let meta_adv_len = caps[0].len();
    let meta_advanced = MetaSyntax(syntx[meta_adv_len..].to_string().trim_start().to_string());

    // If we need to match a variable `:[var]`, we can match it against the next node or any of it's
    // first children. We need to try all possibilities.
    loop {
      let mut tmp_cursor = cursor.clone();
      let node = cursor.node();
      let node_code = node.utf8_text(source_code.as_bytes()).unwrap();
      while !tmp_cursor.goto_next_sibling() {
        if !tmp_cursor.goto_parent() {
            break;
        }
      }
      println!("Matching {} with {}", node_code, syntx);

      if let (mut recursive_matches, true) =
          get_matches_for_node(tmp_cursor.clone(), source_code.clone(), &meta_advanced, false, memo)
      {
        let mut match_map = HashMap::new();
        match_map.insert(
          var_name.to_string(),
          node_code.to_string(),
        );

        recursive_matches.push(Match {
          matched_string: var_name.to_string(),
          range: Range::from(node.range()),
          matches: match_map,
          associated_comma: None,
          associated_comments: Vec::new(),
        });
        return (recursive_matches, true);
      }

      if !cursor.goto_first_child() {
        break;
      }
    }

    return (Vec::new(), false);

  } else if node.child_count() == 0 {
    println!("Matching {} with {}", code, syntx);
    if (syntx.starts_with(code.trim())) {
      let advance_by = code.len();
      let meta_substring = MetaSyntax(syntx[advance_by..].to_string().trim_start().to_owned());
      while (!cursor.goto_next_sibling()) {
        if !cursor.goto_parent() {
          break;
        }
      }
      return get_matches_for_node(
        cursor,
        source_code.clone(),
        &meta_substring,
        false,
        memo
      );
    }
    return (Vec::new(), false)

  } else {
    cursor.goto_first_child();
    return get_matches_for_node(cursor.clone(), source_code.clone(), meta, false, memo);
  }

}
