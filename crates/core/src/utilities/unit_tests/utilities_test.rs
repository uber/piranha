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

use crate::utilities::find_file;
use serde_derive::Deserialize;
use std::path::PathBuf;

use super::{read_file, read_toml};

#[derive(Deserialize, Default)]
struct TestStruct {
  name: String,
}

#[test]
fn test_read_file() {
  let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path_to_test_file = project_root.join("test-resources/utility_tests/sample.toml");
  let result = read_file(&path_to_test_file);
  assert!(result.is_ok());
  let content = result.ok().unwrap();
  assert!(!content.is_empty());
  assert!(content.trim_end().eq(r#"name = 'Piranha'"#));
}

#[test]
fn test_read_toml() {
  let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path_to_test_file = project_root.join("test-resources/utility_tests/sample.toml");
  let result: TestStruct = read_toml(&path_to_test_file, false);
  assert!(result.name.eq("Piranha"));
}

#[test]
fn test_read_toml_default() {
  let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path_to_test_file =
    project_root.join("test-resources/utility_tests/another_sample.toml.toml");
  let result: TestStruct = read_toml(&path_to_test_file, true);
  assert!(result.name.is_empty());
}

#[test]
fn test_find_file_positive() {
  let project_root =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources/utility_tests/");
  let f = find_file(&project_root, "sample.toml");
  assert!(f.is_file());
}

#[test]
#[should_panic]
fn test_find_file_negative() {
  let project_root =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources/utility_tests/");
  let f = find_file(&project_root, "another_sample.toml.toml");
  assert!(f.is_file());
}
