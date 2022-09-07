
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
use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

use tree_sitter::Query;

use crate::models::piranha_arguments::{PiranhaArgumentsBuilder};

use {
  super::{get_parser, substitute_tags, PiranhaHelpers, TreeSitterHelpers},
  crate::models::{
    constraint::Constraint, rule::Rule, rule_store::RuleStore, source_code_unit::SourceCodeUnit,
  },
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
  let language_name = String::from("java");
  let query = Query::new(
    language_name.get_language(),
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

  let mut parser = get_parser(String::from("java"));
  let ast = parser
    .parse(&source_code, None)
    .expect("Could not parse code");
  let node = ast.root_node();

  let matches = node.get_all_matches_for_query(
    source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
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
  let language_name = String::from("java");
  let query = Query::new(
    language_name.get_language(),
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

  let mut parser = get_parser(String::from("java"));
  let ast = parser
    .parse(&source_code, None)
    .expect("Could not parse code");
  let node = ast.root_node();

  let matches = node.get_all_matches_for_query(
    source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
  );
  assert!(matches.is_empty());
}

#[test]
fn test_satisfies_constraints_positive() {
  let rule = Rule::new(
    "test",
    "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          value: [(true) (false)] @init)) @variable_declaration)
      )",
    "variable_declaration",
    "",
    HashSet::new(),
    HashSet::from([Constraint::new(
      String::from("(method_declaration) @md"),
      vec![String::from(
        "(
         ((assignment_expression
                         left: (_) @a.lhs
                         right: (_) @a.rhs) @assignment)
         (#eq? @a.lhs \"@variable_name\")
         (#not-eq? @a.rhs \"@init\")
       )",
      )],
    )]),
  );
  let source_code = "class Test {
      pub void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = true;
        if (isFlagTreated) {
        // Do something;
        }
       }
      }";

  let mut rule_store = RuleStore::dummy();
  let language_name = String::from("java");
  let mut parser = get_parser(language_name.to_string());
  let piranha_args = PiranhaArgumentsBuilder::default().language_name(language_name).build().unwrap();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  assert!(node.satisfies_constraint(
    source_code_unit.clone(),
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string())
    ]),
    &mut rule_store,
  ));
}

#[test]
fn test_satisfies_constraints_negative() {
  let rule = Rule::new(
    "test",
    "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          value: [(true) (false)] @init)) @variable_declaration)
      )",
    "variable_declaration",
    "",
    HashSet::new(),
    HashSet::from([Constraint::new(
      String::from("(method_declaration) @md"),
      vec![String::from(
        "(
         ((assignment_expression
                         left: (_) @a.lhs
                         right: (_) @a.rhs) @assignment)
         (#eq? @a.lhs \"@variable_name\")
         (#not-eq? @a.rhs \"@init\")
       )",
      )],
    )]),
  );
  let source_code = "class Test {
      pub void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
        // Do something;
        }
       }
      }";

  let mut rule_store = RuleStore::dummy();
  let language_name = String::from("java");
  let mut parser = get_parser(language_name.to_string());
  let piranha_arguments = &PiranhaArgumentsBuilder::default().language_name(language_name).build().unwrap();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    piranha_arguments
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  assert!(!node.satisfies_constraint(
    source_code_unit.clone(),
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string())
    ]),
    &mut rule_store,
  ));
}

#[test]
fn test_substitute_tags() {
  let substitutions = HashMap::from([
    ("variable_name".to_string(), "isFlagTreated".to_string()),
    ("init".to_string(), "true".to_string()),
  ]);
  assert_eq!(
    substitute_tags(
      "@variable_name foo bar @init".to_string(),
      &substitutions,
      false
    ),
    "isFlagTreated foo bar true"
  )
}
