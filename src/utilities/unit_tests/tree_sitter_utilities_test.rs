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
use std::collections::HashMap;

use tree_sitter::{Query, Range};

use crate::{
  models::{capture_group_patterns::CGPattern, default_configs::{ERB, JAVA, RUBY}, language::PiranhaLanguage},
  utilities::{tree_sitter_utilities::get_all_matches_for_query, Instantiate},
};

#[test]
fn test_get_all_matches_for_query_positive() {
  let source_code = r#"
      class Test {
        void foobar(Experiment exp) {
          if (exp.isFlagTreated(SOME_FLAG)){
            //Do Something
          }
          // Do something else
          
          if (abc && exp.isFlagTreated(SOME_FLAG)){
            // Do this too!
          }
        }
      }
    "#;
  let language = PiranhaLanguage::from(JAVA);
  let query = Query::new(
    *language.language(),
    r#"((
        (method_invocation 
          name : (_) @name
          arguments: ((argument_list 
                          ([
                            (field_access field: (_)@argument)
                            (_) @argument
                           ])) )
              
       ) @method_invocation
      )
      (#eq? @name "isFlagTreated")
      (#eq? @argument "SOME_FLAG")
      )"#,
  )
  .unwrap();

  let mut parser = PiranhaLanguage::from(JAVA).parser();
  let ast = parser
    .parse(source_code, None)
    .expect("Could not parse code");
  let node = ast.root_node();

  let matches = get_all_matches_for_query(
    &node,
    source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
    None,
  );
  assert_eq!(matches.len(), 2);
}

fn extract_ranges(node: tree_sitter::Node, source: &str, content_ranges: &mut Vec<Range>, ruby_ranges: &mut Vec<Range>) {
  let node_type = node.kind();
  // Check if this is a content node (HTML) or code node (Ruby)
  if node_type == "content" {
    content_ranges.push(Range {
      start_byte: node.start_byte(),
      end_byte: node.end_byte(),
      start_point: node.start_position(),
      end_point: node.end_position(),
    });
  } else if node_type == "directive" {
    // For code nodes, we want the actual Ruby code inside
    if let Some(code_child) = node.named_child(0) {
      ruby_ranges.push(Range {
        start_byte: code_child.start_byte(),
        end_byte: code_child.end_byte(),
        start_point: code_child.start_position(),
        end_point: code_child.end_position(),
      });
    }
  }

  // Recursively process children
  for i in 0..node.child_count() {
    if let Some(child) = node.child(i) {
      extract_ranges(child, source, content_ranges, ruby_ranges);
    }
  }
}

#[test]
fn test_erb_parser() {
  use std::fs;
  let erb_source_code = fs::read_to_string("/Users/bsunder/uber/piranha/erb_test/sample.html.erb").expect("Failed to read ERB file");
  let language = PiranhaLanguage::from(ERB);
  let ruby_language = PiranhaLanguage::from(RUBY);
  let mut erb_parser = language.parser();
  let erb_tree = erb_parser.parse(&erb_source_code, None).unwrap();
  let erb_root = erb_tree.root_node();
  println!("ERB AST:");
  println!("{}", erb_root.to_sexp());

  let mut content_ranges = Vec::new();
  let mut ruby_ranges = Vec::new();
  extract_ranges(erb_root, &erb_source_code, &mut content_ranges, &mut ruby_ranges);

  let query = Query::new(
    *ruby_language.language(),
    r#"
      (((call
        method: (identifier) @flag_name
      )@flag_exp
      )
      (#eq? @flag_name "msp?")
      )    
    "#,
  )
  .unwrap();

  let mut ruby_parser = PiranhaLanguage::from(RUBY).parser();

  let mut ruby_snippet: String = String::new();
  for (i, range) in ruby_ranges.iter().enumerate() {
    let text = &erb_source_code[range.start_byte..range.end_byte];
    ruby_snippet.push_str(text);  
    println!("  Ruby {}: {:?} -> {:?}", i, range, text);
  }
  println!("Ruby Snippet: {}", ruby_snippet);  
  erb_parser.set_language(tree_sitter_ruby::language()).unwrap();
  erb_parser.set_included_ranges(&ruby_ranges).unwrap();

  println!("erb_root: {}", erb_root.to_sexp());

  let matches = get_all_matches_for_query(
    &erb_root,
    erb_source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
    None,
  );
  assert_eq!(matches.len(), 2);
}

#[test]
fn test_get_all_matches_for_query_negative() {
  let source_code = r#"
      class Test {
        void foobar(Experiment exp) {
          if (exp.isFlagTreated(SOME_OTHER)){
            //Do Something
          }
          // Do something else
          
          if (abc && exp.isFlagTreated(SOME_OTHER)){
            // Do this too!
          }
        }
      }
    "#;
  let language = PiranhaLanguage::from(JAVA);
  let query = Query::new(
    *language.language(),
    r#"((
        (method_invocation 
          name : (_) @name
          arguments: ((argument_list 
                          ([
                            (field_access field: (_)@argument)
                            (_) @argument
                           ])) )
              
       ) @method_invocation
      )
      (#eq? @name "isFlagTreated")
      (#eq? @argument "SOME_FLAG")
      )"#,
  )
  .unwrap();

  let mut parser = PiranhaLanguage::from(JAVA).parser();
  let ast = parser
    .parse(source_code, None)
    .expect("Could not parse code");
  let node = ast.root_node();

  let matches = get_all_matches_for_query(
    &node,
    source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
    None,
  );
  assert!(matches.is_empty());
}

#[test]
fn test_instantiate() {
  let substitutions = HashMap::from([
    ("variable_name".to_string(), "isFlagTreated".to_string()),
    ("init".to_string(), "true".to_string()),
  ]);
  assert_eq!(
    CGPattern("@variable_name foo bar @init".to_string())
      .instantiate(&substitutions)
      .0,
    "isFlagTreated foo bar true"
  )
}

#[test]
fn test_instantiate_cs() {
  let substitutions = HashMap::from([
    ("variable_name".to_string(), "isFlagTreated".to_string()),
    ("init".to_string(), "true".to_string()),
  ]);
  assert_eq!(
    CGPattern(":[variable_name] foo bar :[init]".to_string())
      .instantiate(&substitutions)
      .0,
    "isFlagTreated foo bar true"
  )
}
