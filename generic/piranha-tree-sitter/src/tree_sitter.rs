use std::collections::HashMap;

use colored::Colorize;
use tree_sitter::{InputEdit, Language, Node, Point, Query, QueryCapture, Range};

extern "C" {
    fn tree_sitter_java() -> Language;
    fn tree_sitter_swift() -> Language;
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
) -> HashMap<String, String> {
    let mut tag_capture = HashMap::new();
    // let capture_names = &query.capture_names();
    for capture in captures {
        let name = query
            .capture_names()
            .get(capture.index as usize)
            .expect("Capture name not found!");
        let code_snippet = capture
            .node
            .utf8_text(source_code_bytes)
            .expect("Could not get source code for node");
        if !tag_capture.contains_key(name) {
            tag_capture.insert(String::from(name), String::from(code_snippet));
        } else {
            tag_capture
                .entry(String::from(name))
                .and_modify(|s| s.push_str(["\n", code_snippet].join("").as_str()));
        }
    }

    for cn in query.capture_names() {
        tag_capture.entry(String::from(cn)).or_default();
    }

    tag_capture
}

pub fn node_matches_range(n: Node, range: Range) -> bool {
    n.start_byte() == range.start_byte && n.end_byte() == range.end_byte
}

pub trait TreeSitterQuery {
    fn get_language(&self) -> Language;
    fn get_extension(&self) -> &'static str;
    fn substitute_parameterized_rule_holes(
        &self,
        substitutions: &HashMap<String, String>,
    ) -> String;
    fn substitute_rule_holes(&self, substitutions: &HashMap<String, String>) -> String;
    fn create_query(&self, language: Language) -> Query;
    fn to_rule_hole(&self) -> String;
    fn to_parameterized_rule_hole(&self) -> String;
}

impl TreeSitterQuery for String {
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
    fn substitute_parameterized_rule_holes(
        &self,
        substitutions: &HashMap<String, String>,
    ) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in substitutions {
            let key = tag.to_parameterized_rule_hole();
            output = output.replace(&key, substitute)
        }
        output
        // substitute_in_str(substitutions, self, &toParameterizedRuleHole)
    }

    fn substitute_rule_holes(&self, substitutions: &HashMap<String, String>) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in substitutions {
            let key = tag.to_rule_hole();
            output = output.replace(&key, substitute)
        }
        output
    }

    fn create_query(&self, language: Language) -> Query {
        if let Ok(q) = Query::new(language, self) {
            return q;
        }
        panic!("Could not parse the query : {}", self);
    }

    fn to_rule_hole(&self) -> String {
        format!("@{}", self)
    }

    fn to_parameterized_rule_hole(&self) -> String {
        format!("[@{}]", self)
    }
}
