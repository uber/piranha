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
use crate::{
  filter,
  models::{
    default_configs::{JAVA, RUBY, UNUSED_CODE_PATH},
    filter::Filter,
    language::PiranhaLanguage,
    matches::{Point, Range},
    piranha_arguments::PiranhaArgumentsBuilder,
    rule_graph::{PARENT, PARENT_ITERATIVE},
  },
  utilities::eq_without_whitespace,
};

use super::InstantiatedRule;
use {
  crate::models::{rule_store::RuleStore, source_code_unit::SourceCodeUnit},
  std::collections::HashMap,
  std::path::PathBuf,
};

/// Tests whether a valid rule can be correctly instantiated given valid substitutions.
#[test]
fn test_rule_try_instantiate_positive() {
  let rule = piranha_rule! {
      name= "test",
      query= "(
        ((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc)
        (#eq? @a.lhs \"@variable_name\")
      )",
      replace_node = "abc",
      replace = "",
      holes = ["variable_name"]
  };

  let substitutions: HashMap<String, String> = HashMap::from([
    (String::from("variable_name"), String::from("foobar")),
    (String::from("@a.lhs"), String::from("something")), // Should not substitute, since it `a.lhs` is not in `rule.holes`
  ]);
  let instantiated_rule = InstantiatedRule::new(&rule, &substitutions);
  assert!(eq_without_whitespace(
    instantiated_rule.query().pattern().as_str(),
    "(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"foobar\"))"
  ))
}

#[test]
fn test_rule_try_instantiate_positive_cs() {
  let rule = piranha_rule! {
      name= "test",
      query= "cs :[variable_name] = :[value]",
      replace_node = "*",
      replace = "",
      holes = ["variable_name"]
  };

  let substitutions: HashMap<String, String> =
    HashMap::from([(String::from("variable_name"), String::from("foobar"))]);
  let instantiated_rule = InstantiatedRule::new(&rule, &substitutions);
  assert!(eq_without_whitespace(
    instantiated_rule.query().pattern().as_str(),
    "cs foobar = :[value]"
  ))
}

/// Tests whether a valid rule is *not* instantiated given invalid substitutions.
#[test]
#[should_panic]
fn test_rule_try_instantiate_negative() {
  let rule = piranha_rule! {
    name= "test",
    query= "(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"@variable_name\"))",
    replace_node = "abc",
    replace = "",
    holes = ["variable_name"]
  };
  let substitutions: HashMap<String, String> = HashMap::from([
    (String::from("@a.lhs"), String::from("something")), // Should not substitute, since it `a.lhs` is not in `rule.holes`
  ]);
  let _ = InstantiatedRule::new(&rule, &substitutions);
}

/// Positive tests for `rule.get_edit` method for given rule and input source code.
#[test]
fn test_get_edit_positive_recursive() {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
    ((local_variable_declaration
      declarator: (variable_declarator
          name: (_) @variable_name
          value: [(true) (false)] @init)) @variable_declaration)
     )",
    replace_node = "variable_declaration",
    replace = "",
    filters =[
        filter! {
          enclosing_node= "(method_declaration) @md",
          not_contains = [
            "(
              ((assignment_expression
                left: (_) @a.lhs
                right: (_) @a.rhs) @assignment)
              (#eq? @a.lhs \"@variable_name\")
              (#not-eq? @a.rhs \"@init\")
            )",
          ]
        }
    ]
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
          public void foobar(){
            boolean isFlagTreated = true;
            isFlagTreated = true;
            if (isFlagTreated) {
              // Do something;
            }
          }
        }";

  let mut rule_store = RuleStore::default();

  let args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .build();
  let mut parser = args.language().parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &args,
  );
  let node = source_code_unit.root_node();
  let matches = source_code_unit.get_matches(&rule, &mut rule_store, node, true);
  assert!(!matches.is_empty());

  let edit = source_code_unit.get_edit(&rule, &mut rule_store, node, true);
  assert!(edit.is_some());
}

