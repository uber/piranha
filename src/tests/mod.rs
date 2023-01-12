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

use crate::execute_piranha;
use crate::models::piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder};
use crate::models::piranha_output::PiranhaOutputSummary;
use crate::utilities::{eq_without_whitespace, find_file, read_file};
use log::error;
use std::fs;
use std::path::{Path, PathBuf};

mod test_piranha_java;
mod test_piranha_kt;

mod test_piranha_swift;

mod test_piranha_python;

mod test_piranha_go;
mod test_piranha_ts;
mod test_piranha_tsx;

use std::sync::Once;

static INIT: Once = Once::new();

fn get_piranha_arguments_for_test(
  relative_path_to_tests: &str, language: &str,
) -> PiranhaArguments {
  PiranhaArgumentsBuilder::default()
    .path_to_codebase(format!("test-resources/{relative_path_to_tests}/input/"))
    .path_to_configurations(format!(
      "test-resources/{relative_path_to_tests}/configurations/"
    ))
    .language(language.to_string())
    .dry_run(true)
    .build()
}

fn get_piranha_arguments_for_test_with_substitutions(
  relative_path_to_tests: &str, language: &str, substitutions: Vec<Vec<String>>,
) -> PiranhaArguments {
  PiranhaArgumentsBuilder::default()
    .substitutions(substitutions)
    .build()
    .merge(get_piranha_arguments_for_test(
      relative_path_to_tests,
      language,
    ))
}

fn initialize() {
  INIT.call_once(|| {
    env_logger::init();
  });
}

// Runs a piranha over the target `<relative_path_to_tests>/input` (using configurations `<relative_path_to_tests>/configuration`)
// and checks if the number of matches == `number_of_matches`.
fn run_match_test(piranha_arguments: PiranhaArguments, expected_number_of_matches: usize) {
  print!("{:?}", piranha_arguments);
  let output_summaries = execute_piranha(&piranha_arguments);
  assert_eq!(
    output_summaries
      .iter()
      .flat_map(|os| os.matches().iter())
      .count(),
    expected_number_of_matches
  );
}

// Runs a piranha over the target `<relative_path_to_tests>/input` (using configurations `<relative_path_to_tests>/configuration`)
// and checks if the output of piranha is same as `<relative_path_to_tests>/expected`.
// It also asserts the number of changed files in the expected output.
fn run_rewrite_test(
  piranha_arguments: PiranhaArguments, n_files_changed: usize, relative_path_to_tests: &str,
) {
  print!("Here {:?}", piranha_arguments);

  let output_summaries = execute_piranha(&piranha_arguments);
  // Checks if there are any rewrites performed for the file
  assert!(
    output_summaries
      .iter()
      .flat_map(|os| os.rewrites().iter())
      .count()
      > 0
  );

  assert_eq!(output_summaries.len(), n_files_changed);
  let path_to_expected = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join(format!("test-resources/{relative_path_to_tests}/expected"));
  check_result(output_summaries, path_to_expected);
}

fn copy_folder(src: &Path, dst: &Path) {
  for entry in fs::read_dir(src).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_file() {
      let file_path = dst.join(path.file_name().unwrap());
      let source_code = read_file(&path).unwrap();
      _ = fs::write(file_path.as_path(), source_code);
    }
  }
}

/// Checks if the file updates returned by piranha are as expected.
fn check_result(output_summaries: Vec<PiranhaOutputSummary>, path_to_expected: PathBuf) {
  let mut all_files_match = true;

  for summary in &output_summaries {
    let updated_file_name = &summary
      .path()
      .file_name()
      .and_then(|f| f.to_str().map(|x| x.to_string()))
      .unwrap();
    let expected_file_path = find_file(&path_to_expected, updated_file_name);
    let expected_content = read_file(&expected_file_path).unwrap();

    if !eq_without_whitespace(summary.content(), &expected_content) {
      all_files_match = false;
      error!("{}", &summary.content());
    }
  }
  assert!(all_files_match);
}

macro_rules! create_rewrite_test {
  ($test_name: ident, $piranha_arg: expr, $expected_path: expr, $files_changed: expr ) => {
    #[test]
    fn $test_name() {
      initialize();

      // Copy the test scenario to temporary directory
      let temp_dir = TempDir::new_in(".", "tmp_test").unwrap();
      let temp_dir_path = &temp_dir.path();
      copy_folder(
        Path::new(&$piranha_arg.path_to_codebase()),
        temp_dir_path,
      );

      // Default piranha argument with `path_to_codebase` pointing
      // to `temp_dir`
      let arg_for_codebase_path = PiranhaArgumentsBuilder::default()
          .path_to_codebase(temp_dir_path.to_str().unwrap().to_string())
          .build()

      //Overrides the $piranha_arg parameter's `path_to_code_base` field
      let piranha_arguments = arg_for_codebase_path.merge($piranha_arg);

      let output_summaries = execute_piranha(&piranha_arguments);
      // Checks if there are any rewrites performed for the file
      assert!(output_summaries.iter().any(|x|!x.rewrites().is_empty()));

      assert_eq!(output_summaries.len(), $files_changed);
      let path_to_expected = Path::new(env!("CARGO_MANIFEST_DIR")).join($expected_path);
      check_result(output_summaries, path_to_expected);
      // Delete temp_dir
      _ = temp_dir.close().unwrap();
    }
  };
}

macro_rules! substitutions(
  { $($key:literal => $value:literal),+ } => {
      {
          let mut substitutions: Vec<Vec<String>> = vec![];
          $(
            substitutions.push(vec![$key.to_string(), $value.to_string()]);
          )+
          substitutions
      }
   };
);

pub(crate) use create_rewrite_test;
pub(crate) use substitutions;
