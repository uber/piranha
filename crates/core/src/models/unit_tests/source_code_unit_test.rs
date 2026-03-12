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

use tree_sitter::Parser;

use crate::{
  filter,
  models::{
    default_configs::{JAVA, RUBY, SWIFT, UNUSED_CODE_PATH},
    filter::Filter,
    language::PiranhaLanguage,
    matches::{Point, Range},
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
};

impl SourceCodeUnit {
  pub(crate) fn default(content: &str, parser: &mut Parser, language_name: String) -> Self {
    SourceCodeUnit::new(
      parser,
      content.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
      &PiranhaArgumentsBuilder::default()
        .paths_to_codebase(vec!["some/test/path/".to_string()])
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
    start_point: Point {
      row: start_row,
      column: start_column,
    },
    end_point: Point {
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
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
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

#[test]
fn test_satisfies_filters_child_count() {
  let rule_positive = piranha_rule! {
    name= "test",
    query= "(
      (method_invocation 
          name: (_) @name
          arguments: (argument_list)@args) @mi 
      (#eq? @name \"someOtherFunction\")
      )",
    replace_node= "args",
    replace= "()",
    filters= [filter!{
      , child_count = 3
    }]
  };
  let rule_positive = InstantiatedRule::new(&rule_positive, &HashMap::new());

  let rule_neg = piranha_rule! {
    name= "test",
    query= "(
      (method_invocation 
          name: (_) @name
          arguments: (argument_list)@args) @mi 
      (#eq? @name \"someOtherFunction\")
      )",
    replace_node= "args",
    replace= "()",
    filters= [filter!{
      , child_count = 2
    }]
  };
  let rule_neg = InstantiatedRule::new(&rule_neg, &HashMap::new());

  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          someOtherFunction(1, 2, 3);
        }
       }
      }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_arguments = &PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
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
    .descendant_for_byte_range(167, 175)
    .unwrap();

  assert!(source_code_unit.is_satisfied(*node, &rule_positive, &HashMap::new(), &mut rule_store,));

  assert!(!source_code_unit.is_satisfied(*node, &rule_neg, &HashMap::new(), &mut rule_store,));
}

#[test]
fn test_satisfies_filters_sibling_count() {
  let rule_positive = piranha_rule! {
    name= "test",
    query= "(
      (method_invocation 
          name: (_) @name
          arguments: (argument_list (_)@arg)) @mi 
      (#eq? @name \"someOtherFunction\")
      (#eq? @arg \"1\")
      )",
    replace_node= "arg",
    replace= "()",
    filters= [filter!{
      , sibling_count = 3
    }]
  };
  let rule_positive = InstantiatedRule::new(&rule_positive, &HashMap::new());

  let rule_neg = piranha_rule! {
    name= "test",
    query= "(
      (method_invocation 
          name: (_) @name
          arguments: (argument_list (_) @arg )) @mi 
      (#eq? @name \"someOtherFunction\")
      (#eq? @arg \"1\")
      )",
    replace_node= "arg",
    replace= "()",
    filters= [filter!{
      , sibling_count = 2
    }]
  };
  let rule_neg = InstantiatedRule::new(&rule_neg, &HashMap::new());

  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          someOtherFunction(1, 2, 3);
        }
       }
      }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_arguments = &PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
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
    .descendant_for_byte_range(167, 168)
    .unwrap();

  assert!(source_code_unit.is_satisfied(*node, &rule_positive, &HashMap::new(), &mut rule_store,));

  assert!(!source_code_unit.is_satisfied(*node, &rule_neg, &HashMap::new(), &mut rule_store,));
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

  let start = Point { row: 1, column: 8 };
  let end = Point { row: 9, column: 9 };
  let node = &source_code_unit
    .root_node()
    .descendant_for_point_range(start.into(), end.into())
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

#[test]
fn test_satisfies_outermost_enclosing_node() {
  let rule_positive = piranha_rule! {
    name= "test",
    query= "(
      (method_declaration name: (_) @name) @md
      (#eq? @name \"foobar\")
      )",
    filters= [filter!{
      , outermost_enclosing_node = "(class_declaration) @cd"
      , contains = "((method_invocation name: (_) @mname) @mi (#eq? @mname \"foobar\"))"
    }]
  };
  let rule_positive = InstantiatedRule::new(&rule_positive, &HashMap::new());

  let rule_negative = piranha_rule! {
    name= "test",
    query= "(
      (method_declaration name: (_) @name) @md
      (#eq? @name \"foobar\")
      )",
    filters= [filter!{
      , outermost_enclosing_node = "(class_declaration) @cd"
      , not_contains = ["((method_invocation name: (_) @mname) @mi (#eq? @mname \"foobar\"))",]
    }]
  };
  let rule_negative = InstantiatedRule::new(&rule_negative, &HashMap::new());

  let source_code = "class OuterClass {

    void someMethod() {
      Test t = new Test();
      t.foobar();
    }
    class MiddleClass {
      class Test {
        private void foobar(){
          System.out.println();
         }
        }
      }
    }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_arguments = &PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
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
    .descendant_for_byte_range(119, 178)
    .unwrap();

  assert!(!source_code_unit.is_satisfied(*node, &rule_negative, &HashMap::new(), &mut rule_store,));

  assert!(source_code_unit.is_satisfied(*node, &rule_positive, &HashMap::new(), &mut rule_store,));
}

#[test]
fn test_removes_blank_lines_after_inline_cleanup() {
  let inline_cleanup_rule = piranha_rule! {
    name= "inline_cleanup_rule",
    query= "
    (
      (if_modifier
          body : ((_) @body)
          [
            condition: (false)
            condition: (parenthesized_statements (false))
          ]
      )@if_modifier
    )          
    ",
    replace_node = "if_modifier",
    replace = ""
  };

  let inline_rule = InstantiatedRule::new(&inline_cleanup_rule, &HashMap::new());

  let source_code = r#"
      def method_name
        do_something if false
      end
  "#
  .trim();

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .language(PiranhaLanguage::from(RUBY))
    .build();

  let mut rule_store = RuleStore::new(&piranha_arguments);
  let mut parser = piranha_arguments.language().parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_arguments,
  );
  let matches = source_code_unit.get_matches(
    &inline_rule,
    &mut rule_store,
    source_code_unit.root_node(),
    true,
  );
  assert_eq!(matches.len(), 1);
  assert_eq!(
    matches
      .first()
      .unwrap()
      .associated_leading_empty_lines
      .len(),
    1
  );
  assert_eq!(
    *matches
      .first()
      .unwrap()
      .associated_leading_empty_lines
      .first()
      .unwrap(),
    Range {
      start_byte: 15,
      end_byte: 24,
      start_point: Point { row: 0, column: 15 },
      end_point: Point { row: 1, column: 8 }
    }
  );
}

