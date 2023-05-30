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

use tree_sitter::{Parser, Point};

use crate::models::filter::FilterBuilder;
use crate::utilities::tree_sitter_utilities::TSQuery;
use crate::{
  filter,
  models::{
    default_configs::{JAVA, UNUSED_CODE_PATH},
    filter::Filter,
    language::PiranhaLanguage,
    piranha_arguments::PiranhaArgumentsBuilder,
    rule::InstantiatedRule,
    rule_store::RuleStore,
  },
  piranha_rule,
  utilities::eq_without_whitespace,
};
use {
  super::SourceCodeUnit,
  crate::models::edit::Edit,
  std::{collections::HashMap, path::PathBuf},
  tree_sitter::Range,
};

impl SourceCodeUnit {
  pub(crate) fn default(content: &str, parser: &mut Parser, language_name: String) -> Self {
    SourceCodeUnit::new(
      parser,
      content.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
      &PiranhaArgumentsBuilder::default()
        .path_to_codebase("some/test/path/".to_string())
        .language(PiranhaLanguage::from(language_name.as_str()))
        .build(),
    )
  }
}

fn range(
  start_byte: usize, end_byte: usize, start_row: usize, start_column: usize, end_row: usize,
  end_column: usize,
) -> Range {
  Range {
    start_byte,
    end_byte,
    start_point: tree_sitter::Point {
      row: start_row,
      column: start_column,
    },
    end_point: tree_sitter::Point {
      row: end_row,
      column: end_column,
    },
  }
}

fn get_java_tree_sitter_language() -> PiranhaLanguage {
  PiranhaLanguage::from(JAVA)
}

/// Positive test of an edit being applied  given replacement range  and replacement string.
#[test]
fn test_apply_edit_positive() {
  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = true;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }";

  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();

  let mut source_code_unit =
    SourceCodeUnit::default(source_code, &mut parser, java.extension().to_string());

  let _ = source_code_unit.apply_edit(
    &Edit::delete_range(source_code, range(49, 78, 3, 9, 3, 38)),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("boolean isFlagTreated = true;", ""),
    source_code_unit.code()
  ));
}

/// Negative test of an edit being applied given invalid replacement range and replacement string.
#[test]
#[should_panic(expected = "byte index 1000 is out of bounds")]
fn test_apply_edit_negative() {
  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }";

  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let mut source_code_unit =
    SourceCodeUnit::default(source_code, &mut parser, java.extension().to_string());

  let _ = source_code_unit.apply_edit(
    &Edit::delete_range(source_code, range(1000, 2000, 0, 0, 0, 0)),
    &mut parser,
  );
}

/// Tests for contains, at_least, and at_most

fn run_test_satisfies_filters(
  filter: Filter, // Replace with the filter to test
  assertion: fn(bool) -> bool,
) -> bool {
  let _rule = piranha_rule! {
      name= "test",
      query= "(
            ((local_variable_declaration
                            declarator: (variable_declarator
                                            name: (_) @variable_name
                                            value: [(true)] @init)) @variable_declaration)
            )",
      filters= [filter,]
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
        public void foobar(){
            boolean isFlagTreated = true;
            if (isFlagTreated) {
              x = anotherFunction(isFlagTreated);
              y = anotherFunction();
              x.equals(y);
            }
        }
        }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  let satisfied = source_code_unit.is_satisfied(
    *node,
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string()),
    ]),
    &mut rule_store,
  );
  assert!(assertion(satisfied));
  satisfied
}

#[test]
fn test_satisfies_filters_contains_positive() {
  run_test_satisfies_filters(
    filter! {
        enclosing_node= "(method_declaration) @md",
        contains= "(
                    ((method_invocation
                        arguments: (argument_list (
                            (identifier) @id))) @method)
                    (#eq? @id \"@variable_name\")
                )"
    },
    |result| result,
  );
}

#[test]
fn test_satisfies_filters_bounds_positive() {
  run_test_satisfies_filters(
    filter! {
        enclosing_node= "(method_declaration) @md",
        contains= "(
                    ((method_invocation
                        arguments: (argument_list (
                            (identifier) @id))) @method)
                )",
        at_least = 2,
        at_most = 4
    },
    |result| result,
  );
}

#[test]
fn test_satisfies_filters_at_least_negative() {
  run_test_satisfies_filters(
    filter! {
        enclosing_node= "(method_declaration) @md",
        contains= "(
                    ((method_invocation
                        arguments: (argument_list (
                            (identifier) @id))) @method)
                    (#eq? @id \"@variable_name\")
                )",
        at_least = 2
    },
    |result| !result,
  );
}

#[test]
fn test_satisfies_filters_at_most_negative() {
  run_test_satisfies_filters(
    filter! {
        enclosing_node= "(method_declaration) @md",
        contains= "(
                    ((method_invocation) @method)
                )",
        at_most = 1
    },
    |result| !result,
  );
}

