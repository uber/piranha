use std::collections::HashMap;

use colored::Colorize;
// use serde::{Serialize};
use crate::utilities::MapOfVec;
use itertools::Itertools;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use tree_sitter::QueryMatch;
use tree_sitter::{InputEdit, Language, Node, Point, Query, QueryCapture, QueryCursor, Range};

extern "C" {
    fn tree_sitter_java() -> Language;
    fn tree_sitter_swift() -> Language;
}

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

#[derive(Clone, Debug)]
pub struct TagMatches(HashMap<String, String>);

impl TagMatches {
    pub fn new(matches: HashMap<String, String>) -> Self {
        TagMatches(matches)
    }

    // pub fn new_vec(matches: Vec<HashMap<String, String>>) -> Self {
    //     let mut new_map = HashMap::new();
    //     for tag_matches in tag_matches_list {
    //         new_map.extend(tag_matches.0);
    //     }
    //     Self::new(new_map)
    // }

    pub fn new_list(tag_matches_list: Vec<TagMatches>) -> Self {
        let mut new_map = HashMap::new();
        for tag_matches in tag_matches_list {
            new_map.extend(tag_matches.0);
        }
        Self::new(new_map)
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

    // pub fn values(&self) -> Vec<String> {
    //     self.0.values().map(|x|x.to_string()).collect_vec()
    // }
}

pub fn get_edit(
    source_code: &str,
    replace_range: Range,
    rewritten_snippet: &str,
) -> (String, InputEdit) {
    let new_source_code = [
        &source_code[..replace_range.start_byte],
        rewritten_snippet,
        &source_code[replace_range.end_byte..],
    ]
    .concat();

    let replace_code = &source_code[replace_range.start_byte..replace_range.end_byte];
    #[rustfmt::skip]
    println!("{} at ({:?}) -\n {}", if rewritten_snippet.is_empty() { "Delete code" } else {"Update code" }.green(),
        ((&replace_range.start_point.row, &replace_range.start_point.column),
            (&replace_range.end_point.row, &replace_range.end_point.column)),
        if !rewritten_snippet.is_empty() {format!("{}\n to \n{}",replace_code.italic(),rewritten_snippet.italic())
        } else {format!("{} ", replace_code.italic())}
    );

    if replace_code.eq("true") && rewritten_snippet.eq("true"){
        panic!("Replace true with true?");
    }

    let len_new_source_code_bytes = rewritten_snippet.as_bytes().len();
    let byte_vec = &source_code.as_bytes().to_vec();
    let edit = InputEdit {
        start_byte: replace_range.start_byte,
        old_end_byte: replace_range.end_byte,
        new_end_byte: replace_range.start_byte + len_new_source_code_bytes,
        start_position: position_for_offset(byte_vec, replace_range.start_byte),
        old_end_position: position_for_offset(byte_vec, replace_range.end_byte),
        new_end_position: position_for_offset(
            byte_vec,
            replace_range.start_byte + len_new_source_code_bytes,
        ),
    };

    (new_source_code, edit)
}

fn position_for_offset(input: &Vec<u8>, offset: usize) -> Point {
    let mut result = Point { row: 0, column: 0 };
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

pub fn group_captures_by_tag<'a>(
    captures: &[QueryCapture],
    query: &'a Query,
    source_code_bytes: &'a [u8],
) -> TagMatches {
    let code_snippets = |idx: &u32| {
        String::from(
            &captures
                .iter()
                .filter(|x| x.index.eq(idx))
                .map(|cs| cs.node.utf8_text(source_code_bytes).unwrap())
                .join("\n"),
        )
    };
    TagMatches::new(
        query
            .capture_names()
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.clone(), code_snippets(&(idx as u32))))
            .collect(),
    )
}

pub trait TreeSitterHelpers {
    fn get_language(&self) -> Language;
    fn get_extension(&self) -> &'static str;
    fn substitute_tags(&self, substitutions: &TagMatches) -> String;
    fn to_rule_hole(&self) -> String;
}

impl TreeSitterHelpers for String {
    fn get_language(&self) -> Language {
        unsafe {
            match self.as_str() {
                "Java" => tree_sitter_java(),
                "Swift" => tree_sitter_swift(),
                _ => panic!("Language not supported"),
            }
        }
    }

    fn get_extension(&self) -> &'static str {
        match self.as_str() {
            "Java" => "java",
            "Swift" => "swift",
            _ => panic!("Language not supported"),
        }
    }

    fn substitute_tags(&self, substitutions: &TagMatches) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in &substitutions.0 {
            let key = tag.to_rule_hole();
            output = output.replace(&key, &substitute)
        }
        output
    }

    fn to_rule_hole(&self) -> String {
        format!("@{}", self)
    }
}

