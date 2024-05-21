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

use crate::execute_piranha;
use crate::models::piranha_arguments::PiranhaArguments;
use crate::models::piranha_output::PiranhaOutputSummary;
use crate::utilities::{eq_without_whitespace, read_file};

use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempdir::TempDir;

mod test_piranha_java;
mod test_piranha_kt;

mod test_piranha_swift;

mod test_piranha_python;

mod test_piranha_go;
mod test_piranha_ts;
mod test_piranha_tsx;

mod test_piranha_scala;

mod test_piranha_thrift;

mod test_piranha_ruby;
mod test_piranha_scm;
mod test_piranha_strings;

use std::sync::Once;

static INIT: Once = Once::new();

fn initialize() {
  INIT.call_once(|| {
    env_logger::init();
  });
}

// We use a `.placeholder` file because git does not allow us to commit an empty directory
static PLACEHOLDER: &str = ".placeholder";

/// Copies the files under `src` to `dst`.
/// The copy is NOT recursive.
/// The files under `src` are copied under `dst`.
///
/// # Arguments
///
/// * src: Path to the directory to be copied
/// * dest: Path to destination
///
/// This method causes side effects - writes new files to a directory
fn copy_folder_to_temp_dir(src: &Path) -> TempDir {
  // Copy the test scenario to temporary directory
  let temp_dir = TempDir::new_in(".", "tmp_test").unwrap();
  let temp_dir_path = &temp_dir.path();
  for entry in fs::read_dir(src).unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_file() {
      _ = fs::copy(
        path.to_str().unwrap(),
        temp_dir_path
          .join(path.file_name().unwrap())
          .to_str()
          .unwrap(),
      )
      .unwrap();
    }
  }
  temp_dir
}

fn assert_frequency_for_matches(
  summaries: &[PiranhaOutputSummary], match_freq: &HashMap<&str, u32>,
) {
  let frequencies: HashMap<String, u32> = summaries
    .iter()
    .flat_map(|x| x.matches())
    .into_group_map_by(|x| x.0.clone())
    .into_iter()
    .map(|(k, v)| (k, v.len() as u32))
    .collect();
  for (matched_rule, count) in match_freq.iter() {
    assert_eq!(frequencies[*matched_rule], *count)
  }
}

/// Checks if the file updates returned by piranha are as expected.
fn execute_piranha_and_check_result(
  piranha_arguments: &PiranhaArguments, path_to_expected: &Path, files_changed: usize,
  ignore_whitespace: bool,
) {
  let path_to_codebase = Path::new(piranha_arguments.paths_to_codebase().first().unwrap());
  let output_summaries = execute_piranha(piranha_arguments);
  assert_eq!(output_summaries.len(), files_changed);

  let mut all_files_match = true;

  let count_files = |path: &Path| {
    let mut count = 0;
    for dir_entry in fs::read_dir(path).unwrap().flatten() {
      if dir_entry.path().is_file() {
        // If the directory contains a file named `.placeholder` we ignore that file
        if PLACEHOLDER == dir_entry.file_name().to_str().unwrap() {
          continue;
        }
        count += 1;
      }
    }
    count
  };

  assert_eq!(count_files(path_to_codebase), count_files(path_to_expected));

  for p in fs::read_dir(path_to_codebase).unwrap() {
    let dir_entry = p.unwrap();
    if dir_entry.path().is_file() {
      let path = dir_entry.path();
      let file_name = path.file_name().unwrap();
      let cb_content = read_file(&dir_entry.path().to_path_buf()).unwrap();
      let expected_file_path = path_to_expected.join(file_name);
      let expected_content = read_file(&expected_file_path).unwrap();

      if (ignore_whitespace && eq_without_whitespace(&cb_content, &expected_content))
        || (cb_content
          .trim_end()
          .eq(&expected_content.trim_end().to_string()))
      {
        continue;
      }

      all_files_match = false;
      print!("{}", &cb_content);
    }
  }

  assert!(all_files_match);
}

