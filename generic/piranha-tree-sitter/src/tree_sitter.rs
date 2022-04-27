/*
Copyright (c) 2019 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Defines the traits containing with utility functions that interface with tree-sitter. 
use std::collections::HashMap;

use crate::utilities::MapOfVec;
use itertools::Itertools;
use tree_sitter::{Language, Node, Query, QueryCapture, QueryCursor, Range};

extern "C" {
    fn tree_sitter_java() -> Language;
}

pub trait TreeSitterHelpers {
    /// Gets the tree-sitter language model.
    fn get_language(&self) -> Language;
    /// Gets the file extension.
    fn get_extension(&self) -> &'static str;
    /// replaces the all the occurrences of keys (of `substitutions` map) in the string with its corresponding value.
    fn substitute_tags(&self, substitutions: &HashMap<String, String>) -> String;
    /// Compiles query string to tree_sitter::Query
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
                "Java" => tree_sitter_java(),
                _ => panic!("Language not supported"),
            }
        }
    }
    fn get_extension(&self) -> &'static str {
        match self.as_str() {
            "Java" => "java",
            _ => panic!("Language not supported"),
        }
    }

    fn substitute_tags(&self, substitutions: &HashMap<String, String>) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in substitutions {
            // Before replacing the key, it is transformed to a tree-sitter tag by adding `@` as prefix
            let key = format!("@{}", tag);
            output = output.replace(&key, &substitute)
        }
        output
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
    fn get_all_matches_for_query(&self, source_code: String, query: &Query, recursive: bool, specific_tag: Option<String>) -> Vec<(Range, HashMap<String, String>)>;

    /// Applies the query upon `self`, and gets all the matches
    /// # Arguments
    /// * `source_code` - the corresponding source code string for the node.
    /// * `query` - the query to be applied
    /// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
    ///
    /// # Returns
    /// A vector of tuples containing the range of the matches in the source code and the corresponding mapping for the tags (to code snippets).
    /// By default it returns the range of the outermost node for each query match.
    /// If `specific_tag` is provided, it returns the range of the node corresponding to it.
    fn get_match_for_query(&self, source_code: &String, query: &Query, recursive: bool) -> Option<(Range, HashMap<String, String>)>;
}

impl PiranhaRuleMatcher for Node<'_> {
    fn get_match_for_query(
        &self,
        source_code: &String,
        query: &Query,
        recursive: bool,
    ) -> Option<(Range, HashMap<String, String>)> {
        self.get_all_matches_for_query(source_code.to_string(), query, recursive, None)
            .first()
            .map(|x| x.clone())
    }

    fn get_all_matches_for_query(
        &self,
        source_code: String,
        query: &Query,
        recursive: bool,
        replace_node: Option<String>,
    ) -> Vec<(Range, HashMap<String, String>)> {
        let mut cursor = QueryCursor::new();
        let node_as_str = |n: &Node| n.utf8_text(source_code.as_bytes()).unwrap();

        let query_matches = cursor.matches(&query, self.clone(), source_code.as_bytes());

        let mut query_matches_by_node_range: HashMap<Range, Vec<Vec<QueryCapture>>> =
            HashMap::new();
        for query_match in query_matches {
            // The first capture in any query match is it's outermost tag.
            if let Some(captured_node) = query_match.captures.first() {
                query_matches_by_node_range.collect(
                    captured_node.node.range(),
                    query_match.captures.iter().cloned().collect_vec(),
                );
            }
        }

        let tag_names_by_index: HashMap<usize, &String> =
            query.capture_names().iter().enumerate().collect();

        let mut output = vec![];
        for (captured_node_range, query_matches) in query_matches_by_node_range {
            // Ensures that all patterns in the query match the same outermost node.
            if query_matches.len() != query.pattern_count() {
                continue;
            }

            let range_matches_self = self.start_byte() == captured_node_range.start_byte
                && self.end_byte() == captured_node_range.end_byte;

            if recursive || range_matches_self {

                let mut code_snippet_by_tag: HashMap<String, String> = HashMap::new();
                let mut replace_node_range = captured_node_range;

                for tag_name in query.capture_names().iter() {
                    for captures in &query_matches {
                        for capture in captures {
                            if tag_names_by_index[&(capture.index as usize)].eq(tag_name) {
                                let code_snippet = node_as_str(&capture.node);
                                if let Some(sp) = replace_node.as_ref() {
                                    if tag_name.eq(sp) {
                                        replace_node_range = capture.node.range();
                                    }
                                }
                                // Join code snippets corresponding to the same tag, with `\n`.
                                code_snippet_by_tag
                                    .entry(tag_name.clone())
                                    .and_modify(|x| {
                                        x.push_str(format!("\n{}", code_snippet).as_str())
                                    })
                                    .or_insert_with(|| code_snippet.to_string());
                            }
                        }
                    }
                    // If tag name did not match a code snippet, add an empty string.
                    code_snippet_by_tag
                        .entry(tag_name.clone())
                        .or_insert(String::new());
                }
                output.push((replace_node_range,code_snippet_by_tag));
            }
        }
        output
    }
}
