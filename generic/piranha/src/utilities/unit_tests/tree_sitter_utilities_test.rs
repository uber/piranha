use std::{collections::HashMap, path::PathBuf};

use tree_sitter::Query;

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
    None,
    Some(vec![Constraint::new(
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

  let mut parser = get_parser(String::from("java"));

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
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
    None,
    Some(vec![Constraint::new(
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

  let mut parser = get_parser(String::from("java"));

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
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
    substitute_tags("@variable_name foo bar @init".to_string(), &substitutions),
    "isFlagTreated foo bar true"
  )
}
