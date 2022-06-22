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
  assert!(content.eq(r#"name = 'Piranha'"#));
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
  assert!(result.name.eq(""));
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