/// This macro creates a new match test case.
///
/// # Arguments:
/// * test_name: Name of the test (identifier)
/// * relative_path: relative path such that `test-resources/<language>/<relative_path>` leads to a directory containing the folders `input` and `configurations`
/// * matches_frequency: The expected frequency for each match
///
/// Usage:
/// ```
/// create_match_tests! {
///  "java",
///  test_a1:  "relative/path_1", HashMap::from([("match_class", 2));
///  test_a2:  "relative/path_2", HashMap::from([("match_class", 2), ("match_class_1", 1)])
///  
/// ;
/// }
/// ```
macro_rules! create_match_tests {
  ($language: expr,
    $($test_name:ident: $path_to_test: expr,
                        $matches_frequency: expr
                        $(,$kw: ident = $value: expr)* ; )*) => {
    $(
    #[test]
    fn $test_name() {
      super::initialize();
      let _path= std::path::PathBuf::from("test-resources").join($language).join($path_to_test);
      let paths_to_codebase = vec![_path.join("input").to_str().unwrap().to_string()];
      let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
      let piranha_arguments =  $crate::models::piranha_arguments::PiranhaArgumentsBuilder::default()
        .paths_to_codebase(paths_to_codebase)
        .path_to_configurations(path_to_configurations)
        .language($crate::models::language::PiranhaLanguage::from($language))
        $(
          .$kw($value)
        )*
      .build();
      let output_summaries = $crate::execute_piranha(&piranha_arguments);
      super::assert_frequency_for_matches(&output_summaries, &$matches_frequency);
    }
  )*
  };
}

/// This macro creates a new rewrite test case.
///
/// # Arguments:
/// * language: target language
/// * test_name: Name of the test (identifier)
/// * relative_path: relative path such that `test-resources/<language>/<relative_path>` leads to a directory containing the folders `input`, `expected` and `configurations`
/// * files_changed: expression returning the expected number of files changed after the rewriting
///
/// Usage:
/// ```
/// create_rewrite_tests! {
/// "java".to_string(),
///  test_a1:  "relative/path_1", 2;
///  test_a2:  "relative/path_2", 3;
/// }
/// ```
macro_rules! create_rewrite_tests {
  ($language: expr,
    $($test_name:ident: $path_to_test: expr,
                        $files_changed: expr
                        $(,$kw: ident = $value: expr)* ; )*) => {
    $(
    #[test]
    fn $test_name() {
      super::initialize();
      let _path= std::path::PathBuf::from("test-resources").join($language).join($path_to_test);
      let temp_dir= super::copy_folder_to_temp_dir(&_path.join("input"));
      let piranha_arguments =  $crate::models::piranha_arguments::PiranhaArgumentsBuilder::default()
        .paths_to_codebase(vec![temp_dir.path().to_str().unwrap().to_string()])
        .path_to_configurations(_path.join("configurations").to_str().unwrap().to_string())
        .language($crate::models::language::PiranhaLanguage::from($language))
        $(
          .$kw($value)
        )*
      .build();

      super::execute_piranha_and_check_result(&piranha_arguments, &_path.join("expected"), $files_changed, true);
      // Delete temp_dir
      _ = temp_dir.close().unwrap();
    }
  )*
  };
}

/// This macro accepts substitutions as `key` => `value` pairs and transforms it to a `Vec<Vec<String>>`.
///
/// Usage:
/// ```
/// substitutions! {
/// "project" => "Piranha",
/// "language" => "Rust"
/// }
/// ```
///
/// expands to
///
/// ```
/// vec!\[
///      vec!\["project".to_string(), "Piranha".to_string()\]
///      vec!\["language".to_string(), "Rust".to_string()\]
/// \]
/// ```
///
macro_rules! substitutions(
  () =>  { vec![] };
  { $($key:literal => $value:literal),+ } => {
      {
          vec![$(($key.to_string(), $value.to_string()),)+]

      }
   };
);

pub(crate) use create_match_tests;
pub(crate) use create_rewrite_tests;
pub(crate) use substitutions;