#[test]
fn test_switch_entry_blank_lines() {
  let inline_cleanup_rule = piranha_rule! {
    name= "inline_cleanup_rule",
    query= "
    (
        (switch_entry
            (switch_pattern) @p1
            (switch_pattern) @p2
            (switch_pattern) @p3
            (switch_pattern) @p4
            (switch_pattern) @p5
            (switch_pattern) @p6
        ) @custom_entry
        (#eq? @p1 \".case_zeta_first\")
    )
    ",
    replace_node = "custom_entry",
    replace = ""
  };

  let inline_rule = InstantiatedRule::new(&inline_cleanup_rule, &HashMap::new());

  let source_code = r#"
    public var namespace: ParameterNamespace {
        switch self {
        case .case_random_word_alpha,
             .case_random_word_beta,
             .case_random_word_gamma,
             .case_random_word_delta:
            return .namespace_group_alpha
        case .case_zeta_first,
             .case_zeta_second,
             .case_zeta_third,
             .case_zeta_fourth,
             .case_zeta_fifth,
             .case_zeta_sixth:
            return .namespace_group_beta
        case .case_random_word_omega:
            return .namespace_group_gamma
        }
    }
  "#
  .trim();

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .language(PiranhaLanguage::from(SWIFT))
    .build();

  let mut rule_store = RuleStore::new(&piranha_arguments);
  let mut parser = piranha_arguments.language().parser();

  let mut source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_arguments,
  );

  source_code_unit.apply_rule(inline_rule, &mut rule_store, &mut parser, &None);
  let transformed_code = source_code_unit.code();

  let expected_code = r#"
    public var namespace: ParameterNamespace {
        switch self {
        case .case_random_word_alpha,
             .case_random_word_beta,
             .case_random_word_gamma,
             .case_random_word_delta:
            return .namespace_group_alpha
        case .case_random_word_omega:
            return .namespace_group_gamma
        }
    }
  "#
  .trim();
  assert_eq!(transformed_code, expected_code);
}

