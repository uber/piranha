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

use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};

use std::{collections::HashMap, fs::File, path::Path, process::Command};
use tempdir::TempDir;

use super::create_match_tests;

use crate::{
  models::{default_configs::PYTHON, piranha_output::PiranhaOutputSummary},
  utilities::{eq_without_whitespace, read_file},
};

/// This test is almost equivalent to create_rewrite_tests!(PYTHON, test_delete_modify_str_literal_from_list: ...)
/// It is "almost equivalent" because we pass `--dry-run`and the compare the contents of
/// of expected files against `PiranhaOutputSummary`
#[test]
fn test_delete_modify_str_literal_from_list_via_cli() {
  let temp_dir = TempDir::new_in(".", "tmp_test").unwrap();
  let temp_file = temp_dir.path().join("output.json");
  _ = File::create(temp_file.as_path());

  let mut cmd = Command::cargo_bin("polyglot_piranha").unwrap();
  cmd
    .args([
      "-c",
      "test-resources/python/delete_cleanup_str_in_list/input",
    ])
    .args([
      "-f",
      "test-resources/python/delete_cleanup_str_in_list/configurations",
    ])
    .args(["-l", "python"])
    .args(["-j", temp_file.to_str().unwrap()])
    .arg("--dry-run")
    .args(["-s", "str_literal=dependency2"])
    .args(["-s", "str_to_replace=dependency1"])
    .args(["-s", "str_replacement=dependency1_1"]);

  cmd.assert().success();
  let content = read_file(&temp_file).unwrap();
  let output: Vec<PiranhaOutputSummary> = serde_json::from_str(&content).unwrap();
  let expected_path =
    Path::new("test-resources/python/delete_cleanup_str_in_list/expected/only_lists.py");
  let expected = read_file(&expected_path.to_path_buf()).unwrap();
  assert!(eq_without_whitespace(output[0].content(), &expected));

  _ = temp_dir.close();
}

create_match_tests!(PYTHON, test_match_only: "structural_find", HashMap::from([("find_lists_with_str_literals", 3)]););
