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

use crate::models::default_configs::GO;

use super::{
  get_piranha_arguments_for_test, get_piranha_arguments_for_test_with_substitutions, initialize,
  run_match_test, run_rewrite_test,
};

#[test]
fn test_go_match_only_go_expr_for_loop() {
  initialize();
  let relative_path_to_tests = &format!("{}/{}/{}", GO, "structural_find", "go_stmt_for_loop");
  run_match_test(
    get_piranha_arguments_for_test(relative_path_to_tests, GO),
    1,
  );
}

#[test]
fn test_go_match_only_for_loop() {
  initialize();
  let relative_path_to_tests = &format!("{}/{}/{}", GO, "structural_find", "for_loop");
  run_match_test(
    get_piranha_arguments_for_test(relative_path_to_tests, GO),
    4,
  );
}

#[test]
fn test_go_builtin_boolean_expression_simplify() {
  initialize();
  let relative_path_to_tests = &format!(
    "{}/{}/{}/{}",
    GO, "feature_flag", "builtin_rules", "boolean_expression_simplify"
  );
  let substitutions = vec![
    vec!["true_flag_name".to_string(), "true".to_string()],
    vec!["false_flag_name".to_string(), "false".to_string()],
    vec!["nil_flag_name".to_string(), "nil".to_string()],
  ];

  let piranha_argument =
    get_piranha_arguments_for_test_with_substitutions(relative_path_to_tests, GO, substitutions);

  run_rewrite_test(piranha_argument, 1, relative_path_to_tests);
}

#[test]
fn test_go_builtin_statement_cleanup() {
  initialize();
  let relative_path_to_tests = &format!(
    "{}/{}/{}/{}",
    GO, "feature_flag", "builtin_rules", "statement_cleanup"
  );
  let substitutions = vec![
    vec!["treated_complement".to_string(), "false".to_string()],
    vec!["treated".to_string(), "true".to_string()],
  ];

  let piranha_argument =
    get_piranha_arguments_for_test_with_substitutions(relative_path_to_tests, GO, substitutions);

  run_rewrite_test(piranha_argument, 1, relative_path_to_tests);
}

#[test]
fn test_go_const_same_file() {
  initialize();

  let relative_path_to_tests = &format!(
    "{}/{}/{}/{}",
    GO, "feature_flag", "system_1", "const_same_file"
  );

  let substitutions = vec![
    vec!["stale_flag_name".to_string(), "staleFlag".to_string()],
    vec!["treated".to_string(), "false".to_string()],
  ];
  let piranha_argument =
    get_piranha_arguments_for_test_with_substitutions(relative_path_to_tests, GO, substitutions);

  run_rewrite_test(piranha_argument, 1, relative_path_to_tests);
}
