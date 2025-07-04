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

//! Defines the traits containing with utility functions that interface with tree-sitter.

use super::eq_without_whitespace;
use crate::{
  models::{
    edit::Edit,
    matches::{Match, Range},
  },
  utilities::MapOfVec,
};
use itertools::Itertools;
use log::debug;

use std::collections::HashMap;
use tree_sitter::{InputEdit, Node, Parser, Query, QueryCapture, QueryCursor};
use tree_sitter_traversal::{traverse, Order};

/// Applies the query upon the given `node`, and gets the first match
/// # Arguments
/// * `node` - the root node to apply the query upon
/// * `source_code` - the corresponding source code string for the node.
/// * `query` - the query to be applied
/// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
///
/// # Returns
/// The range of the match in the source code and the corresponding mapping from tags to code snippets.
pub(crate) fn get_all_matches_for_query(
  node: &Node, source_code: String, query: &Query, recursive: bool, replace_node: Option<String>,
  replace_node_idx: Option<u8>,
) -> Vec<Match> {
  let query_capture_groups = _get_query_capture_groups(node, &source_code, query);
  // In the below code, we get the code snippet corresponding to each tag for each QueryMatch.
  // It could happen that we have multiple occurrences of the same tag (in queries
  // that use the quantifier operator (*/+)). Therefore for each query match, we have to group (join) the codes snippets
  // corresponding to the same tag.
  let mut output = vec![];
  for (captured_node_range, query_matches) in query_capture_groups {
    // This ensures that each query pattern in rule.query matches the same node.
    if query_matches.len() != query.pattern_count() {
      continue;
    }

    // Check if the range of the self (node), and the range of outermost node captured by the query are equal.
    let range_matches_self = node.start_byte() == *captured_node_range.start_byte()
      && node.end_byte() == *captured_node_range.end_byte();

    // If `recursive` it allows matches to the subtree of self (Node)
    // Else it ensure that the query perfectly matches the node (`self`).
    if recursive || range_matches_self {
      let mut replace_node_range = captured_node_range;
      if let Some(replace_node_name) = &replace_node {
        if let Some(r) =
          get_range_for_replace_node(query, &query_matches, replace_node_name, replace_node_idx)
        {
          replace_node_range = r;
        } else {
          continue;
        }
      }

      let code_snippet_by_tag = accumulate_repeated_tags(query, query_matches, &source_code);

      output.push(Match {
        matched_string: source_code
          [*replace_node_range.start_byte()..*replace_node_range.end_byte()]
          .to_string(),
        range: replace_node_range,
        matches: code_snippet_by_tag,
        ..Default::default()
      });
    }
  }
  // This sorts the matches from bottom to top
  output.sort_by(|a, b| a.range().start_byte().cmp(b.range().start_byte()));
  output.reverse();
  output
}

/// Applies the query upon given `node`, and gets the first match
/// # Arguments
/// * `node` - the root node to apply the query upon
/// * `source_code` - the corresponding source code string for the node.
/// * `query` - the query to be applied
///
/// # Returns
/// List of matches (list of captures), grouped by the outermost tag of the query
fn _get_query_capture_groups<'a>(
  node: &'a Node<'a>, source_code: &'a str, query: &'a Query,
) -> HashMap<Range, Vec<Vec<QueryCapture<'a>>>> {
  let mut cursor = QueryCursor::new();

  // Match the query to the node get list of QueryMatch instances.
  // a QueryMatch is like a Map<tag, Node>
  let query_matches = cursor.matches(query, *node, source_code.as_bytes());

  // Since a node can be a part of multiple QueryMatch instances,
  // we group the query match instances based on the range of the outermost node they matched.
  let mut query_matches_by_node_range: HashMap<Range, Vec<Vec<QueryCapture>>> = HashMap::new();
  for query_match in query_matches {
    // The first capture in any query match is it's outermost tag.
    // Ensure the outermost s-expression for is tree-sitter query is tagged.
    if let Some(captured_node) = query_match.captures.first() {
      query_matches_by_node_range.collect(
        captured_node.node.range().into(),
        query_match.captures.iter().cloned().collect_vec(),
      );
    }
  }
  query_matches_by_node_range
}

