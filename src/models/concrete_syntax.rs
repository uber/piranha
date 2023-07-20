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
use std::collections::{HashMap, VecDeque};
use tree_sitter::{Node, Parser, TreeCursor};

use crate::models::capture_group_patterns::MetaSyntax;
use crate::models::language::PiranhaLanguage;
use crate::models::matches::Match;
use crate::utilities::tree_sitter_utilities::get_all_matches_for_query;

pub(crate) fn get_all_matches_for_metasyntax(
  node: &Node, source_code: String, meta: &MetaSyntax, recursive: bool,
  _replace_node: Option<String>,
) -> (Vec<Match>, bool) {
  let vec_deque = vec![node.clone()].into_iter().collect::<VecDeque<_>>();
  get_all_matches_for_metasyntax_aux(vec_deque, source_code, meta, _replace_node)
}

pub(crate) fn get_all_matches_for_metasyntax_aux(
  mut nodes: VecDeque<Node>, source_code: String, meta: &MetaSyntax, _replace_node: Option<String>,
) -> (Vec<Match>, bool) {
  let mut matches: Vec<Match> = Vec::new();
  let re_var = Regex::new(r"^:\[(?P<var_name>\w+)\]").unwrap();
  let syntx = meta.0.trim_start();

  // If there is no node to process, and the template is empty, we succeed
  if nodes.is_empty() {
    return (matches, syntx.is_empty());
  }

  let node = nodes.pop_front().unwrap();
  let code = node.utf8_text(source_code.as_bytes()).unwrap();

  // In case the template starts with :[var_name], we try match
  if let Some(caps) = re_var.captures(syntx) {
    let var_name = &caps["var_name"];
    let meta_adv_len = caps[0].len();
    let meta_adv = MetaSyntax(syntx[meta_adv_len..].to_string().trim_start().to_owned());

    let (mut inner_matches, success) =
      get_all_matches_for_metasyntax_aux(nodes.clone(), source_code.clone(), &meta_adv, None);

    if success {
      let mut match_map = HashMap::new();
      match_map.insert(
        var_name.to_string(),
        node.utf8_text(source_code.as_bytes()).unwrap().to_string(),
      );

      inner_matches.push(Match {
        matched_string: var_name.to_string(),
        range: Range::from(node.range()),
        matches: match_map,
        associated_comma: None,
        associated_comments: Vec::new(),
      });
      return (inner_matches, true);
    }

    // Unroll and try again
    let mut cursor = node.walk();
    let mut children = node.children(&mut cursor).collect();
    let mut new_nodes = VecDeque::new();
    new_nodes.append(&mut children);
    new_nodes.append(&mut nodes);

    return get_all_matches_for_metasyntax_aux(new_nodes, source_code.clone(), meta, None);
  } else if syntx.starts_with(code.trim()) {
    let advance_by = code.len();
    // create new template after advancement
    let meta_substring = MetaSyntax(syntx[advance_by..].to_string().trim_start().to_owned());
    return get_all_matches_for_metasyntax_aux(nodes, source_code.clone(), &meta_substring, None);
  } else {
    // for each child of metasyntax, recursive call this function
    let mut cursor = node.walk();
    let mut children = node.children(&mut cursor).collect();

    // append children into the beginning of the list
    let mut new_nodes = VecDeque::new();
    new_nodes.append(&mut children);
    new_nodes.append(&mut nodes);

    return get_all_matches_for_metasyntax_aux(new_nodes, source_code.clone(), meta, None);
  }

  (matches, false)
}
