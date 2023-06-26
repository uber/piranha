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

use tree_sitter::{Node, Parser, Query};

use crate::models::rule::Rule;
use crate::utilities::tree_sitter_utilities::{
  get_all_matches_for_query, get_match_for_query, get_node_for_range,
};
use regex::Regex;
use std::string::String;

/// Find defined tags in a query
pub fn get_tags_from_matcher(node: &Rule) -> Vec<String> {
  let query_source_code = node.query().get_query();

  let tsq = tree_sitter_query::language();
  let mut parser = Parser::new();
  parser
    .set_language(tsq)
    .expect("Could not set the language for the parser.");

  let tree = parser.parse(query_source_code.clone(), None).unwrap();

  let query = Query::new(tsq, "(predicate) @pred").unwrap();
  let node_match_query = Query::new(tsq, "(capture) @cap").unwrap();
  let matches = get_all_matches_for_query(
    &tree.root_node(),
    query_source_code.clone(),
    &node_match_query,
    true,
    None,
    None
  );
  let mut tags = vec![];
  for m in matches {
    let range = m.range();
    let matched_node = get_node_for_range(tree.root_node(), range.start_byte, range.end_byte);
    if _check_not_enclosing_node(query_source_code.as_str(), matched_node, &query, &parser) {
      tags.push(m.matched_string().clone());
    }
  }
  tags
}

/// Find all tags used in predicates
pub fn get_tags_usage_from_matcher(node: &Rule) -> Vec<String> {
  let query_source_code = node.query().get_query();

  let tsq = tree_sitter_query::language();
  let mut parser = Parser::new();
  parser
    .set_language(tsq)
    .expect("Could not set the language for the parser.");

  let tree = parser.parse(query_source_code.clone(), None).unwrap();

  // Individual queries for capture, identifier, and string
  let capture_query = Query::new(tsq, "(capture) @cap").unwrap();
  let identifier_query = Query::new(tsq, "(identifier) @id").unwrap();
  let string_query = Query::new(tsq, "(string) @str").unwrap();

  let mut tags = vec![];

  // Regular expression to match substrings starting with "@"
  // This is not the best way of doing it
  let re = Regex::new(r"@[\w.-]+").unwrap();

  // Function to process matches and extract tags
  let mut process_matches = |query: &Query, remove_quotations: bool| {
    let matches = get_all_matches_for_query(
      &tree.root_node(),
      query_source_code.clone(),
      query,
      true,
      None,
      None
    );
    for m in matches {
      let range = m.range();
      let matched_node = get_node_for_range(tree.root_node(), range.start_byte, range.end_byte);
      let query = Query::new(tsq, "(predicate) @pred").unwrap();
      if _check_enclosing_node(query_source_code.as_str(), matched_node, &query, &parser) {
        let mut tag = m.matched_string().clone();
        // Remove quotations if required
        if remove_quotations {
          tag = tag.replace('\"', "");
        }
        // Use regular expression to find substrings starting with "@"
        for cap in re.captures_iter(&tag) {
          tags.push(cap[0].to_string());
        }
      }
    }
  };

  // Process matches for each type
  process_matches(&capture_query, false);
  process_matches(&identifier_query, false);
  process_matches(&string_query, true);

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
    if get_match_for_query(&parent, source_code, query, false).is_some() {
      return false;
    }
    current_node = parent;
  }
  true
}

fn _check_enclosing_node(source_code: &str, node: Node, query: &Query, _parser: &Parser) -> bool {
  let mut current_node = node;
  // This ensures that the below while loop considers the current node too when checking for filters.
  if current_node.child_count() > 0 {
    current_node = current_node.child(0).unwrap();
  }

  while let Some(parent) = current_node.parent() {
    if get_match_for_query(&parent, source_code, query, false).is_some() {
      return true;
    }
    current_node = parent;
  }
  false
}
