/*
Copyright (c) 2022 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Defines the traits containing with utility functions that interface with tree-sitter.

use crate::utilities::MapOfVec;
use itertools::Itertools;
use std::collections::HashMap;
use tree_sitter::{Language, Node, Query, QueryCursor, Range};

extern "C" {
  fn tree_sitter_java() -> Language;
}

pub trait TreeSitterHelpers {
  /// Gets the tree-sitter language model.
  fn get_language(&self) -> Language;
  /// Compiles query string to `tree_sitter::Query`
  fn create_query(&self, language: Language) -> Query;
}

impl TreeSitterHelpers for String {
  fn create_query(&self, language: Language) -> Query {
    if let Ok(q) = Query::new(language, self.as_str()) {
      return q;
    }
    panic!("Could not parse the query : {}", self);
  }

  fn get_language(&self) -> Language {
    unsafe {
      match self.as_str() {
        "java" => tree_sitter_java(),
        _ => panic!("Language not supported"),
      }
    }
  }
}

#[rustfmt::skip]
pub trait PiranhaRuleMatcher {

    /// Applies the query upon `self`, and gets the first match
    /// # Arguments
    /// * `source_code` - the corresponding source code string for the node.
    /// * `query` - the query to be applied
    /// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
    ///
    /// # Returns
    /// The range of the match in the source code and the corresponding mapping from tags to code snippets.
    fn get_all_matches_for_query(&self, source_code: String, query: &Query, recursive: bool, replace_node_tag: Option<String>) -> Vec<(Range, HashMap<String, String>)>;

    /// Applies the query upon `self`, and gets all the matches
    /// # Arguments
    /// * `source_code` - the corresponding source code string for the node.
    /// * `query` - the query to be applied
    /// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
    ///
    /// # Returns
    /// A vector of `tuples` containing the range of the matches in the source code and the corresponding mapping for the tags (to code snippets).
    /// By default it returns the range of the outermost node for each query match.
    /// If `replace_node` is provided in the rule, it returns the range of the node corresponding to that tag.
    fn get_match_for_query(&self, source_code: &str, query: &Query, recursive: bool) -> Option<(Range, HashMap<String, String>)>;
}

impl PiranhaRuleMatcher for Node<'_> {
  fn get_match_for_query(
    &self, source_code: &str, query: &Query, recursive: bool,
  ) -> Option<(Range, HashMap<String, String>)> {
    self
      .get_all_matches_for_query(source_code.to_string(), query, recursive, None)
      .first().cloned()
  }

  fn get_all_matches_for_query(
    &self, source_code: String, query: &Query, recursive: bool, replace_node: Option<String>,
  ) -> Vec<(Range, HashMap<String, String>)> {
    let mut cursor = QueryCursor::new();
    let node_as_str = |n: &Node| n.utf8_text(source_code.as_bytes()).unwrap();

    // Match the query to the node get list of QueryMatch instances.
    // a QueryMatch is like a Map<tag, Node>
    let query_matches = cursor.matches(query, *self, source_code.as_bytes());

    // Since a node can be a part of multiple QueryMatch instances,
    // we group the query match instances based on the range of the outermost node they matched.
    let mut query_matches_by_node_range = HashMap::new();
    for query_match in query_matches {
      // The first capture in any query match is it's outermost tag.
      // Ensure the outermost s-expression for is tree-sitter query is tagged.
      if let Some(captured_node) = query_match.captures.first() {
        query_matches_by_node_range.collect(
          captured_node.node.range(),
          query_match.captures.iter().cloned().collect_vec(),
        );
      }
    }

    let tag_names_by_index: HashMap<usize, &String> =
      query.capture_names().iter().enumerate().collect();

    // In the below code, we get the code snippet corresponding to each tag for each QueryMatch.
    // It could happen that we have multiple occurrences of the same tag (in queries
    // that use the quantifier operator (*/+)). Therefore for each query match, we have to group (join) the codes snippets
    // corresponding to the same tag.
    let mut output = vec![];
    for (captured_node_range, query_matches) in query_matches_by_node_range {
      // This ensures that each query pattern in rule.query matches the same node.
      if query_matches.len() != query.pattern_count() {
        continue;
      }

      // Check if the range of the self (node), and the range of outermost node captured by the query are equal.
      let range_matches_self = self.start_byte() == captured_node_range.start_byte
        && self.end_byte() == captured_node_range.end_byte;

      // If `recursive` it allows matches to the subtree of self (Node)
      // Else it ensure that the query perfectly matches the node (`self`).
      if recursive || range_matches_self {
        let mut code_snippet_by_tag: HashMap<String, String> = HashMap::new();
        let mut replace_node_range = captured_node_range;

        // Iterate over each tag name in the query
        for tag_name in query.capture_names().iter() {
          // Iterate over each query match for this range of code snippet
          for captures in query_matches.clone() {
            // Iterate over each capture
            for capture in captures {
              if tag_names_by_index[&(capture.index as usize)].eq(tag_name) {
                // In some queries, the `rule.query` matches a larger node, while the rewrite rule replaces the a sub-AST with a new pattern
                // For instance: cleanup_riles/java/rules:remove_unnecessary_nested_block (here the outermost tag is @block while the
                // replace_node is @nested.block)
                // If parameter `replace_node` is provided we group the captures by this replace node and not the
                // outermost node captured by the query.
                let code_snippet = node_as_str(&capture.node);
                if let Some(sp) = &replace_node {
                  if tag_name.eq(sp) {
                    replace_node_range = capture.node.range();
                  }
                }
                // Join code snippets corresponding to the corresponding to the same tag with `\n`.
                // This scenario occurs when we use the `*` or the `+` quantifier in the tree-sitter query
                // Look at - cleanup_riles/java/rules:remove_unnecessary_nested_block
                code_snippet_by_tag
                  .entry(tag_name.clone())
                  .and_modify(|x| x.push_str(format!("\n{}", code_snippet).as_str()))
                  .or_insert_with(|| code_snippet.to_string());
              }
            }
          }
          // If tag name did not match a code snippet, add an empty string.
          code_snippet_by_tag
            .entry(tag_name.clone())
            .or_insert(String::new());
        }
        output.push((replace_node_range, code_snippet_by_tag));
      }
    }
    // This sorts the matches from bottom to top
    output.sort_by(|a, b| a.0.start_byte.cmp(&b.0.start_byte));
    output.reverse();
    output
  }
}