// ============================================================
// Facts tests
// ============================================================

use crate::models::{capture_group_patterns::CGPattern, rule::RuleBuilder};

/// Helper: build a fact rule that matches Java local variable declarations and records
/// a fact keyed by the provided `fact` map (values may contain `@name` / `@value` tags).
fn make_var_fact_rule(fact: HashMap<String, String>) -> InstantiatedRule {
  let rule = RuleBuilder::default()
    .name("record_var_fact".to_string())
    .query(CGPattern::new(
      "(local_variable_declaration \
         declarator: (variable_declarator \
           name: (_) @name \
           value: (_) @value)) @decl"
        .to_string(),
    ))
    .replace_node("decl".to_string())
    .fact(fact)
    .is_seed_rule(true)
    .build()
    .unwrap();
  InstantiatedRule::new(&rule, &HashMap::new())
}

/// Helper: build a SourceCodeUnit + RuleStore for Java.
fn java_scu_and_store(
  source_code: &str, parser: &mut Parser,
) -> (SourceCodeUnit, RuleStore) {
  let java = get_java_tree_sitter_language();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![UNUSED_CODE_PATH.to_string()])
    .language(java)
    .build();
  let rule_store = RuleStore::new(&piranha_args);
  let scu = SourceCodeUnit::new(
    parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );
  (scu, rule_store)
}

/// A fact rule records one fact per match with the rule's `fact` map values
/// substituted from tree-sitter captures.
#[test]
fn test_fact_rule_records_facts() {
  let source_code = "class Test {\n  void foobar() {\n    boolean isFlagTreated = true;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  let rule = make_var_fact_rule(HashMap::from([
    ("var_name".to_string(), "@name".to_string()),
    ("initial_value".to_string(), "@value".to_string()),
  ]));

  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);

  // One variable declaration → one fact
  assert_eq!(scu.facts().len(), 1);
  let fact = &scu.facts()[0];

  // Fact data has @tag references substituted with captured values
  assert_eq!(fact.data().get("var_name").unwrap(), "isFlagTreated");
  assert_eq!(fact.data().get("initial_value").unwrap(), "true");

  // Fact is not voided
  assert!(!fact.voided());

  // Fact range points to valid byte offsets within the source
  assert!(fact.range().start_byte < fact.range().end_byte);
  assert!(fact.range().end_byte <= source_code.len());

  // Fact range actually covers the variable declaration text
  let fact_text = &source_code[fact.range().start_byte..fact.range().end_byte];
  assert!(fact_text.contains("isFlagTreated"));
  assert!(fact_text.contains("true"));
}

