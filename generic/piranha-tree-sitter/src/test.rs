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

use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use colored::Colorize;
use log::info;

use crate::config::command_line_arguments::{CommandLineArguments, PiranhaArguments};
use crate::piranha::{FlagCleaner, SourceCodeUnit};
use crate::utilities::{initialize_logger, read_file};
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
  INIT.call_once(|| {
    initialize_logger(true);
  });
}
#[test]
fn test_java_scenarios_treated() {
  initialize();
  let language = "java";
  let path_to_test_resource =
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("src/test-resources/{language}"));
  let args = PiranhaArguments::new(CommandLineArguments {
    path_to_codebase: format!("src/test-resources/{language}/input/"),
    path_to_feature_flag_rules: format!("src/test-resources/{language}/configurations/"),
    path_to_piranha_arguments: format!("src/test-resources/{language}/configurations/")
      + "piranha_arguments_treated.toml",
  });

  let updated_files = get_cleanups_for_code_base_new(args);

  let path_to_expected = path_to_test_resource.join("expected_treated");

  assert_eq!(updated_files.len(), 5);

  check_result(updated_files, path_to_expected);
}

#[test]
fn test_java_scenarios_control() {
  initialize();
  let language = "java";
  let path_to_test_resource =
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("src/test-resources/{language}"));
  let args = PiranhaArguments::new(CommandLineArguments {
    path_to_codebase: format!("src/test-resources/{language}/input/"),
    path_to_feature_flag_rules: format!("src/test-resources/{language}/configurations/"),
    path_to_piranha_arguments: format!("src/test-resources/{language}/configurations/")
      + "piranha_arguments_control.toml",
  });

  let updated_files = get_cleanups_for_code_base_new(args);

  let path_to_expected = path_to_test_resource.join("expected_control");

  assert_eq!(updated_files.len(), 5);

  check_result(updated_files, path_to_expected);
}

/// Compares two strings, ignoring new lines, and space.
fn eq_without_whitespace(s1: &String, s2: &String) -> bool {
  s1.replace("\n", "")
    .replace(" ", "")
    .eq(&s2.replace("\n", "").replace(" ", ""))
}

/// Checks if the file updates returned by piranha are as expected.
fn check_result(updated_files: Vec<SourceCodeUnit>, path_to_expected: PathBuf) {
  let mut results = HashMap::new();
  for source_code_unit in &updated_files {
    let updated_file_name = &source_code_unit
      .path()
      .file_name()
      .and_then(|f| f.to_str().map(|x| x.to_string()))
      .unwrap();
    let expected_file_path = find_file(&path_to_expected, &updated_file_name);
    let expected_content = read_file(&expected_file_path).unwrap();
    let result = eq_without_whitespace(&source_code_unit.code(), &expected_content);
    results.insert(source_code_unit.path().clone(), result);
  }

  let mut all_files_match = true;
  for (file_name, is_as_expected) in results {
    if is_as_expected {
      #[rustfmt::skip]
      info!("{}", format!("Match successful for {:?}", file_name).green());
    } else {
      info!("{}", format!("Match failed for {:?}", file_name).red());
      all_files_match = false;
    }
  }
  assert!(all_files_match);
}

fn get_cleanups_for_code_base_new(args: PiranhaArguments) -> Vec<SourceCodeUnit> {
  let mut flag_cleaner = FlagCleaner::new(args);

  flag_cleaner.perform_cleanup();

  flag_cleaner.get_updated_files()
}

/// Checks if the given `dir_entry` is a file named `file_name`
pub fn has_name(dir_entry: &DirEntry, file_name: &str) -> bool {
  dir_entry
    .path()
    .file_name()
    .map(|e| e.eq(file_name))
    .unwrap_or(false)
}

/// Returns the file with the given name within the given directory.
pub fn find_file(input_dir: &PathBuf, name: &String) -> PathBuf {
  fs::read_dir(input_dir)
    .unwrap()
    .filter_map(|d| d.ok())
    .filter(|de| has_name(de, name))
    .next()
    .unwrap()
    .path()
}
