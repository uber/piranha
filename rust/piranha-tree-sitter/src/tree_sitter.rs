use super::configs::Rule;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCapture, Range, Tree};

pub fn get_edit(
    source_code: &str,
    replace_range: Range,
    rewritten_snippet: &str,
) -> (String, InputEdit) {
    println!("{}", source_code.len());
    let new_source_code = [
        &source_code[..replace_range.start_byte],
        rewritten_snippet,
        &source_code[replace_range.end_byte..],
    ]
    .concat();
    let len_new_source_code_bytes = rewritten_snippet.as_bytes().len();
    let input = &source_code.as_bytes().to_vec();
    let edit = InputEdit {
        start_byte: replace_range.start_byte,
        old_end_byte: replace_range.end_byte,
        new_end_byte: replace_range.start_byte + len_new_source_code_bytes,
        start_position: position_for_offset(input, replace_range.start_byte),
        old_end_position: position_for_offset(input, replace_range.end_byte),
        new_end_position: position_for_offset(
            input,
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

pub fn parse_code(language: Language, source_code: &String) -> (Parser, Tree) {
    let mut parser = Parser::new();
    parser.set_language(language).expect("Could not set language");
    let tree = parser.parse(&source_code, None).expect("Could not parse code");
    (parser, tree)
}

pub fn get_language(language: &str) -> Option<Language> {
    match language {
        "Java" => Option::Some(tree_sitter_java::language()),
        _ => None
    }
}

pub fn get_replace_node<'a>(
    query: &Query,
    rule: &Rule,
    captures: &'a [QueryCapture],
    source_code_bytes: &[u8],
) -> Option<Node<'a>> {
    println!(
        "Capture names = {}",
        query.capture_names().to_vec().join(", ")
    );
    for c in captures {
        println!(
            "Captured node {:?} {:?}",
            c.node.utf8_text(source_code_bytes),
            c.index
        );
    }
    captures
        .iter()
        .filter(|c| {
            let capture_name = query
                .capture_names().get(c.index as usize)
                .expect("not found");
            return capture_name.eq(&rule.complete_capture_var);
        })
        .next()
        .map(|_n| _n.node)
        .clone()
}