/// A fact rule applied to code with multiple matching nodes records one fact per match.
/// Note: `get_all_matches_for_query` returns matches bottom-to-top (reverse source order)
/// so we locate facts by their content rather than their index.
#[test]
fn test_fact_rule_records_multiple_facts_in_source_order() {
  let source_code =
    "class Test {\n  void foobar() {\n    boolean a = true;\n    boolean b = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  let rule = make_var_fact_rule(HashMap::from([
    ("var_name".to_string(), "@name".to_string()),
    ("val".to_string(), "@value".to_string()),
  ]));

  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);

  // Two variable declarations → two facts
  assert_eq!(scu.facts().len(), 2);

  // Find each fact by its var_name (order is bottom-to-top, so don't assume index)
  let fact_a = scu
    .facts()
    .iter()
    .find(|f| f.data().get("var_name").map(String::as_str) == Some("a"))
    .expect("fact for 'a' not found");
  let fact_b = scu
    .facts()
    .iter()
    .find(|f| f.data().get("var_name").map(String::as_str) == Some("b"))
    .expect("fact for 'b' not found");

  assert_eq!(fact_a.data().get("val").unwrap(), "true");
  assert_eq!(fact_b.data().get("val").unwrap(), "false");

  // Fact ranges respect source order even when the vec order doesn't
  assert!(fact_a.range().start_byte < fact_b.range().start_byte);

  // Neither is voided
  assert!(!fact_a.voided());
  assert!(!fact_b.voided());
}

/// A rewrite whose edit range overlaps a fact's range marks that fact as `voided`.
#[test]
fn test_fact_voided_by_overlapping_rewrite() {
  let source_code = "class Test {\n  void foobar() {\n    boolean isFlagTreated = true;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // Record a fact for the variable declaration
  let rule = make_var_fact_rule(HashMap::from([("k".to_string(), "@name".to_string())]));
  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);
  assert_eq!(scu.facts().len(), 1);
  assert!(!scu.facts()[0].voided());

  // Delete exactly the range the fact covers — this overlaps the fact
  let fact_range = *scu.facts()[0].range();
  let edit = Edit::delete_range(scu.code(), fact_range);
  scu.apply_edit(&edit, &mut parser);

  // The fact should now be voided
  assert!(scu.facts()[0].voided());
}

/// A rewrite that precedes a fact's range shifts the fact's byte offsets by the edit delta.
#[test]
fn test_fact_shifted_by_preceding_rewrite() {  let source_code =
    "class Test {\n  void foobar() {\n    boolean a = true;\n    boolean b = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // Record facts for both declarations
  let rule = make_var_fact_rule(HashMap::from([("k".to_string(), "@name".to_string())]));
  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);
  assert_eq!(scu.facts().len(), 2);

  // Identify the earlier fact (smaller start_byte = declaration 'a') and later fact ('b').
  // get_matches returns bottom-to-top so we determine order by byte position.
  let (early_idx, late_idx) =
    if scu.facts()[0].range().start_byte < scu.facts()[1].range().start_byte {
      (0, 1)
    } else {
      (1, 0)
    };

  let early_range = *scu.facts()[early_idx].range();
  let original_late_start = scu.facts()[late_idx].range().start_byte;
  let original_late_end = scu.facts()[late_idx].range().end_byte;
  let deleted_bytes = early_range.end_byte - early_range.start_byte;

  // Delete the earlier fact's range — entirely before the later fact
  let edit = Edit::delete_range(scu.code(), early_range);
  scu.apply_edit(&edit, &mut parser);

  // The early fact overlapped the edit → voided
  assert!(scu.facts()[early_idx].voided());

  // The late fact was entirely after the edit → byte offsets shifted left by deleted_bytes
  assert!(!scu.facts()[late_idx].voided());
  assert_eq!(
    scu.facts()[late_idx].range().start_byte,
    original_late_start - deleted_bytes
  );
  assert_eq!(
    scu.facts()[late_idx].range().end_byte,
    original_late_end - deleted_bytes
  );
}

// ============================================================
// Fact filter tests
// ============================================================

