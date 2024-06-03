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

use glob::Pattern;

use super::{
  assert_frequency_for_matches, copy_folder_to_temp_dir, create_match_tests, create_rewrite_tests,
  execute_piranha_and_check_result, initialize, substitutions,
};
use crate::{
  edges, execute_piranha, filter,
  models::{
    default_configs::JAVA, language::PiranhaLanguage, piranha_arguments::PiranhaArgumentsBuilder,
    rule_graph::RuleGraphBuilder,
  },
  piranha_rule,
  utilities::eq_without_whitespace,
};
use std::{collections::HashMap, path::PathBuf};

create_rewrite_tests! {
  JAVA,
  test_feature_flag_system_1_treated: "feature_flag_system_1/treated/", 2,
    substitutions = substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    };
  test_feature_flag_system_1_control: "feature_flag_system_1/control", 2,
    substitutions =  substitutions! {
      "stale_flag_name"=> "STALE_FLAG",
      "treated"=> "false",
      "treated_complement" => "true",
      "namespace" => "some_long_name"
    },cleanup_comments = true , delete_file_if_empty = false;
  test_feature_flag_system_2_treated: "feature_flag_system_2/treated", 4,
    substitutions = substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    }, cleanup_comments = true;
  test_feature_flag_system_2_control: "feature_flag_system_2/control/", 4,
    substitutions = substitutions! {
      "stale_flag_name"=> "STALE_FLAG",
      "treated"=> "false",
      "treated_complement" => "true",
      "namespace" => "some_long_name"
    }, cleanup_comments = true , delete_file_if_empty = false;
  test_scenarios_find_and_propagate:  "find_and_propagate", 2, substitutions = substitutions! {"super_interface_name" => "SomeInterface"},  delete_file_if_empty = false;
  test_non_seed_user_rule:  "non_seed_user_rule", 1, substitutions = substitutions! {"input_type_name" => "ArrayList"};
  test_insert_field_and_initializer:  "insert_field_and_initializer", 1;
  test_user_option_delete_if_empty: "user_option_delete_if_empty", 1;
  test_user_option_do_not_delete_if_empty : "user_option_do_not_delete_if_empty", 1, delete_file_if_empty = false;
  test_new_line_character_used_in_string_literal:  "new_line_character_used_in_string_literal",   1;
  test_java_delete_method_invocation_argument: "delete_method_invocation_argument", 1;
  test_java_delete_method_invocation_argument_no_op: "delete_method_invocation_argument_no_op", 0;
  test_regex_based_matcher: "regex_based_matcher", 1, cleanup_comments = true;
  test_parent_iterative: "parent_iterative/positive", 1;
  // This test shows what happens if edges is not PARENT_ITERATIVE
  // It tests the same scenario as the above test, but with edge kind as PARENT
  test_parent_iterative_negative: "parent_iterative/negative", 1;
}

create_match_tests! {
  JAVA,
  test_java_match_only: "structural_find",
              HashMap::from([
                  ("find_enum_constant", 1),
                  ("find_method", 1),
                  ("replace_isToggleEnabled_with_boolean_literal", 20)
                  ]);
  test_java_match_only_with_include_exclude: "structural_find_with_include_exclude",
              HashMap::from([
                  ("find_enum_constant", 1),
                  ("find_method", 1),
                  ("replace_isToggleEnabled_with_boolean_literal", 20)
                  ]),
              include = vec![Pattern::new("*/folder_2/**/*").unwrap()],
              exclude = vec![Pattern::new("*/folder_2_1/**/*").unwrap()];
}

#[test]
#[should_panic(expected = "Could not fetch range or node for replace_node")]
fn test_delete_method_invocation_argument_invalid() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("delete_method_invocation_argument_invalid");
  let path_to_codebase = _path.join("input").to_str().unwrap().to_string();
  let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![path_to_codebase])
    .path_to_configurations(path_to_configurations)
    .language(PiranhaLanguage::from(JAVA))
    .build();

  let _ = execute_piranha(&piranha_arguments);
}

