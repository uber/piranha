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

use log::error;

use crate::config::CommandLineArguments;
use crate::execute_piranha;
use crate::models::piranha_arguments::PiranhaArguments;
use crate::models::piranha_output::PiranhaOutputSummary;
use crate::utilities::{eq_without_whitespace, find_file, read_file};

mod test_piranha_java;
mod test_piranha_kt;

mod test_piranha_strings;

mod test_piranha_swift;

mod test_piranha_python;

mod test_piranha_go;
mod test_piranha_ts;
mod test_piranha_tsx;

use std::sync::Once;

static INIT: Once = Once::new();

fn initialize() {
  INIT.call_once(|| {
    env_logger::init();
  });
}

// Runs a piranha over the target `<relative_path_to_tests>/input` (using configurations `<relative_path_to_tests>/configuration`)
// and checks if the number of matches == `number_of_matches`.
fn run_match_test(relative_path_to_tests: &str, number_of_matches: usize) {
  let path_to_test_ff = format!("test-resources/{relative_path_to_tests}");

  let args = PiranhaArguments::new(CommandLineArguments {
    path_to_codebase: format!("{path_to_test_ff}/input/"),
    path_to_configurations: format!("{path_to_test_ff}/configurations/"),
    path_to_output_summary: None,
    dry_run: true,
  });
  let output_summaries = execute_piranha(&args);

  assert_eq!(
    output_summaries
      .iter()
      .flat_map(|os| os.matches().iter())
      .count(),
    number_of_matches
  );
}

// Runs a piranha over the target `<relative_path_to_tests>/input` (using configurations `<relative_path_to_tests>/configuration`)
// and checks if the output of piranha is same as `<relative_path_to_tests>/expected`.
// It also asserts the number of changed files in the expected output.
fn run_rewrite_test(relative_path_to_tests: &str, n_files_changed: usize) {
  let path_to_test_ff = format!("test-resources/{relative_path_to_tests}");

  let args = PiranhaArguments::new(CommandLineArguments {
    path_to_codebase: format!("{path_to_test_ff}/input/"),
    path_to_configurations: format!("{path_to_test_ff}/configurations/"),
    path_to_output_summary: None,
    dry_run: true,
  });
  let output_summaries = execute_piranha(&args);
  // Checks if there are any rewrites performed for the file
  assert!(
    output_summaries
      .iter()
      .flat_map(|os| os.rewrites().iter())
      .count()
      > 0
  );

  assert_eq!(output_summaries.len(), n_files_changed);
  let path_to_expected =
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("{path_to_test_ff}/expected"));
  check_result(output_summaries, path_to_expected);
}

/// Checks if the file updates returned by piranha are as expected.
fn check_result(updated_files: Vec<PiranhaOutputSummary>, path_to_expected: PathBuf) {
  let mut all_files_match = true;

  for source_code_unit in &updated_files {
    let updated_file_name = &source_code_unit
      .path()
      .file_name()
      .and_then(|f| f.to_str().map(|x| x.to_string()))
      .unwrap();
    let expected_file_path = find_file(&path_to_expected, updated_file_name);
    let expected_content = read_file(&expected_file_path).unwrap();

    if !eq_without_whitespace(source_code_unit.content(), &expected_content) {
      all_files_match = false;
      error!("{}", &source_code_unit.content());
    }
  }
  assert!(all_files_match);
}
