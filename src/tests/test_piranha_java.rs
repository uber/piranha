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
  check_result, copy_folder, create_match_tests, create_rewrite_tests, initialize, substitutions,
};
use crate::{
  execute_piranha,
  models::{
    default_configs::JAVA,
    piranha_arguments::{piranha_arguments, PiranhaArgumentsBuilder},
  },
};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

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
  test_scenarios_find_and_propagate:  "find_and_propagate", 2, delete_file_if_empty= false;
  test_non_seed_user_rule:  "non_seed_user_rule", 1, substitutions = substitutions! {"input_type_name" => "ArrayList"};
  test_new_line_character_used_in_string_literal:  "new_line_character_used_in_string_literal",   1;
  test_insert_field_and_initializer:  "insert_field_and_initializer", 1;
  test_consecutive_scope_level_rules: "consecutive_scope_level_rules",1;
}

create_match_tests! {
  JAVA,
  test_java_match_only: "structural_find", 20;
}
