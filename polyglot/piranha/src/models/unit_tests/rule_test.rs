use {
  super::Rule,
  crate::{
    models::{constraint::Constraint, rule_store::RuleStore, source_code_unit::SourceCodeUnit},
    utilities::tree_sitter_utilities::get_parser,
  },
  std::collections::HashMap,
  std::path::PathBuf,
};

/// Tests whether a valid rule can be correctly instantiated given valid substitutions.
#[test]
fn test_rule_try_instantiate_positive() {
  let rule = Rule::new("test","(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"@variable_name\"))",
        "@abc", "",Some(vec![String::from("variable_name")]), None);
  let substitutions: HashMap<String, String> = HashMap::from([
    (String::from("variable_name"), String::from("foobar")),
    (String::from("@a.lhs"), String::from("something")), // Should not substitute, since it `a.lhs` is not in `rule.holes`
  ]);
  let instantiated_rule = rule.try_instantiate(&substitutions);
  assert!(instantiated_rule.is_ok());
  assert_eq!(
    instantiated_rule.ok().unwrap().query(),
    "(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"foobar\"))"
  )
}

/// Tests whether a valid rule can be is *not* instantiated given invalid substitutions.
#[test]
fn test_rule_try_instantiate_negative() {
  let rule = Rule::new("test","(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"@variable_name\"))",
        "abc", "",Some(vec![String::from("variable_name")]), None);
  let substitutions: HashMap<String, String> = HashMap::from([
    (String::from("@a.lhs"), String::from("something")), // Should not substitute, since it `a.lhs` is not in `rule.holes`
  ]);
  let instantiated_rule = rule.try_instantiate(&substitutions);
  assert!(instantiated_rule.is_err());
}

/// Positive tests for `rule.get_edit` method for given rule and input source code.
#[test]
fn test_get_edit_positive_recursive() {
  let rule = Rule::new("test", "(
                           ((local_variable_declaration
                                           declarator: (variable_declarator
                                                               name: (_) @variable_name
                                                               value: [(true) (false)] @init)) @variable_declaration)
                           )", "variable_declaration", "" ,None, 
                           Some(vec![
                          Constraint::new(String::from("(method_declaration) @md"),
                            vec![String::from("(
                              ((assignment_expression
                                              left: (_) @a.lhs
                                              right: (_) @a.rhs) @assignment)
                              (#eq? @a.lhs \"@variable_name\")
                              (#not-eq? @a.rhs \"@init\")
                            )")]),
                          ]));
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
  let node = source_code_unit.root_node();
  let matches = rule.get_matches(&source_code_unit, &mut rule_store, node, true);
  assert!(!matches.is_empty());

  let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
  assert!(edit.is_some());
}

/// Negative tests for `rule.get_edit` method for given rule and input source code.
#[test]
fn test_get_edit_negative_recursive() {
  let rule = Rule::new("test", "(
                           ((local_variable_declaration
                                           declarator: (variable_declarator
                                                               name: (_) @variable_name
                                                               value: [(true) (false)] @init)) @variable_declaration)
                           )", "variable_declaration", "" ,None, 
                           Some(vec![
                          Constraint::new(String::from("(method_declaration) @md"),
                            vec![String::from("(
                              ((assignment_expression
                                              left: (_) @a.lhs
                                              right: (_) @a.rhs) @assignment)
                              (#eq? @a.lhs \"@variable_name\")
                              (#not-eq? @a.rhs \"@init\")
                            )")]),
                          ]));
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
  let node = source_code_unit.root_node();
  let matches = rule.get_matches(&source_code_unit, &mut rule_store, node, true);
  assert!(matches.is_empty());
  let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
  assert!(edit.is_none());
}

/// Positive tests for `rule.get_edit_for_context` method for given rule and input source code.
#[test]
fn test_get_edit_for_context_positive() {
  let rule = Rule::new(
    "test",
    "(
          (binary_expression
              left : (_)* @lhs
              operator:\"&&\"
              right: [(true) (parenthesized_expression (true))]
          )
      @binary_expression)",
    "binary_expression",
    "",
    None,
    None,
  );

  let source_code = "class A {
          boolean f = something && true;
        }";

  let mut rule_store = RuleStore::dummy();

  let mut parser = get_parser(String::from("java"));

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
  );
  let edit = Rule::get_edit_for_context(
    &source_code_unit,
    41_usize,
    44_usize,
    &mut rule_store,
    &vec![rule],
  );
  // let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
  assert!(edit.is_some());
}

/// Negative tests for `rule.get_edit_for_context` method for given rule and input source code.
#[test]
fn test_get_edit_for_context_negative() {
  let rule = Rule::new(
    "test",
    "(
          (binary_expression
              left : (_)* @lhs
              operator:\"&&\"
              right: [(true) (parenthesized_expression (true))]
          )
      @binary_expression)",
    "binary_expression",
    "",
    None,
    None,
  );

  let source_code = "class A {
          boolean f = true;
          boolean x = something && true;
        }";

  let mut rule_store = RuleStore::dummy();

  let mut parser = get_parser(String::from("java"));

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
  );
  let edit = Rule::get_edit_for_context(
    &source_code_unit,
    29_usize,
    33_usize,
    &mut rule_store,
    &vec![rule],
  );
  // let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
  assert!(edit.is_none());
}
