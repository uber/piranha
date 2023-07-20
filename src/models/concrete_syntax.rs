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
  find_all_matches_for_metasyntax(vec_deque, source_code, meta, _replace_node)
}


pub(crate) fn find_all_matches_for_metasyntax(
  mut nodes: VecDeque<Node>, source_code: String, meta: &MetaSyntax, _replace_node: Option<String>,
) -> (Vec<Match>, bool) {
  let mut matches: Vec<Match> = Vec::new();
  let re_var = Regex::new(r"^:\[(?P<var_name>\w+)\]").unwrap();
  let syntx = meta.0.trim_start();

  // If there is no node to process, and the template is empty, we succeed
  if nodes.is_empty() {
    return (matches, syntx.is_empty());
  }

  let node = nodes.front().unwrap();
  let code = node.utf8_text(source_code.as_bytes()).unwrap();

  // In case the template starts with :[var_name], we try match the left most node
  if let Some(caps) = re_var.captures(syntx) {
    let var_name = &caps["var_name"];
    let meta_adv_len = caps[0].len();
    let meta_adv = MetaSyntax(syntx[meta_adv_len..].trim_start().to_owned());

    let (mut inner_matches, success) = find_all_matches_for_metasyntax(nodes.clone(), source_code.clone(), &meta_adv, None);

    if success {
      let mut match_map = HashMap::new();
      match_map.insert(var_name.to_string(), node.utf8_text(source_code.as_bytes()).unwrap().to_string());

      inner_matches.push(Match {
        matched_string: var_name.to_string(),
        range: Range::from(node.range()),
        matches: match_map,
        associated_comma: None,
        associated_comments: Vec::new(),
      });

      return (inner_matches, true);
    }

    // If not successful, unroll and try again
    let mut cursor = node.walk();
    nodes = node.children(&mut cursor).into_iter().chain(nodes.into_iter()).collect();
  } else if syntx.starts_with(code.trim()) {

    let advance_by = code.len();
    // create new template after advancement
    nodes.pop_front();
    let meta_substring = MetaSyntax(syntx[advance_by..].to_string().trim_start().to_owned());
    return find_all_matches_for_metasyntax(nodes, source_code.clone(), &meta_substring, None);
  } else {
    let mut cursor = node.walk();
    let mut children = node.children(&mut cursor).collect();

    nodes.pop_front();
    // append children into the beginning of the list
    let mut new_nodes = VecDeque::new();
    new_nodes.append(&mut children);
    new_nodes.append(&mut nodes);

    return find_all_matches_for_metasyntax(new_nodes, source_code.clone(), meta, None);
  }

  (matches, false)
}