#[test]
fn test_ruby_rule_1() {
  let _rule = piranha_rule! {
  name="test",
  query ="
    (
      (
        (call
            arguments: (argument_list
              (pair
                key: (_)
                value: (simple_symbol) @extracted_flag_name
              )
            )
        )@call
      )
      (#eq? @extracted_flag_name @flag_name)
    )
    ",
  replace_node = "extracted_flag_name",
  replace = "-> { true }",
  holes = ["flag_name"]
  };
  let source_code = "
    def method_name
      do_something if false
    end
  ";

  let args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .allow_dirty_ast(true)
    .language(PiranhaLanguage::from(RUBY))
    .build();
  let substitutions: HashMap<String, String> = HashMap::from([(
    String::from("flag_name"),
    String::from(":esm_enhancements_enabled?"),
  )]);
  let rule = InstantiatedRule::new(&_rule, &substitutions);
  let mut rule_store = RuleStore::new(&args);
  let mut parser = args.language().parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &args,
  );
  let node = source_code_unit.root_node();
  let matches = source_code_unit.get_matches(&rule, &mut rule_store, node, true);
  println!("{:?}", matches.iter().count());
  assert!(matches.is_empty());
}
/// Negative tests for `rule.get_edit` method for given rule and input source code.
#[test]
fn test_get_edit_negative_recursive() {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
      ((local_variable_declaration
        declarator: (variable_declarator
          name: (_) @variable_name
          value: [(true) (false)] @init)) @variable_declaration)
      )",
    replace_node = "variable_declaration",
    replace = "",
    filters =  [filter! {
          enclosing_node= "(method_declaration) @md",
          not_contains = [
            "(
              ((assignment_expression
                left: (_) @a.lhs
                right: (_) @a.rhs) @assignment)
              (#eq? @a.lhs \"@variable_name\")
              (#not-eq? @a.rhs \"@init\")
            )",
          ]
        }
      ]
  };
  let source_code = "class Test {
          public void foobar(){
            boolean isFlagTreated = true;
            isFlagTreated = false;
            if (isFlagTreated) {
              // Do something;
            }
          }
        }";

  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let mut rule_store = RuleStore::default();

  let args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .build();
  let mut parser = args.language().parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &args,
  );
  let node = source_code_unit.root_node();
  let matches = source_code_unit.get_matches(&rule, &mut rule_store, node, true);
  assert!(matches.is_empty());
  let edit = source_code_unit.get_edit(&rule, &mut rule_store, node, true);
  assert!(edit.is_none());
}

/// Positive tests for `rule.get_edit_for_context` method for given rule and input source code.
#[test]
fn test_get_edit_for_context_positive() {
  let _rule = piranha_rule! {
    name = "test",
    query = "(
      (binary_expression
          left : (_)* @lhs
          operator:\"&&\"
          right: [(true) (parenthesized_expression (true))]
      )
  @binary_expression)",
  replace_node = "binary_expression",
  replace = ""
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());

  let source_code = "class A {
          boolean f = something && true;
        }";

  let mut rule_store = RuleStore::default();
  let args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .build();
  let mut parser = args.language().parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &args,
  );
  let range = Range {
    start_byte: 41_usize,
    end_byte: 44_usize,
    ..Default::default()
  };

  let next_rules = HashMap::from([
    (String::from(PARENT), vec![rule.clone()]),
    (String::from(PARENT_ITERATIVE), vec![rule]),
  ]);
  let edit = source_code_unit.get_edit_for_ancestors(&range, &mut rule_store, &next_rules);
  assert!(edit.is_some());
}

/// Negative tests for `rule.get_edit_for_context` method for given rule and input source code.
#[test]
fn test_get_edit_for_context_negative() {
  let _rule = piranha_rule! {
    name = "test",
    query = "(
      (binary_expression
          left : (_)* @lhs
          operator:\"&&\"
          right: [(true) (parenthesized_expression (true))]
      )
  @binary_expression)",
  replace_node = "binary_expression",
  replace = ""
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class A {
          boolean f = true;
          boolean x = something && true;
        }";

  let mut rule_store = RuleStore::default();

  let args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .build();
  let mut parser = args.language().parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &args,
  );
  let range = Range {
    start_byte: 29_usize,
    end_byte: 33_usize,
    ..Default::default()
  };
  let next_rules = HashMap::from([
    (String::from(PARENT), vec![rule.clone()]),
    (String::from(PARENT_ITERATIVE), vec![rule]),
  ]);
  let edit = source_code_unit.get_edit_for_ancestors(&range, &mut rule_store, &next_rules);
  assert!(edit.is_none());
}

// Tests for not_enclosing_node
fn run_test_satisfies_filters_not_enclosing_node(
  filter: Filter, // Replace with the filter to test
  assertion: fn(bool) -> bool,
) {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          )) @variable_declaration)
      )",
    replace_node= "variable_declaration",
    replace= "",
    filters= [filter,]
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
      public void foobar(){
        if (isFlagTreated) {
          int testNumber = 0;
        }
       }
      }";

  let mut rule_store = RuleStore::default();
  let java = PiranhaLanguage::from(JAVA);
  let mut parser = java.parser();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );

  let start = Point { row: 3, column: 10 };
  let end = Point { row: 3, column: 29 };
  let node = &source_code_unit
    .root_node()
    .descendant_for_point_range(start.into(), end.into())
    .unwrap();

  let map: HashMap<String, String> = HashMap::new();
  assert!(assertion(source_code_unit.is_satisfied(
    *node,
    &rule,
    &map,
    &mut rule_store,
  )));
}

#[test]
fn test_satisfies_filter_not_enclosing_node_positive() {
  run_test_satisfies_filters_not_enclosing_node(
    filter! {,
    not_enclosing_node = "(if_statement) @if_stmt"},
    |result| !result,
  );
}

#[test]
fn test_satisfies_filter_not_enclosing_node_negative() {
  run_test_satisfies_filters_not_enclosing_node(
    filter! {,
    not_enclosing_node = "(while_statement ) @while"},
    |result| result,
  );
}
