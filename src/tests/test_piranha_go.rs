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

use crate::models::{default_configs::GO, piranha_arguments::PiranhaArgumentsBuilder};

use super::{initialize, run_match_test, run_match_test_for_args, run_rewrite_test};

#[test]
fn test_go_match_only_go_expr_for_loop() {
  initialize();
  let relative_path_to_tests = &format!("{}/{}/{}", GO, "structural_find", "go_stmt_for_loop");
  let args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(format!("test-resources/{relative_path_to_tests}/input/"))
    .path_to_configurations(format!(
      "test-resources/{relative_path_to_tests}/configurations/"
    ))
    .language(vec![GO.to_string()])
    .build();

  run_match_test_for_args(args, 1);
}

#[test]
fn test_go_match_only_for_loop() {
  initialize();
  run_match_test(&format!("{}/{}/{}", GO, "structural_find", "for_loop"), 4);
}

#[test]
fn test_go_builtin_boolean_expression_simplify() {
  initialize();
  run_rewrite_test(
    &format!(
      "{}/{}/{}/{}",
      GO, "feature_flag", "builtin_rules", "boolean_expression_simplify"
    ),
    1,
  );
}

#[test]
fn test_go_builtin_statement_cleanup() {
  initialize();
  run_rewrite_test(
    &format!(
      "{}/{}/{}/{}",
      GO, "feature_flag", "builtin_rules", "statement_cleanup"
    ),
    1,
  );
}

#[test]
fn test_go_const_same_file() {
  initialize();
  run_rewrite_test(
    &format!(
      "{}/{}/{}/{}",
      GO, "feature_flag", "system_1", "const_same_file"
    ),
    1,
  );
}
