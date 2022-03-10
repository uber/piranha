use colored::Colorize;
use tree_sitter::{InputEdit, Language, Node, Point, QueryCapture, Range};

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

    let replace_code =
        &source_code[replace_range.start_byte..replace_range.end_byte];

    println!(
        "{} at ({:?}) -\n {}",
        if rewritten_snippet.is_empty() {
            "Delete code"
        } else {
            "Update code"
        }
        .green(),
        replace_range,
        if !rewritten_snippet.is_empty() {
            format!(
                "{}\n to \n{}",
                replace_code.italic(),
                rewritten_snippet.italic()
            )
        } else {
            format!("{} ", replace_code.italic())
        }
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

// pub fn parse_code(language: Language, source_code: &String) -> (Parser, Tree) {
//     let mut parser = Parser::new();
//     parser
//         .set_language(language)
//         .expect("Could not set language");
//     let tree = parser
//         .parse(&source_code, None)
//         .expect("Could not parse code");
//     (parser, tree)
// }

pub fn get_language(language: &str) -> Language {
    unsafe {
        match language {
            "Java" => tree_sitter_java(),
            "Swift" => tree_sitter_swift(),
            _ => panic!("Language not supported"),
        }
    }
}

// Returns the captured node with the largest span.
pub fn get_node_captured_by_query<'a>(
    captures: &'a [QueryCapture],
) -> Result<Node<'a>, &'static str> {
    captures
        .iter()
        .map(|n| n.node.clone())
        .max_by(|c1, c2| c1.byte_range().len().cmp(&c2.byte_range().len()))
        .ok_or("Could not compute the captured node for this query.")
}