#[test]
fn test_satisfies_filters_at_most_0_negative() {
  let contains_0 = run_test_satisfies_filters(
    filter! {
        enclosing_node= "(method_declaration) @md",
        contains= "(
                    ((method_invocation name: (_) @name) @method)
                    (#eq? @name \"equals\")
                )",
        at_least = 0,
        at_most = 0
    },
    |result| !result,
  );
  let not_contains = run_test_satisfies_filters(
    filter! {
        enclosing_node= "(method_declaration) @md",
        not_contains= ["(
                    ((method_invocation name: (_) @name) @method)
                    (#eq? @name \"equals\")
                )",]
    },
    |result| !result,
  );
  assert_eq!(contains_0, not_contains);
}

/// Tests for not contains
#[test]
fn test_satisfies_filters_not_contains_positive() {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          value: [(true) (false)] @init)) @variable_declaration)
      )",
    replace_node= "variable_declaration",
    replace= "",
    filters= [filter!{
      enclosing_node= "(method_declaration) @md",
      not_contains= ["(
        ((assignment_expression
                        left: (_) @a.lhs
                        right: (_) @a.rhs) @assignment)
        (#eq? @a.lhs \"@variable_name\")
        (#not-eq? @a.rhs \"@init\")
      )",]
    }]

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
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  assert!(source_code_unit.is_satisfied(
    *node,
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string())
    ]),
    &mut rule_store,
  ));
}

#[test]
fn test_satisfies_filters_not_contains_negative() {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
      ((local_variable_declaration
          declarator: (variable_declarator
          name: (_) @variable_name
          value: [(true) (false)] @init)) @variable_declaration)
      )",
    replace_node= "variable_declaration",
    replace= "",
    filters= [filter!{
      enclosing_node= "(method_declaration) @md",
      not_contains= ["(
        ((assignment_expression
                        left: (_) @a.lhs
                        right: (_) @a.rhs) @assignment)
        (#eq? @a.lhs \"@variable_name\")
        (#not-eq? @a.rhs \"@init\")
      )",]
    }]
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
        // Do something;
        }
       }
      }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_arguments = &PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    piranha_arguments,
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  assert!(!source_code_unit.is_satisfied(
    *node,
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string())
    ]),
    &mut rule_store,
  ));
}

// Tests for contains without providing an enclosing node
fn run_test_satisfies_filters_without_enclosing(
  filter: Filter, // Replace with the filter to test
  assertion: fn(bool) -> bool,
) {
  let _rule = piranha_rule! {
      name= "test",
      query= "(
            (method_declaration
              name: (identifier) @method_name) @md
            )",
      filters= [filter,]
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
        public void foobar(){
            boolean isFlagTreated = true;
            if (isFlagTreated) {
              x = anotherFunction(isFlagTreated);
              y = foobar();
              y = foobar();
              x.equals(y);
            }
        }
        }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );

  let start = Point::new(1, 8);
  let end = Point::new(9, 9);
  let node = &source_code_unit
    .root_node()
    .descendant_for_point_range(start, end)
    .unwrap();

  assert!(assertion(source_code_unit.is_satisfied(
    *node,
    &rule,
    &HashMap::from([("method_name".to_string(), "foobar".to_string()),]),
    &mut rule_store,
  )));
}

#[test]
fn test_not_contains_no_enclosing_negative() {
  run_test_satisfies_filters_without_enclosing(
    filter! {,
    not_contains= ["(
                   (method_invocation
                      name: (identifier) @inv) @md
                   (#eq? @inv \"@method_name\")
                      )",]},
    |result| !result,
  );
}

// Tests for contains with enclosing
#[test]
fn test_contains_no_enclosing_positive() {
  run_test_satisfies_filters_without_enclosing(
    filter! {,
    contains= "(
                   (method_invocation
                      name: (identifier) @inv) @md
                   (#eq? @inv \"@method_name\")
                      )",
    at_least =2},
    |result| result,
  );
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
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );

  let start = Point::new(3, 10);
  let end = Point::new(3, 29);
  let node = &source_code_unit
    .root_node()
    .descendant_for_point_range(start, end)
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

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `at_least` or `at_most` is set, but `contains` is empty !!!"
)]
fn test_filter_bad_arg_at_least() {
  FilterBuilder::default().at_least(2).build();
}

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `at_least` or `at_most` is set, but `contains` is empty !!!"
)]
fn test_filter_bad_arg_at_most() {
  FilterBuilder::default().at_least(5).build();
}

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `contains` and `not_contains` cannot be set at the same time !!! Please use two filters instead."
)]
fn test_filter_bad_arguments_contains_not_contains() {
  FilterBuilder::default()
    .contains(TSQuery::new(String::from("(if_statement) @if_stmt")))
    .not_contains(vec![TSQuery::new(String::from("(for_statement) @for"))])
    .build();
}

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `at_least` should be less than or equal to `at_most` !!!"
)]
fn test_filter_bad_range() {
  FilterBuilder::default()
    .contains(TSQuery::new(String::from("(if_statement) @if_stmt")))
    .at_least(5)
    .at_most(4)
    .build();
}