// Join code snippets corresponding to the corresponding to the same tag with `\n`.
// This scenario occurs when we use the `*` or the `+` quantifier in the tree-sitter query
// Look at - cleanup_riles/java/rules:remove_unnecessary_nested_block
// If tag name did not match a code snippet, add an empty string.
// Returns the mapping between the tag and source code snippet (accumulated).
fn accumulate_repeated_tags(
  query: &Query, query_matches: Vec<Vec<tree_sitter::QueryCapture>>, source_code: &str,
) -> HashMap<String, String> {
  let mut code_snippet_by_tag: HashMap<String, String> = HashMap::new();
  let tag_names_by_index: HashMap<usize, &String> =
    query.capture_names().iter().enumerate().collect();
  // Iterate over each tag name in the query
  for tag_name in query.capture_names().iter() {
    // Iterate over each query match for this range of code snippet
    for captures in query_matches.clone() {
      // Iterate over each capture
      for capture in captures {
        if tag_names_by_index[&(capture.index as usize)].eq(tag_name) {
          let code_snippet = &capture.node.utf8_text(source_code.as_bytes()).unwrap();
          code_snippet_by_tag
            .entry(tag_name.clone())
            .and_modify(|x| x.push_str(format!("\n{code_snippet}").as_str()))
            .or_insert_with(|| code_snippet.to_string());
        }
      }
    }
    // If tag name did not match a code snippet, add an empty string.
    code_snippet_by_tag.entry(tag_name.clone()).or_default();
  }
  code_snippet_by_tag
}

// In some queries, the `rule.query` matches a larger node, while the rewrite rule replaces the a sub-AST with a new pattern
// For instance: cleanup_riles/java/rules:remove_unnecessary_nested_block (here the outermost tag is @block while the
// replace_node is @nested.block)
// If parameter `replace_node` is provided we group the captures by this replace node and not the
// outermost node captured by the query.
// This function gets the range of the ast corresponding to the `replace_node` tag of the query.
fn get_range_for_replace_node(
  query: &Query, query_matches: &[Vec<tree_sitter::QueryCapture>], replace_node_name: &String,
  replace_node_idx: Option<u8>,
) -> Option<Range> {
  let tag_names_by_index: HashMap<usize, &String> =
    query.capture_names().iter().enumerate().collect();
  // Iterate over each tag name in the query
  for tag_name in query.capture_names().iter() {
    // Iterate over each query match for this range of code snippet
    for captures in query_matches.iter().cloned() {
      // Iterate over each capture
      for capture in captures {
        if tag_names_by_index[&(capture.index as usize)].eq(tag_name)
          && tag_name.eq(replace_node_name)
        {
          if let Some(child_index) = replace_node_idx {
            let c_usize = child_index as usize;
            if c_usize >= capture.node.named_child_count() {
              return None;
            }
            return Some(capture.node.named_child(c_usize).unwrap().range().into());
          }

          return Some(capture.node.range().into());
        }
      }
    }
  }
  panic!(
    "Could not fetch range or node for replace_node {}. Context: {:?}",
    replace_node_name,
    query.capture_names()
  );
}

