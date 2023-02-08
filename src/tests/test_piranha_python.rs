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

use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};

use std::{fs::File, process::Command};
use tempdir::TempDir;

use super::{create_match_tests, create_rewrite_tests, substitutions};

use crate::{
  models::{default_configs::PYTHON, piranha_output::PiranhaOutputSummary},
  utilities::read_file,
};

create_rewrite_tests!(
  PYTHON,
  test_delete_modify_str_literal_from_list:  "delete_cleanup_str_in_list", 1,
  substitutions = substitutions! {
    "str_literal" => "dependency2",
    "str_to_replace"=>  "dependency1",
    "str_replacement" => "dependency1_1"
  };
);

#[test]
fn test_cli() {
  let temp_dir = TempDir::new_in(".", "tmp_test").unwrap();
  let temp_file = temp_dir.path().join("output.txt");
  _ = File::create(temp_file.as_path());

  let mut cmd = Command::cargo_bin("polyglot_piranha").unwrap();
  cmd
    .args(["-c", "test-resources/py/delete_cleanup_str_in_list/input"])
    .args([
      "-f",
      "test-resources/py/delete_cleanup_str_in_list/configurations",
    ])
    .args(["-p", "py"])
    .args(["-j", temp_file.to_str().unwrap()])
    .arg("--dry-run")
    .args(["-S", "str_literal=dependency2"])
    .args(["-S", "str_to_replace=dependency1"])
    .args(["-S", "str_replacement=dependency1_1"]);

  cmd.assert().success();

  let content = read_file(&temp_file).unwrap();
  assert!(!content.is_empty());
  let output: Vec<PiranhaOutputSummary> = serde_json::from_str(&content).unwrap();
  assert!(!output.is_empty());
  assert!(output[0].path().to_str().unwrap().contains("only_lists.py"));
  assert!(!output[0].rewrites().is_empty());

  _ = temp_dir.close();
}

create_match_tests!(PYTHON, test_match_only: "structural_find", 3;);
