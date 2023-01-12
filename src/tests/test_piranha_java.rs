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
  check_result, copy_folder, create_match_test, create_rewrite_test, initialize, substitutions,
};
use crate::{
  execute_piranha,
  models::{
    default_configs::JAVA,
    piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
  },
};
use std::path::Path;
use tempdir::TempDir;

fn feature_flag_system_1_treated() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    JAVA,
    "test-resources/java/feature_flag_system_1/treated/input",
    "test-resources/java/feature_flag_system_1/treated/configurations",
    substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    },
  )
}

fn feature_flag_system_2_treated() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    JAVA,
    "test-resources/java/feature_flag_system_2/treated/input",
    "test-resources/java/feature_flag_system_2/treated/configurations",
    substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    },
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .cleanup_comments(true)
      .build(),
  )
}

fn feature_flag_system_1_control() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    JAVA,
    "test-resources/java/feature_flag_system_1/control/input",
    "test-resources/java/feature_flag_system_1/control/configurations",
    substitutions! {
      "stale_flag_name"=> "STALE_FLAG",
      "treated"=> "false",
      "treated_complement" => "true",
      "namespace" => "some_long_name"
    },
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .cleanup_comments(true)
      .build(),
  )
}

fn feature_flag_system_2_control() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    JAVA,
    "test-resources/java/feature_flag_system_2/control/input",
    "test-resources/java/feature_flag_system_2/control/configurations",
    substitutions! {
      "stale_flag_name"=> "STALE_FLAG",
      "treated"=> "false",
      "treated_complement" => "true",
      "namespace" => "some_long_name"
    },
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .cleanup_comments(true)
      .build(),
  )
}

fn find_and_propagate() -> PiranhaArguments {
  PiranhaArguments::new(
    JAVA,
    "test-resources/java/find_and_propagate/input",
    "test-resources/java/find_and_propagate/configurations",
  )
}

fn non_seed_user_rule() -> PiranhaArguments {
  PiranhaArguments::new(
    JAVA,
    "test-resources/java/non_seed_user_rule/input",
    "test-resources/java/non_seed_user_rule/configurations",
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .substitutions(substitutions! {"input_type_name" => "ArrayList"})
      .build(),
  )
}

fn insert_field_and_initializer() -> PiranhaArguments {
  PiranhaArguments::new(
    JAVA,
    "test-resources/java/insert_field_and_initializer/input",
    "test-resources/java/insert_field_and_initializer/configurations",
  )
}

fn new_line_character_used_in_string_literal() -> PiranhaArguments {
  PiranhaArguments::new(
    JAVA,
    "test-resources/java/new_line_character_used_in_string_literal/input",
    "test-resources/java/new_line_character_used_in_string_literal/configurations",
  )
}

fn consecutive_scope_level_rules() -> PiranhaArguments {
  PiranhaArguments::new(
    JAVA,
    "test-resources/java/consecutive_scope_level_rules/input",
    "test-resources/java/consecutive_scope_level_rules/configurations",
  )
}

create_rewrite_test! {
  test_feature_flag_system_1_treated: feature_flag_system_1_treated(), "test-resources/java/feature_flag_system_1/treated/expected",2,
  test_feature_flag_system_1_control: feature_flag_system_1_control(), "test-resources/java/feature_flag_system_1/control/expected",2,
  test_feature_flag_system_2_treated: feature_flag_system_2_treated(), "test-resources/java/feature_flag_system_2/treated/expected",4,
  test_feature_flag_system_2_control: feature_flag_system_2_control(), "test-resources/java/feature_flag_system_2/control/expected",4,
  test_scenarios_find_and_propagate:  find_and_propagate(), "test-resources/java/find_and_propagate/expected", 2,
  test_non_seed_user_rule:  non_seed_user_rule(), "test-resources/java/non_seed_user_rule/expected", 1,
  test_new_line_character_used_in_string_literal:  new_line_character_used_in_string_literal(), "test-resources/java/new_line_character_used_in_string_literal/expected", 1,
  test_insert_field_and_initializer: insert_field_and_initializer(), "test-resources/java/insert_field_and_initializer/expected", 1,
  test_consecutive_scope_level_rules:  consecutive_scope_level_rules(), "test-resources/java/consecutive_scope_level_rules/expected", 1,
}

fn match_only() -> PiranhaArguments {
  PiranhaArguments::new(
    JAVA,
    "test-resources/java/structural_find/input/XPFlagCleanerPositiveCases.java",
    "test-resources/java/structural_find/configurations",
  )
}

create_match_test! {
  test_java_match_only: match_only(), 20,
}
