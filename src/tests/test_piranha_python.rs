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

use crate::models::default_configs::PYTHON;

use super::{
  get_piranha_arguments_for_test, get_piranha_arguments_for_test_with_substitutions, initialize,
  run_match_test, run_rewrite_test,
};

static LANGUAGE: &str = "python";

#[test]
fn test_python_delete_modify_str_literal_from_list() {
  let relative_path_to_tests = &format!("{}/{}", LANGUAGE, "delete_cleanup_str_in_list");

  let substitutions = vec![
    vec!["str_literal".to_string(), "dependency2".to_string()],
    vec!["str_to_replace".to_string(), "dependency1".to_string()],
    vec!["str_replacement".to_string(), "dependency1_1".to_string()],
  ];

  let piranha_argument = get_piranha_arguments_for_test_with_substitutions(
    relative_path_to_tests,
    PYTHON,
    substitutions,
  );

  run_rewrite_test(piranha_argument, 1, relative_path_to_tests);
}

#[test]
fn test_python_match_only() {
  initialize();
  let relative_path_to_tests = &format!("{}/{}", LANGUAGE, "structural_find");
  run_match_test(
    get_piranha_arguments_for_test(relative_path_to_tests, PYTHON),
    3,
  );
}