#[test]
#[should_panic(
  expected = "Could not instantiate the rule Rule { name: \"find_interface_extension\""
)]
fn test_scenarios_find_and_propagate_panic() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("find_and_propagate");
  let path_to_codebase = _path.join("input").to_str().unwrap().to_string();
  let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![path_to_codebase])
    .path_to_configurations(path_to_configurations)
    .language(PiranhaLanguage::from(JAVA))
    .build();

  let _ = execute_piranha(&piranha_arguments);
}

#[test]
#[should_panic(expected = "Could not instantiate the rule Rule { name: \"delete_class\"")]
fn test_scenarios_find_and_propagate_invalid_substitutions_panic() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("find_and_propagate_invalid_substitutions");
  let path_to_codebase = _path.join("input").to_str().unwrap().to_string();
  let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![path_to_codebase])
    .path_to_configurations(path_to_configurations)
    .language(PiranhaLanguage::from(JAVA))
    .substitutions(substitutions! {"super_interface_name" => "SomeInterface"})
    .build();

  let _ = execute_piranha(&piranha_arguments);
}

#[test]
fn test_user_option_delete_consecutive_lines() {
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("user_option_delete_consecutive_lines");
  _helper_user_option_delete_consecutive_lines(_path, true);
}

#[test]
fn test_new_line_character_used_in_string_literal_code_snippet() {
  initialize();
  let path_to_scenario = PathBuf::from("test-resources")
    .join(JAVA)
    .join("new_line_character_used_in_string_literal");
  let code_snippet = "package com.uber.piranha;
  class SomeClass {
    void someMethod(String s) {
      assert (s.equals(\"Hello \\n World\"));
    }
  }";
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .path_to_configurations(
      path_to_scenario
        .join("configurations")
        .to_str()
        .unwrap()
        .to_string(),
    )
    .language(PiranhaLanguage::from(JAVA))
    .code_snippet(code_snippet.to_string())
    .build();

  let expected = "package com.uber.piranha;
  class SomeClass {
    void someMethod(String s) {
      assert (\"Hello \\n World\".equals(s));
    }
  }";
  let output_summaries = execute_piranha(&piranha_arguments);
  assert!(output_summaries.len() == 1);
  assert!(eq_without_whitespace(
    output_summaries[0].content(),
    expected
  ));
  assert!(output_summaries[0].original_content().eq(code_snippet));
}

#[test]
fn test_user_option_do_not_delete_consecutive_lines() {
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("user_option_do_not_delete_consecutive_lines");
  _helper_user_option_delete_consecutive_lines(_path, false);
}

fn _helper_user_option_delete_consecutive_lines(
  path_to_scenario: PathBuf, delete_consecutive_new_lines: bool,
) {
  initialize();

  let temp_dir = copy_folder_to_temp_dir(&path_to_scenario.join("input"));

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![temp_dir.path().to_str().unwrap().to_string()])
    .path_to_configurations(
      path_to_scenario
        .join("configurations")
        .to_str()
        .unwrap()
        .to_string(),
    )
    .language(PiranhaLanguage::from(JAVA))
    .delete_consecutive_new_lines(delete_consecutive_new_lines)
    .build();

  execute_piranha_and_check_result(
    &piranha_arguments,
    &path_to_scenario.join("expected"),
    1,
    false,
  );
  // Delete temp_dir
  temp_dir.close().unwrap();
}

#[test]
fn test_consecutive_scope_level_rules() {
  super::initialize();
  let _path = std::path::PathBuf::from("test-resources")
    .join(JAVA)
    .join("consecutive_scope_level_rules");
  let temp_dir = super::copy_folder_to_temp_dir(&_path.join("input"));

  let rules = vec![
    piranha_rule! {
      name = "add_inner_class",
      query = "(
        (class_declaration name: (_)@class_name 
            body : (class_body ((_)*) @class_members)  @class_body
        ) @class_declaration
        (#eq? @class_name \"FooBar\")
        )",
      replace_node = "class_body",
      replace = "{
        @class_members
        public class InnerFooBar {  
            private String name;
        }  
        }",
      filters = [
        filter! {
          enclosing_node =  "(class_declaration ) @c_cd",
          not_contains = ["(
            (class_declaration name:(_) @name ) @cd
            (#eq? @name \"InnerFooBar\")
            )",]
        }
      ]
    },
    piranha_rule! {
      name = "add_field_declaration",
      query = "(
        (class_declaration name: (_)@class_name
            body : (class_body ((_)*) @class_members)  @class_body
         ) @class_declaration
        )",
      replace_node = "class_body",
      replace = "{\n private String address;\n @class_members \n}",
      is_seed_rule= false,
      filters = [
        filter! {
          enclosing_node =  "(class_declaration ) @c_cd",
          not_contains = ["(
          (field_declaration (variable_declarator name:(_) @name )) @field
          (#eq? @name \"address\")
          )",]
        }
      ]
    },
  ];

  let edges = vec![edges! {
    from = "add_inner_class",
    to = ["add_field_declaration"],
    scope = "Class"
  }];

  let args = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![temp_dir.path().to_str().unwrap().to_string()])
    .language(PiranhaLanguage::from(JAVA))
    .rule_graph(
      RuleGraphBuilder::default()
        .rules(rules)
        .edges(edges)
        .build(),
    )
    .build();

  execute_piranha_and_check_result(&args, _path.join("expected").as_path(), 1, true)
}

/// This test is to check if Piranha is able to handle a syntactically incorrect tree.
#[test]
fn test_handle_syntactically_incorrect_tree() {
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("handle_syntactically_incorrect_tree");
  let temp_dir = copy_folder_to_temp_dir(&_path.join("input"));

  let rule = piranha_rule! {
    name = "Append l",
    query = "(
  (variable_declarator value: (decimal_integer_literal) @value)
  (#not-match? @value \"l|L\")
  )",
    replace_node = "value",
    replace = "@valuel"
  };

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![temp_dir.path().to_str().unwrap().to_string()])
    .language(PiranhaLanguage::from(JAVA))
    .rule_graph(RuleGraphBuilder::default().rules(vec![rule]).build())
    .allow_dirty_ast(true)
    .build();

  execute_piranha_and_check_result(&piranha_arguments, &_path.join("expected"), 1, true);
  // Delete temp_dir
  temp_dir.close().unwrap();
}

