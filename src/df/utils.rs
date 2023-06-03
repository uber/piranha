use tree_sitter::{Node, Parser, Query};

use crate::models::rule::Rule;
use crate::utilities::tree_sitter_utilities::{
  get_all_matches_for_query, get_match_for_query, get_node_for_range,
};
use std::string::String;

pub fn get_tags_from_matcher(node: &Rule, find_in_predicates: bool) -> Vec<String> {
  let query_source_code = node.query().get_query();

  let tsq = tree_sitter_query::language();
  let mut parser = Parser::new();
  parser
    .set_language(tsq)
    .expect("Could not set the language for the parser.");

  let tree = parser.parse(query_source_code.clone(), None).unwrap();
  let node_match_query = Query::new(tsq, "(capture) @cap").unwrap();
  let matches = get_all_matches_for_query(
    &tree.root_node(),
    query_source_code.clone(),
    &node_match_query,
    true,
    None,
  );

  let query = Query::new(tsq, "(predicate) @pred").unwrap();
  let mut tags = vec![];
  for m in matches {
    let range = m.range();
    let matched_node = get_node_for_range(tree.root_node(), range.start_byte, range.end_byte);
    if find_in_predicates {
      if _check_enclosing_node(query_source_code.as_str(), matched_node, &query, &parser) {
        tags.push(m.matched_string().clone());
      }
    } else {
      if _check_not_enclosing_node(query_source_code.as_str(), matched_node, &query, &parser) {
        tags.push(m.matched_string().clone());
      }
    }
  }
  tags
}

/// Search for any ancestor of `node` (including itself) that matches `query_str`
fn _check_not_enclosing_node(
  source_code: &str, node: Node, query: &Query, _parser: &Parser,
) -> bool {
  let mut current_node = node;
  // This ensures that the below while loop considers the current node too when checking for filters.
  if current_node.child_count() > 0 {
    current_node = current_node.child(0).unwrap();
  }

  while let Some(parent) = current_node.parent() {
    if get_match_for_query(&parent, source_code, &query, false).is_some() {
      return false;
    }
    current_node = parent;
  }
  return true;
}

fn _check_enclosing_node(source_code: &str, node: Node, query: &Query, _parser: &Parser) -> bool {
  let mut current_node = node;
  // This ensures that the below while loop considers the current node too when checking for filters.
  if current_node.child_count() > 0 {
    current_node = current_node.child(0).unwrap();
  }

  while let Some(parent) = current_node.parent() {
    if get_match_for_query(&parent, source_code, &query, false).is_some() {
      return true;
    }
    current_node = parent;
  }
  return false;
}
