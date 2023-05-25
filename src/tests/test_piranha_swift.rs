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

use std::path::PathBuf;

use super::{create_rewrite_tests, execute_piranha_and_check_result, substitutions};

use crate::models::{
  default_configs::SWIFT, language::PiranhaLanguage, piranha_arguments::PiranhaArgumentsBuilder,
};

create_rewrite_tests! {
  SWIFT,
  // This scenario is "derived" from plugin cleanup.
  // Tests cascading file delete based on enum and type alias.
  // This cleanup requires the concept of global tags
  test_cascading_delete_file:  "cascade_file_delete", 3,
    substitutions = substitutions! {
      "stale_flag_name" => "Premium-Icon"
    },
    cleanup_comments = true, delete_file_if_empty= false;
  test_cascading_delete_file_custom_global_tag: "cascade_file_delete_custom_global_tag", 3,
    substitutions = substitutions! {
    "stale_flag_name" => "Premium-Icon"
    },
    cleanup_comments = true,
    global_tag_prefix ="universal_tag.".to_string(),
    cleanup_comments_buffer = 3, delete_file_if_empty= false;
  test_leading_comma: "leading_comma", 1,
    substitutions = substitutions! {
      "stale_flag" => "one"
    },
    cleanup_comments = true, delete_file_if_empty= false;
}

fn execute_piranha_with_default_swift_args(scenario: &str, substitutions: Vec<(String, String)>) {
  let _path = PathBuf::from("test-resources").join(SWIFT).join(scenario);
  let temp_dir = super::copy_folder_to_temp_dir(&_path.join("input"));
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .path_to_codebase(temp_dir.path().to_str().unwrap().to_string())
    .path_to_configurations(_path.join("configurations").to_str().unwrap().to_string())
    .language(PiranhaLanguage::from(SWIFT))
    .cleanup_comments(true)
    .substitutions(substitutions)
    .delete_file_if_empty(false)
    .build();
  execute_piranha_and_check_result(&piranha_arguments, &_path.join("expected"), 1, true);
  temp_dir.close().unwrap();
}

#[test]
#[ignore] // Long running test
fn test_cleanup_rules_file() {
  super::initialize();
  execute_piranha_with_default_swift_args(
    "cleanup_rules",
    substitutions! {
      "stale_flag" => "stale_flag_one",
      "treated" => "true",
      "treated_complement" => "false"
    },
  );
}

#[test]
#[ignore] // Long running test
fn test_local_variable_inline_file() {
  super::initialize();
  execute_piranha_with_default_swift_args("variable_inline/local_variable_inline", vec![]);
}

#[test]
#[ignore] // Long running test
fn test_field_variable_inline_file() {
  super::initialize();
  execute_piranha_with_default_swift_args("variable_inline/field_variable_inline", vec![]);
}

#[test]
#[ignore] // Long running test
fn test_adhoc_variable_inline_file() {
  super::initialize();
  execute_piranha_with_default_swift_args("variable_inline/adhoc_variable_inline", vec![]);
}

#[test]
#[ignore] // Long running test
fn test_delete_everything_after_return() {
  super::initialize();
  execute_piranha_with_default_swift_args("delete_statements_after_return", vec![]);
}
