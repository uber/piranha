use std::collections::HashMap;

// use serde::{Serialize};
use crate::utilities::MapOfVec;
use itertools::Itertools;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use tree_sitter::{Language, Node, Query, QueryCapture, QueryCursor, Range};

extern "C" {
    fn tree_sitter_java() -> Language;
    fn tree_sitter_swift() -> Language;
}

/// A new_type for for tree-sitter queries
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TSQuery(String);

impl TSQuery {
    pub fn from(s: String) -> Self {
        TSQuery(s)
    }

    pub fn create_query(&self, language: Language) -> Query {
        if let Ok(q) = Query::new(language, self.0.as_str()) {
            return q;
        }
        panic!("Could not parse the query : {:?}", self);
    }

    pub fn substitute_tags(&self, substitutions: &TagMatches) -> TSQuery {
        Self::from(self.0.substitute_tags(substitutions))
    }

    pub fn contains(&self, s: &String) -> bool {
        self.0.contains(s)
    }
}

/// A new_type for matches between tags and code snippets.
#[derive(Clone, Debug)]
pub struct TagMatches(HashMap<String, String>);

impl TagMatches {
    pub fn new(matches: HashMap<String, String>) -> Self {
        TagMatches(matches)
    }

    pub fn get(&self, s: &String) -> Option<&String> {
        self.0.get(s)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extend(&mut self, other_tag_matches: TagMatches) {
        self.0.extend(other_tag_matches.0)
    }
}

pub trait TreeSitterHelpers {
    fn get_language(&self) -> Language;
    fn get_extension(&self) -> &'static str;
    fn substitute_tags(&self, substitutions: &TagMatches) -> String;
    fn to_rule_hole(&self) -> String;
}

impl TreeSitterHelpers for String {
    /// Gets the tree-sitter language model.
    fn get_language(&self) -> Language {
        unsafe {
            match self.as_str() {
                "Java" => tree_sitter_java(),
                "Swift" => tree_sitter_swift(),
                _ => panic!("Language not supported"),
            }
        }
    }
    /// Gets the file extension.
    fn get_extension(&self) -> &'static str {
        match self.as_str() {
            "Java" => "java",
            "Swift" => "swift",
            _ => panic!("Language not supported"),
        }
    }

    /// replaces the keys in `substitutions` with value.
    /// Before replacing the key, it is transformed to a tree-sitter tag by appending `@`
    fn substitute_tags(&self, substitutions: &TagMatches) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in &substitutions.0 {
            let key = tag.to_rule_hole();
            output = output.replace(&key, &substitute)
        }
        output
    }

    /// appends `@`
    fn to_rule_hole(&self) -> String {
        format!("@{}", self)
    }
}

#[rustfmt::skip]
pub trait PiranhaRuleMatcher {
    fn get_all_matches_for_query(&self, source_code: String, query: &Query, recurssive: bool, specific_tag: Option<String>) -> Vec<(Range, TagMatches)>;
    fn get_match_for_query(&self, source_code: &String, query: &Query, recurssive: bool) -> Option<(Range, TagMatches)>;
    fn node_matches_range(&self, range: Range) -> bool;
}

/// Applies the query upon `self`, and gets the first match
/// # Arguments
/// * `source_code` - the corresponding source code string for the node.
/// * `query` - the query to be applied
/// * `recurssive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
///
/// # Returns
/// The range of the match in the source code and the corresponding mapping from tags to code snippets.
impl PiranhaRuleMatcher for Node<'_> {
    fn get_match_for_query(
        &self,
        source_code: &String,
        query: &Query,
        recurssive: bool,
    ) -> Option<(Range, TagMatches)> {
        self.get_all_matches_for_query(source_code.to_string(), query, recurssive, None)
            .first()
            .map(|x| x.clone())
    }

    /// Applies the query upon `self`, and gets all the matches
    /// # Arguments
    /// * `source_code` - the corresponding source code string for the node.
    /// * `query` - the query to be applied
    /// * `recurssive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
    ///
    /// # Returns
    /// A vector of tuples containing the range of the matches in the source code and the corresponding mapping from tags to code snippets.
    /// By default it returns the range of the outermost node for each query match. 
    /// If `specific_tag` is provided, it returns the range of the node corresponding to it.
    fn get_all_matches_for_query(
        &self,
        source_code: String,
        query: &Query,
        recurssive: bool,
        specific_tag: Option<String>,
    ) -> Vec<(Range, TagMatches)> {

        let mut cursor = QueryCursor::new();
        let node_as_str = |n:&Node| n.utf8_text(source_code.as_bytes()).unwrap();

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
        for (captured_node_range, qms) in query_matches_by_node_range {
            // Ensures that all patterns in the query match the same outermost node.
            if qms.len() != query.pattern_count() {
                continue;
            }

            if recurssive || self.node_matches_range(captured_node_range) {
                let mut capture_str_by_tag: HashMap<String, String> = HashMap::new();
                let mut replace_node_range = captured_node_range;

                for tag_name in query.capture_names().iter() {
                    for captures in &qms {
                        for capture in captures {
                            if tag_names_by_index[&(capture.index as usize)].eq(tag_name) {

                                let node_str = node_as_str(&capture.node);

                                if let Some(sp) = specific_tag.as_ref() {
                                    if tag_name.eq(sp) {
                                        replace_node_range = capture.node.range();
                                    }
                                }
                                // Concats code snippets with `\n` corresponding to tags with One or more quantifier (i.e. `*` por `+`)
                                capture_str_by_tag
                                    .entry(tag_name.clone())
                                    .and_modify(|x| x.push_str(format!("\n{}", node_str).as_str()))
                                    .or_insert_with(|| String::from(node_str));
                            }
                        }
                    }
                    // If tag name did not match a code snippet, add an empty string.
                    capture_str_by_tag
                        .entry(tag_name.clone())
                        .or_insert(String::new());
                }
                output.push((replace_node_range, TagMatches::new(capture_str_by_tag)));
            }
        }
        output
    }

    /// Checks if `self` lies in the exact `range`.
    fn node_matches_range(&self, range: Range) -> bool {
        self.start_byte() == range.start_byte && self.end_byte() == range.end_byte
    }
}
