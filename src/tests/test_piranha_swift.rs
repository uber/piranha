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

use super::{check_result, copy_folder, create_rewrite_tests, initialize, substitutions};
use crate::execute_piranha;
use crate::models::{
  default_configs::SWIFT,
  piranha_arguments::{piranha_arguments, PiranhaArgumentsBuilder},
};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

create_rewrite_tests! {
  SWIFT,
  // Tests cascading file delete based on enum and type alias.
  // This scenario is "derived" from plugin cleanup.
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

    test_cleanup_rules_file: "cleanup_rules", 1,
      substitutions = substitutions! {
        "stale_flag" => "stale_flag_one"
      },
      cleanup_comments = true, delete_file_if_empty= false;

}