/// Helper: build a rewrite rule that deletes `boolean x = false;` declarations,
/// but only when a fact with `{ "stale": "true" }` exists in the file.
fn make_rewrite_rule_with_fact_filter() -> InstantiatedRule {
  let filter = crate::models::filter::FilterBuilder::default()
    .fact_filter(HashMap::from([("stale".to_string(), "true".to_string())]))
    .build();
  let rule = RuleBuilder::default()
    .name("delete_if_stale_fact".to_string())
    .query(CGPattern::new(
      "(local_variable_declaration \
         declarator: (variable_declarator value: (false))) @decl"
        .to_string(),
    ))
    .replace_node("decl".to_string())
    .replace("".to_string())
    .filters(std::collections::HashSet::from([filter]))
    .is_seed_rule(true)
    .build()
    .unwrap();
  InstantiatedRule::new(&rule, &HashMap::new())
}

/// When no fact satisfying `fact_filter` exists, the rewrite rule does not apply.
#[test]
fn test_fact_filter_blocks_rewrite_when_no_matching_fact() {
  let source_code = "class Test {\n  void run() {\n    boolean x = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // No facts have been recorded — fact_filter requires { stale = "true" }
  let rule = make_rewrite_rule_with_fact_filter();
  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);

  // No rewrite should have happened
  assert!(scu.rewrites().is_empty(), "rewrite must be blocked when no matching fact exists");
  assert_eq!(scu.code(), source_code, "source code must be unchanged");
}

/// When a matching fact exists, the fact_filter passes and the rewrite applies.
#[test]
fn test_fact_filter_allows_rewrite_when_matching_fact_exists() {
  let source_code = "class Test {\n  void run() {\n    boolean x = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // Manually inject a fact with { stale = "true" } — simulates a prior fact rule having fired
  use crate::models::{fact::Fact, matches::Range};
  let dummy_range = Range { start_byte: 0, end_byte: 0, ..Default::default() };
  scu
    .facts_mut()
    .push(Fact::new(dummy_range, HashMap::from([("stale".to_string(), "true".to_string())])));

  // Now run the rewrite rule — fact_filter should pass
  let rule = make_rewrite_rule_with_fact_filter();
  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);

  // The rewrite should have fired
  assert!(!scu.rewrites().is_empty(), "rewrite must fire when matching fact exists");
  assert!(!scu.code().contains("boolean x = false"), "deleted declaration must be gone");
}

/// A voided fact does NOT satisfy fact_filter — the filter only considers live (non-voided) facts.
#[test]
fn test_fact_filter_ignores_voided_facts() {
  let source_code = "class Test {\n  void run() {\n    boolean x = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // Inject a voided fact — should not satisfy fact_filter
  use crate::models::{fact::Fact, matches::Range};
  let dummy_range = Range { start_byte: 0, end_byte: 0, ..Default::default() };
  let mut voided_fact =
    Fact::new(dummy_range, HashMap::from([("stale".to_string(), "true".to_string())]));
  voided_fact.voided = true;
  scu.facts_mut().push(voided_fact);

  let rule = make_rewrite_rule_with_fact_filter();
  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);

  // Rewrite must NOT fire since the only matching fact is voided
  assert!(
    scu.rewrites().is_empty(),
    "rewrite must be blocked when the only matching fact is voided"
  );
}

