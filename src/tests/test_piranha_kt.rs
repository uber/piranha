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

use crate::models::{default_configs::KOTLIN, piranha_arguments::PiranhaArgumentsBuilder};

use super::{
  get_piranha_arguments_for_test, get_piranha_arguments_for_test_with_substitutions, initialize,
  run_rewrite_test,
};

static LANGUAGE: &str = "kt";

#[test]
fn test_kotlin_scenarios_treated_ff1() {
  let relative_path_to_tests = &format!("{}/{}/{}", KOTLIN, "feature_flag_system_1", "treated");

  let substitutions = vec![
    vec!["stale_flag_name".to_string(), "STALE_FLAG".to_string()],
    vec!["treated".to_string(), "true".to_string()],
    vec!["treated_complement".to_string(), "false".to_string()],
  ];

  let piranha_argument = get_piranha_arguments_for_test_with_substitutions(
    relative_path_to_tests,
    KOTLIN,
    substitutions,
  );

  run_rewrite_test(piranha_argument, 2, relative_path_to_tests);
}

#[test]
fn test_kotlin_scenarios_treated_ff2() {
  let relative_path_to_tests = &format!("{}/{}/{}", KOTLIN, "feature_flag_system_2", "treated");

  let substitutions = vec![
    vec!["stale_flag_name".to_string(), "STALE_FLAG".to_string()],
    vec!["treated".to_string(), "true".to_string()],
    vec!["treated_complement".to_string(), "false".to_string()],
    vec!["namespace".to_string(), "some_long_name".to_string()],
  ];

  let piranha_argument = PiranhaArgumentsBuilder::default()
    .cleanup_comments(true)
    .build()
    .merge(get_piranha_arguments_for_test_with_substitutions(
      relative_path_to_tests,
      KOTLIN,
      substitutions,
    ));

  run_rewrite_test(piranha_argument, 4, relative_path_to_tests);
}

#[test]
fn test_kotlin_scenarios_control_ff1() {
  let relative_path_to_tests = &format!("{}/{}/{}", KOTLIN, "feature_flag_system_1", "control");

  let substitutions = vec![
    vec!["stale_flag_name".to_string(), "STALE_FLAG".to_string()],
    vec!["treated".to_string(), "false".to_string()],
    vec!["treated_complement".to_string(), "true".to_string()],
  ];

  let piranha_argument = get_piranha_arguments_for_test_with_substitutions(
    relative_path_to_tests,
    KOTLIN,
    substitutions,
  );

  run_rewrite_test(piranha_argument, 2, relative_path_to_tests);
}

#[test]
fn test_kotlin_scenarios_control_ff2() {
  let relative_path_to_tests = &format!("{}/{}/{}", KOTLIN, "feature_flag_system_2", "control");

  let substitutions = vec![
    vec!["stale_flag_name".to_string(), "STALE_FLAG".to_string()],
    vec!["treated".to_string(), "false".to_string()],
    vec!["treated_complement".to_string(), "true".to_string()],
    vec!["namespace".to_string(), "some_long_name".to_string()],
  ];

  let piranha_argument = PiranhaArgumentsBuilder::default()
    .cleanup_comments(true)
    .build()
    .merge(get_piranha_arguments_for_test_with_substitutions(
      relative_path_to_tests,
      KOTLIN,
      substitutions,
    ));

  run_rewrite_test(piranha_argument, 4, relative_path_to_tests);
}

#[test]
fn test_kt_scenarios_file_scoped_chain_rule() {
  initialize();
  let relative_path_to_tests = &format!("{}/{}", LANGUAGE, "file_scoped_chain_rules");
  run_rewrite_test(
    get_piranha_arguments_for_test(relative_path_to_tests, KOTLIN),
    1,
    relative_path_to_tests,
  );
}