/// Replaces the given byte range (`replace_range`) with the `replacement`.
/// Returns tree-sitter's edit representation along with updated source code.
/// Note: This method does not update `self`.
pub(crate) fn get_tree_sitter_edit(code: String, edit: &Edit) -> (String, InputEdit) {
  // Log the edit
  let replace_range: Range = *edit.p_match().range();
  let replacement = edit.replacement_string();
  debug!("{edit}");
  // Create the new source code content by appropriately
  // replacing the range with the replacement string.
  let new_source_code = [
    &code[..*replace_range.start_byte()],
    replacement,
    &code[*replace_range.end_byte()..],
  ]
  .concat();

  let len_of_replacement = replacement.len();
  let old_source_code_bytes = code.as_bytes();
  let new_source_code_bytes = new_source_code.as_bytes();
  let start_byte = *replace_range.start_byte();
  let old_end_byte = *replace_range.end_byte();
  let new_end_byte = start_byte + len_of_replacement;
  (
    new_source_code.to_string(),
    // Tree-sitter edit
    InputEdit {
      start_byte,
      old_end_byte,
      new_end_byte,
      start_position: position_for_offset(old_source_code_bytes, start_byte),
      old_end_position: position_for_offset(old_source_code_bytes, old_end_byte),
      new_end_position: position_for_offset(new_source_code_bytes, new_end_byte),
    },
  )
}

// Finds the position (col and row number) for a given offset.
fn position_for_offset(input: &[u8], offset: usize) -> tree_sitter::Point {
  let mut result = tree_sitter::Point { row: 0, column: 0 };
  for c in &input[0..offset] {
    if *c as char == '\n' {
      result.row += 1;
      result.column = 0;
    } else {
      result.column += 1;
    }
  }
  result
}

// Creates the InputEdit as per the tree-sitter api documentation.
fn _get_tree_sitter_edit(
  replace_range: Range, len_of_replacement: usize, old_source_code_bytes: &[u8],
  new_source_code_bytes: &[u8],
) -> InputEdit {
  let start_byte = *replace_range.start_byte();
  let old_end_byte = *replace_range.end_byte();
  let new_end_byte = start_byte + len_of_replacement;
  InputEdit {
    start_byte,
    old_end_byte,
    new_end_byte,
    start_position: position_for_offset(old_source_code_bytes, start_byte),
    old_end_position: position_for_offset(old_source_code_bytes, old_end_byte),
    new_end_position: position_for_offset(new_source_code_bytes, new_end_byte),
  }
}

/// Get the smallest node within `self` that spans the given range.
pub(crate) fn get_node_for_range(root_node: Node, start_byte: usize, end_byte: usize) -> Node {
  root_node
    .descendant_for_byte_range(start_byte, end_byte)
    .unwrap()
}

fn get_non_str_eq_parent(node: Node, source_code: String) -> Option<Node> {
  if let Some(parent) = node.parent() {
    if !eq_without_whitespace(
      parent.utf8_text(source_code.as_bytes()).unwrap(),
      node.utf8_text(source_code.as_bytes()).unwrap(),
    ) {
      return Some(parent);
    } else {
      return get_non_str_eq_parent(parent, source_code);
    }
  }
  None
}

/// Returns the node, its parent, grand parent and great grand parent
pub(crate) fn get_context(prev_node: Node<'_>, source_code: String, count: u8) -> Vec<Node<'_>> {
  let mut output = Vec::new();
  if count > 0 {
    output.push(prev_node);
    if let Some(parent) = get_non_str_eq_parent(prev_node, source_code.to_string()) {
      output.extend(get_context(parent, source_code, count - 1));
    }
  }
  output
}

pub(crate) fn get_replace_range(input_edit: InputEdit) -> Range {
  Range {
    start_byte: input_edit.start_byte,
    end_byte: input_edit.new_end_byte,
    start_point: input_edit.start_position.into(),
    end_point: input_edit.new_end_position.into(),
  }
}

/// Returns the (tree-sitter) parser for the tree-sitter query DSL
pub(crate) fn get_ts_query_parser() -> Parser {
  let mut parser = Parser::new();
  parser
    .set_language(tree_sitter_query::language())
    .expect("Could not set the language for the parser.");
  parser
}

/// Returns the number of errors in the AST
pub(crate) fn number_of_errors(node: &Node) -> usize {
  traverse(node.walk(), Order::Post)
    .filter(|node| node.is_error() || node.is_missing())
    .count()
}

#[cfg(test)]
#[path = "unit_tests/tree_sitter_utilities_test.rs"]
mod tree_sitter_utilities_test;
