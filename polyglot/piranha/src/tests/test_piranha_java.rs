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

use crate::models::default_configs::JAVA;

use super::{initialize, run_match_test, run_rewrite_test, run_match_test_for_file};

static LANGUAGE: &str = JAVA;

#[test]
fn test_java_scenarios_treated_ff1() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_1", "treated"),
    2,
  );
}

#[test]
fn test_java_scenarios_treated_ff2() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_2", "treated"),
    4,
  );
}

#[test]
fn test_java_scenarios_control_ff1() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_1", "control"),
    2,
  );
}

#[test]
fn test_java_scenarios_control_ff2() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_2", "control"),
    4,
  );
}

#[test]
fn test_java_scenarios_find_and_propagate() {
  initialize();
  run_rewrite_test(&format!("{}/{}", LANGUAGE, "find_and_propagate"), 2);
}

#[test]
fn test_java_scenarios_user_defined_non_seed_rules() {
  initialize();
  run_rewrite_test(&format!("{}/{}", LANGUAGE, "non_seed_user_rule"), 1);
}

#[test]
fn test_java_scenarios_insert_field_and_initializer() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}", LANGUAGE, "insert_field_and_initializer"),
    1,
  );
}

#[test]
fn test_java_scenarios_new_line_character_used_in_string_literal() {
  initialize();
  run_rewrite_test(
    &format!(
      "{}/{}",
      LANGUAGE, "new_line_character_used_in_string_literal"
    ),
    1,
  );
}

#[test]
fn test_java_scenarios_consecutive_scope_level_rules() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}", LANGUAGE, "consecutive_scope_level_rules"),
    1,
  );
}

// run_match_test
#[test]
fn test_java_match_only() {
  initialize();
  run_match_test_for_file(&format!("{}/{}", LANGUAGE, "structural_find"),"XPFlagCleanerPositiveCases.java",  20);
}
