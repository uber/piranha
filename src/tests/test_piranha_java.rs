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
  execute_piranha,
  models::{default_configs::JAVA, piranha_arguments::piranha_arguments},
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
    },cleanup_comments = true , delete_file_if_empty= false;
  test_feature_flag_system_2_treated: "feature_flag_system_2/treated", 4,
    substitutions= substitutions! {
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
    }, cleanup_comments = true , delete_file_if_empty= false;
  test_scenarios_find_and_propagate:  "find_and_propagate", 2, substitutions = substitutions! {"super_interface_name" => "SomeInterface"},  delete_file_if_empty= false;
  test_non_seed_user_rule:  "non_seed_user_rule", 1, substitutions = substitutions! {"input_type_name" => "ArrayList"};
  test_new_line_character_used_in_string_literal:  "new_line_character_used_in_string_literal",   1;
  test_insert_field_and_initializer:  "insert_field_and_initializer", 1;
  test_consecutive_scope_level_rules: "consecutive_scope_level_rules",1;
  test_user_option_delete_if_empty: "user_option_delete_if_empty", 1;
  test_user_option_do_not_delete_if_empty : "user_option_do_not_delete_if_empty", 1, delete_file_if_empty=false;
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
    language= JAVA.to_string(),
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
    language = JAVA.to_string(),
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
