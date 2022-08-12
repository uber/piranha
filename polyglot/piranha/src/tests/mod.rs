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

use std::path::{Path, PathBuf};

use crate::config::CommandLineArguments;
use crate::models::piranha_arguments::PiranhaArguments;
use crate::models::source_code_unit::SourceCodeUnit;
use crate::piranha::execute_piranha;
use crate::utilities::{eq_without_whitespace, find_file, initialize_logger, read_file};

mod test_piranha_java;
mod test_piranha_kt;

mod test_piranha_strings;

use std::sync::Once;

static INIT: Once = Once::new();

fn initialize() {
  INIT.call_once(|| {
    initialize_logger(true);
  });
}

// Runs a piranha over the target `<relative_path_to_tests>/input` (using configurations `<relative_path_to_tests>/configuration`) 
// and checks if the number of matches == `number_of_matches`. 
fn run_match_test(relative_path_to_tests: &str, number_of_matches: usize) {
  let path_to_test_ff = format!("test-resources/{relative_path_to_tests}");

  let args = PiranhaArguments::new(CommandLineArguments {
    path_to_codebase: format!("{path_to_test_ff}/input/"),
    path_to_configurations: format!("{path_to_test_ff}/configurations/"),
  });
  let (_updated_files, matches,_rewrites) = execute_piranha(&args);
  print!("{:?}", matches);
  assert_eq!(matches.values().flat_map(|x|x.iter()).count(), number_of_matches);
}

// Runs a piranha over the target `<relative_path_to_tests>/input` (using configurations `<relative_path_to_tests>/configuration`) 
// and checks if the output of piranha is same as `<relative_path_to_tests>/expected`. 
// It also asserts the number of changed files in the expected output.
fn run_rewrite_test(relative_path_to_tests: &str, n_files_changed: usize) {
  let path_to_test_ff = format!("test-resources/{relative_path_to_tests}");

  let args = PiranhaArguments::new(CommandLineArguments {
    path_to_codebase: format!("{path_to_test_ff}/input/"),
    path_to_configurations: format!("{path_to_test_ff}/configurations/"),
  });
  let (updated_files, _matches, rewrites) = execute_piranha(&args);
  // Checks if there are any rewrites performed for the file
  assert!(rewrites.values().flat_map(|x|x.iter()).count() > 0);

  assert_eq!(updated_files.len(), n_files_changed);
  let path_to_expected =
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("{path_to_test_ff}/expected"));
  check_result(updated_files, path_to_expected);
}

/// Checks if the file updates returned by piranha are as expected.
fn check_result(updated_files: Vec<SourceCodeUnit>, path_to_expected: PathBuf) {
  let mut all_files_match = true;

  for source_code_unit in &updated_files {
    let updated_file_name = &source_code_unit
      .path()
      .file_name()
      .and_then(|f| f.to_str().map(|x| x.to_string()))
      .unwrap();
    let expected_file_path = find_file(&path_to_expected, updated_file_name);
    let expected_content = read_file(&expected_file_path).unwrap();

    if !eq_without_whitespace(&source_code_unit.code(), &expected_content) {
      all_files_match = false;
      println!("{}", &source_code_unit.code());
    }
  }
  assert!(all_files_match);
}