/// When a fact is voided by a real rewrite (not manually injected), the fact_filter rule
/// that depended on it must NOT fire for any remaining code.
///
/// Setup:
///   - Two declarations: `x = true` and `y = false`
///   - Fact rule records `{ stale = "true" }` only for `= true` declarations
///   - We then delete `x = true`, which voids the recorded fact
///   - A fact_filter rewrite rule targeting `= false` declarations must NOT fire
#[test]
fn test_fact_filter_blocked_after_fact_voided_by_real_rewrite() {
  // Two declarations in one method
  let source_code =
    "class T {\n  void run() {\n    boolean x = true;\n    boolean y = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // Fact rule: only matches `= true` declarations, records { stale = "true" }
  let fact_rule = {
    let rule = RuleBuilder::default()
      .name("record_true_vars".to_string())
      .query(CGPattern::new(
        "(local_variable_declaration \
           declarator: (variable_declarator \
             name: (_) @name \
             value: (true))) @decl"
          .to_string(),
      ))
      .replace_node("decl".to_string())
      .fact(HashMap::from([("stale".to_string(), "true".to_string())]))
      .is_seed_rule(true)
      .build()
      .unwrap();
    InstantiatedRule::new(&rule, &HashMap::new())
  };

  scu.apply_rule(fact_rule, &mut rule_store, &mut parser, &None);

  // Only `x = true` was matched → one fact with { stale = "true" }
  assert_eq!(scu.facts().len(), 1);
  assert_eq!(scu.facts()[0].data().get("stale").unwrap(), "true");
  assert!(!scu.facts()[0].voided());

  // Void the fact by deleting the declaration `x = true`
  let fact_range = *scu.facts()[0].range();
  let edit = Edit::delete_range(scu.code(), fact_range);
  scu.apply_edit(&edit, &mut parser);

  // The fact is now voided
  assert!(scu.facts()[0].voided(), "fact must be voided after overlapping rewrite");

  // Rewrite rule: deletes `= false` declarations, gated by fact_filter { stale = "true" }
  let rewrite_rule = make_rewrite_rule_with_fact_filter();
  scu.apply_rule(rewrite_rule, &mut rule_store, &mut parser, &None);

  // `y = false` must still be present — the fact_filter blocked the rewrite
  assert!(
    scu.code().contains("boolean y = false"),
    "y = false must survive because the only stale fact was voided"
  );
  // No rewrite should have fired
  assert_eq!(
    scu.rewrites().len(),
    0,
    "no rewrite must fire when all matching facts are voided"
  );
}

/// Two separate fact rules applied to the same declaration record two independent facts
/// on the same range.  Neither fact interferes with the other.
#[test]
fn test_two_facts_recorded_on_same_range() {
  let source_code = "class T {\n  void run() {\n    boolean x = true;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  // Rule A: records { category = "bool" } for the declaration
  let rule_a = {
    let rule = RuleBuilder::default()
      .name("record_category".to_string())
      .query(CGPattern::new(
        "(local_variable_declaration \
           declarator: (variable_declarator \
             name: (_) @name \
             value: (_) @value)) @decl"
          .to_string(),
      ))
      .replace_node("decl".to_string())
      .fact(HashMap::from([("category".to_string(), "bool".to_string())]))
      .is_seed_rule(true)
      .build()
      .unwrap();
    InstantiatedRule::new(&rule, &HashMap::new())
  };

  // Rule B: records { var_name = "@name" } for the same declaration
  let rule_b = {
    let rule = RuleBuilder::default()
      .name("record_name".to_string())
      .query(CGPattern::new(
        "(local_variable_declaration \
           declarator: (variable_declarator \
             name: (_) @name \
             value: (_) @value)) @decl"
          .to_string(),
      ))
      .replace_node("decl".to_string())
      .fact(HashMap::from([("var_name".to_string(), "@name".to_string())]))
      .is_seed_rule(true)
      .build()
      .unwrap();
    InstantiatedRule::new(&rule, &HashMap::new())
  };

  scu.apply_rule(rule_a, &mut rule_store, &mut parser, &None);
  scu.apply_rule(rule_b, &mut rule_store, &mut parser, &None);

  // Two facts, both recorded for the same declaration
  assert_eq!(scu.facts().len(), 2, "expected two independent facts on the same node");

  // Both target the same range (same @decl node)
  let range_a = scu.facts()[0].range();
  let range_b = scu.facts()[1].range();
  assert_eq!(
    range_a.start_byte, range_b.start_byte,
    "both facts must share the same start_byte"
  );
  assert_eq!(
    range_a.end_byte, range_b.end_byte,
    "both facts must share the same end_byte"
  );

  // Each fact has only its own key
  let all_data: Vec<&HashMap<String, String>> =
    scu.facts().iter().map(|f| f.data()).collect();
  let has_category = all_data.iter().any(|d| d.get("category").map(String::as_str) == Some("bool"));
  let has_var_name = all_data.iter().any(|d| d.get("var_name").map(String::as_str) == Some("x"));
  assert!(has_category, "fact with category=bool not found");
  assert!(has_var_name, "fact with var_name=x not found");

  // Neither is voided
  assert!(!scu.facts()[0].voided());
  assert!(!scu.facts()[1].voided());

  // Voiding one (via a rewrite) leaves the other intact
  let fact_range = *scu.facts()[0].range(); // both have the same range
  let edit = Edit::delete_range(scu.code(), fact_range);
  scu.apply_edit(&edit, &mut parser);

  // A rewrite that overlaps the shared range must void BOTH facts
  assert!(scu.facts()[0].voided(), "fact[0] must be voided after rewrite overlaps its range");
  assert!(scu.facts()[1].voided(), "fact[1] must be voided after rewrite overlaps its range");
}