#[rustfmt::skip]
pub trait PiranhaRuleMatcher {
    fn get_matches_for_query(&self, source_code: String, query: &Query, recurssive: bool) -> Vec<(Range, TagMatches)>;
    fn get_match_for_query(&self, source_code: &String, query: &Query, recurssive: bool) -> Option<(Range, TagMatches)>;
    fn node_matches_range(&self, range: Range) -> bool;
    fn get_matches_for_query_specific_tag(&self, source_code: String, query: &Query, recurssive: bool, specific_tag: String) -> Vec<(Range, TagMatches)>;
}

impl PiranhaRuleMatcher for Node<'_> {
    fn get_match_for_query(
        &self,
        source_code: &String,
        query: &Query,
        recurssive: bool,
    ) -> Option<(Range, TagMatches)> {
        self.get_matches_for_query(source_code.to_string(), query, recurssive)
            .first()
            .map(|x| x.clone())
    }

    fn get_matches_for_query(
        &self,
        source_code: String,
        query: &Query,
        recurssive: bool,
    ) -> Vec<(Range, TagMatches)> {
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, self.clone(), source_code.as_bytes());

        let tag_name_index: HashMap<usize, &String> =
            query.capture_names().iter().enumerate().collect();

        let mut query_matches_by_node_range: HashMap<Range, Vec<QueryMatch>> = HashMap::new();
        for query_match in query_matches {
            if let Some(captured_node) = query_match.captures.first() {
                query_matches_by_node_range
                    .collect_as_counter(captured_node.node.range(), query_match);
            }
        }
        let mut output = vec![];
        for (captured_node_range, query_matches) in query_matches_by_node_range {
            if query_matches.len() != query.pattern_count() {
                continue;
            }

            if !recurssive && !self.node_matches_range(captured_node_range) {
                continue;
            }

            let mut capture_str_by_tag: HashMap<String, String> = HashMap::new();

            for tag_name in query.capture_names().iter() {
                for qm in &query_matches {
                    for capture in qm.captures {
                        if tag_name_index[&(capture.index as usize)].eq(tag_name) {
                            let node_str = capture.node.utf8_text(source_code.as_bytes()).unwrap();
                            capture_str_by_tag
                                .entry(tag_name.clone())
                                .and_modify(|x| x.push_str(format!("\n{}", node_str).as_str()))
                                .or_insert_with(|| String::from(node_str));
                        }
                    }
                }
                capture_str_by_tag.entry(tag_name.clone()).or_insert(String::new());
            }

            let tm = TagMatches::new(capture_str_by_tag);
            println!("{:?}", tm);
            output.push((captured_node_range, tm));
        }
        output
    }

    fn get_matches_for_query_specific_tag(
        &self,
        source_code: String,
        query: &Query,
        recurssive: bool,
        specific_tag: String,
    ) -> Vec<(Range, TagMatches)> {
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, self.clone(), source_code.as_bytes());

        let tag_name_index: HashMap<usize, &String> =
            query.capture_names().iter().enumerate().collect();

        println!("Number of patterns : {} ", query.pattern_count());

        let mut query_matches_by_node_range: HashMap<Range, Vec<QueryMatch>> = HashMap::new();
        for query_match in query_matches {
            if let Some(captured_node) = query_match.captures.first() {
                query_matches_by_node_range
                    .collect_as_counter(captured_node.node.range(), query_match);
            }
        }
        let mut output = vec![];
        for (captured_node_range, qms) in query_matches_by_node_range {
            println!("QMS {:?}", qms);
            if qms.len() != query.pattern_count() {
                continue;
            }
            if recurssive || self.node_matches_range(captured_node_range) {
                let mut capture_str_by_tag: HashMap<String, String> = HashMap::new();
                let mut replace_node_range = captured_node_range;
                println!("{:?}", query.capture_names());
                for tag_name in query.capture_names().iter() {
                    for qm in &qms {
                        println!("QM:");
                        for capture in qm.captures {
                            println!("{:?}", capture );
                            if tag_name_index[&(capture.index as usize)].eq(tag_name) {
                                let node_str = capture.node.utf8_text(source_code.as_bytes()).unwrap();
                                println!("{:?}", node_str);
                                if tag_name.eq(&specific_tag){
                                    replace_node_range = capture.node.range();
                                }
    
                                capture_str_by_tag
                                    .entry(tag_name.clone())
                                    .and_modify(|x| x.push_str(format!("\n{}", node_str).as_str()))
                                    .or_insert_with(|| String::from(node_str));
                                break;
                            }
                        }
                    }
                    capture_str_by_tag.entry(tag_name.clone()).or_insert(String::new());
                }
    
                let tm = TagMatches::new(capture_str_by_tag);
                println!("{:?}", tm);
                output.push((replace_node_range, tm));     
            }

        }
        output
    }

    fn node_matches_range(&self, range: Range) -> bool {
        self.start_byte() == range.start_byte && self.end_byte() == range.end_byte
    }
}