/// This test is to check if Piranha panics when it encounters a syntactically incorrect tree and
/// allow_dirty_ast is *not* set (to true).
#[test]
#[should_panic(expected = "Produced syntactically incorrect source code")]
fn test_do_not_allow_syntactically_incorrect_tree() {
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("handle_syntactically_incorrect_tree");
  let temp_dir = copy_folder_to_temp_dir(&_path.join("input"));

  let rule = piranha_rule! {
    name = "Append l",
    query = "(
  (variable_declarator value: (decimal_integer_literal) @value)
  (#not-match? @value \"l|L\")
  )",
    replace_node = "value",
    replace = "@valuel"
  };

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![temp_dir.path().to_str().unwrap().to_string()])
    .language(PiranhaLanguage::from(JAVA))
    .rule_graph(RuleGraphBuilder::default().rules(vec![rule]).build())
    .build();

  execute_piranha_and_check_result(&piranha_arguments, &_path.join("expected"), 1, true);
  // Delete temp_dir
  temp_dir.close().unwrap();
}

/// This test is to check if Piranha is able to handle a syntactically incorrect tree.
/// We expect Piranha to panic in this case because the rule produces a "more" syntactically incorrect tree.
#[test]
#[should_panic(expected = "Produced syntactically incorrect source code")]
fn test_handle_syntactically_incorrect_tree_panic() {
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("handle_syntactically_incorrect_tree");
  let temp_dir = copy_folder_to_temp_dir(&_path.join("input"));

  let rule = piranha_rule! {
    name = "Append x (wrong rule)",
    query = "(
  (variable_declarator value: (decimal_integer_literal) @value)
  (#not-match? @value \"X|x\")
  )",
    replace_node = "value",
    // Purposefully appending `x` so that it results in a incorrect syntax tree.
    replace = "@valuex"
  };

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![temp_dir.path().to_str().unwrap().to_string()])
    .language(PiranhaLanguage::from(JAVA))
    .rule_graph(RuleGraphBuilder::default().rules(vec![rule]).build())
    .build();

  execute_piranha_and_check_result(&piranha_arguments, &_path.join("expected"), 1, true);
  // Delete temp_dir
  temp_dir.close().unwrap();
}