/// `validate()` must reject a rule that sets both `fact` and `replace` since they are
/// mutually exclusive.
#[test]
#[should_panic(expected = "both 'fact' and 'replace'")]
fn test_validate_rejects_fact_and_replace_together() {
  use crate::models::Validator;
  let rule = RuleBuilder::default()
    .name("bad_rule".to_string())
    .query(CGPattern::new("(local_variable_declaration) @decl".to_string()))
    .replace_node("decl".to_string())
    .replace("// removed".to_string())
    .fact(HashMap::from([("k".to_string(), "v".to_string())]))
    .is_seed_rule(true)
    .build()
    .unwrap();
  // validate() returns Err — unwrap triggers the panic that should_panic matches
  rule.validate().unwrap();
}

/// A rewrite preceding a fact's range shifts both byte offsets AND row/col points correctly.
#[test]
fn test_fact_shift_updates_row_col_points() {
  // Two declarations on separate lines so row arithmetic matters
  let source_code =
    "class T {\n  void run() {\n    boolean a = true;\n    boolean b = false;\n  }\n}";
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let (mut scu, mut rule_store) = java_scu_and_store(source_code, &mut parser);

  let rule = make_var_fact_rule(HashMap::from([("k".to_string(), "@name".to_string())]));
  scu.apply_rule(rule, &mut rule_store, &mut parser, &None);
  assert_eq!(scu.facts().len(), 2);

  // Identify the earlier fact (a) and later fact (b) by byte offset
  let (early_idx, late_idx) =
    if scu.facts()[0].range().start_byte < scu.facts()[1].range().start_byte {
      (0, 1)
    } else {
      (1, 0)
    };

  // Snapshot late fact's original points
  let orig_start_row = scu.facts()[late_idx].range().start_point.row;
  let orig_start_col = scu.facts()[late_idx].range().start_point.column;
  let orig_end_row = scu.facts()[late_idx].range().end_point.row;
  let orig_end_col = scu.facts()[late_idx].range().end_point.column;

  let early_range = *scu.facts()[early_idx].range();
  let deleted_rows =
    early_range.end_point.row as isize - early_range.start_point.row as isize;

  // Delete the early declaration — entirely before the late fact
  let edit = Edit::delete_range(scu.code(), early_range);
  scu.apply_edit(&edit, &mut parser);

  let late = &scu.facts()[late_idx];
  assert!(!late.voided());

  // Row shifts up by however many rows the deleted declaration spanned
  assert_eq!(
    late.range().start_point.row as isize,
    orig_start_row as isize - deleted_rows,
    "start row must shift by deleted rows"
  );
  assert_eq!(
    late.range().end_point.row as isize,
    orig_end_row as isize - deleted_rows,
    "end row must shift by deleted rows"
  );
  // `b` starts its own line, so the column on that line is unchanged
  assert_eq!(late.range().start_point.column, orig_start_col, "start column must be unchanged");
  assert_eq!(late.range().end_point.column, orig_end_col, "end column must be unchanged");
}
