use std::collections::HashMap;

use colored::Colorize;
use tree_sitter::{InputEdit, Language, Point, QueryCapture, Range, Query};

use crate::{utilities::substitute_in_str};

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



pub fn get_language(language: &str) -> Language {
    unsafe {
        match language {
            "Java" => tree_sitter_java(),
            "Swift" => tree_sitter_swift(),
            _ => panic!("Language not supported"),
        }
    }
}

pub fn group_by_tag_str<'a>(
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
        if !tag_capture.contains_key(name){
           tag_capture.insert(String::from(name), String::from(code_snippet));
        }else {
            tag_capture.entry(String::from(name))
            .and_modify(|s| s.push_str(["\n", code_snippet].join("").as_str()));    
        }
        
        
    }
    tag_capture
}

// fn append_at_symb(s: &String) -> String {
//     format!("@{}", s)
// }

// fn create_p_rule_hole(s: &String) -> String {
//     format!("[@{}]", s)
// }

pub trait TreeSitterQuery {
    fn substituteParameterizedRuleHoles(&self, substitutions: &HashMap<String, String>) -> String;
    fn substituteRuleHoles(&self,  substitutions: &HashMap<String, String>) -> String;
    fn createQuery(&self, language: Language) -> Query;
    fn toRuleHole(&self) -> String;
    fn toParameterizedRuleHole(&self) -> String;
}

impl TreeSitterQuery for String {
    fn substituteParameterizedRuleHoles(&self, substitutions: &HashMap<String, String>) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in substitutions {
            let key = tag.toParameterizedRuleHole();
            output = output.replace(&key, substitute)
        }
        output
        // substitute_in_str(substitutions, self, &toParameterizedRuleHole)
    }

    fn substituteRuleHoles(&self,  substitutions: &HashMap<String, String>) -> String {
        let mut output = String::from(self);
        for (tag, substitute) in substitutions {
            let key = tag.toRuleHole();
            output = output.replace(&key, substitute)
        }
        output
    }

    fn createQuery(&self, language: Language) -> Query {
        if let Ok(q)  = Query::new(language, self){
            return q;
        }
        panic!("Could not parse the query : {}", self);
    }

    fn toRuleHole(&self) -> String {
        format!("@{}", self)
    }

    fn toParameterizedRuleHole(&self) -> String {
        format!("[@{}]", self)
    }

    // fn substitute_in_str(
    //     substitutes: &HashMap<String, String>,
    //     value: &String,
    //     key_mapper: &dyn Fn(&String) -> String
    // ) -> String {
    //     let mut output = String::from(value);
    //     for (tag, substitute) in substitutes {
    //         let key = key_mapper(tag);
    //         output = output.replace(&key, substitute)
    //     }
    //     output
    // }
}