#[test]
#[should_panic(expected = "Cannot parse")]
fn test_incorrect_rule() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("incorrect_rule");
  let path_to_codebase = _path.join("input").to_str().unwrap().to_string();
  let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .paths_to_codebase(vec![path_to_codebase])
    .path_to_configurations(path_to_configurations)
    .language(PiranhaLanguage::from(JAVA))
    .build();

  let _ = execute_piranha(&piranha_arguments);
}

/// This test is to check if Piranha is able to handle a syntactically incorrect tree.
#[test]
fn test_dyn_rule() {
  let rule = piranha_rule! {
    name = "match_class",
    query = "cs println(:[xs])",
    replace_node = "*",
    replace = "println2(@xs, 2)"
  };

  let piranha_arguments = PiranhaArgumentsBuilder::default()
      .code_snippet(String::from("class A { public static      void main(String[] args) { if(println    (  StaleFlag )) { println(x(println(println(u)))); println(x -> foo()); println(x->y); } } }"))
    .language(PiranhaLanguage::from(JAVA))
    .rule_graph(RuleGraphBuilder::default().rules(vec![rule]).build())
    .allow_dirty_ast(true)
    .build();

  let output_summaries = execute_piranha(&piranha_arguments);
  // assert
  assert_eq!(output_summaries.len(), 1);
}

#[test]
fn test_multiple_code_bases() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("structural_find_replace_multiple_code_bases");
  let path_to_codebase1 = _path.join("folder_1").to_str().unwrap().to_string();
  let path_to_codebase2 = _path.join("folder_2").to_str().unwrap().to_string();
  let rule = piranha_rule! {
    name = "match_import",
    query = "cs import java.util.List;"
  };
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .language(PiranhaLanguage::from(JAVA))
    .paths_to_codebase(vec![path_to_codebase1, path_to_codebase2])
    .rule_graph(RuleGraphBuilder::default().rules(vec![rule]).build())
    .build();
  let output_summaries = execute_piranha(&piranha_arguments);
  // Note that we expect 2 matches because we have 2 code bases, and each code base has 1 match.
  // We also have another codebase `folder_3` but the `paths_to_codebase` does not include it.
  assert_eq!(output_summaries.len(), 2);
  assert_frequency_for_matches(&output_summaries, &HashMap::from([("match_import", 2)]));
}

#[test]
#[should_panic(
  expected = "Path to codebase does not exist: test-resources/java/structural_find_replace_multiple_code_bases/folder_0"
)]
fn test_incorrect_codebase_path() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("structural_find_replace_multiple_code_bases");
  // Note that structural_find_replace_multiple_code_bases/folder_0 does not exist
  let path_to_codebase1 = _path.join("folder_0").to_str().unwrap().to_string();
  let rule = piranha_rule! {
    name = "match_import",
    query = "cs import java.util.List;"
  };
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .language(PiranhaLanguage::from(JAVA))
    .paths_to_codebase(vec![path_to_codebase1])
    .rule_graph(RuleGraphBuilder::default().rules(vec![rule]).build())
    .build();
  _ = execute_piranha(&piranha_arguments);
}
