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

use super::{
  copy_folder_to_temp_dir, create_match_tests, create_rewrite_tests,
  execute_piranha_and_check_result, initialize, substitutions,
};
use crate::{
  constraint, edges, execute_piranha,
  models::{
    default_configs::JAVA, language::PiranhaLanguage, piranha_arguments::piranha_arguments,
    rule_graph::RuleGraphBuilder,
  },
  piranha_rule,
  utilities::eq_without_whitespace,
};
use std::path::PathBuf;

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
  test_user_option_do_not_delete_if_empty : "user_option_do_not_delete_if_empty", 1, delete_file_if_empty =false;
  test_new_line_character_used_in_string_literal:  "new_line_character_used_in_string_literal",   1;
}

create_match_tests! {
  JAVA,
  test_java_match_only: "structural_find", 20;
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
  let piranha_arguments = piranha_arguments! {
    path_to_codebase = path_to_codebase,
    path_to_configurations = path_to_configurations,
    language = PiranhaLanguage::from(JAVA),
  };

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
  let piranha_arguments = piranha_arguments! {
    path_to_codebase = path_to_codebase,
    path_to_configurations = path_to_configurations,
    language = PiranhaLanguage::from(JAVA),
    substitutions = substitutions! {"super_interface_name" => "SomeInterface"},
  };

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

  let piranha_arguments = piranha_arguments! {
    path_to_configurations = path_to_scenario.join("configurations").to_str().unwrap().to_string(),
    language = PiranhaLanguage::from(JAVA),
    dry_run = true,
    code_snippet = "package com.uber.piranha;
    class SomeClass {
      void someMethod(String s) {
        assert (s.equals(\"Hello \\n World\"));
      }
    }".to_string(),
  };

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

  let piranha_arguments = piranha_arguments! {
    path_to_codebase = temp_dir.path().to_str().unwrap().to_string(),
    path_to_configurations = path_to_scenario.join("configurations").to_str().unwrap().to_string(),
    language = PiranhaLanguage::from(JAVA),
    delete_consecutive_new_lines = delete_consecutive_new_lines,
  };

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
      constraints = [
        constraint! {
          matcher = "(class_declaration ) @c_cd",
          queries = ["(
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
      groups = ["Cleanup Rule"],
      constraints = [
        constraint! {
          matcher = "(class_declaration ) @c_cd",
          queries = ["(
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

  let args = piranha_arguments! {
    path_to_codebase = temp_dir.path().to_str().unwrap().to_string(),
    language = PiranhaLanguage::from(JAVA),
    rule_graph = RuleGraphBuilder::default()
                .rules(rules)
                .edges(edges)
                .build(),
  };

  execute_piranha_and_check_result(&args, _path.join("expected").as_path(), 1, true)
}
