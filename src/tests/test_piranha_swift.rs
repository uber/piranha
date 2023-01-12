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

use super::{check_result, copy_folder, create_rewrite_test, initialize, substitutions};
use crate::execute_piranha;
use crate::models::{
  default_configs::SWIFT,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
};
use std::path::Path;
use tempdir::TempDir;

fn cascading_delete_file() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    SWIFT,
    "test-resources/swift/cascade_file_delete/input",
    "test-resources/swift/cascade_file_delete/configurations",
    substitutions! {
      "stale_flag_name" => "Premium-Icon"
    },
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .cleanup_comments(true)
      .build(),
  )
}

fn cascading_delete_file_custom_global_tag() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    SWIFT,
    "test-resources/swift/cascade_file_delete_custom_global_tag/input",
    "test-resources/swift/cascade_file_delete_custom_global_tag/configurations",
    substitutions! {
      "stale_flag_name" => "Premium-Icon"
    },
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .cleanup_comments(true)
      .global_tag_prefix("universal_tag.".to_string())
      .cleanup_comments_buffer(3)
      .build(),
  )
}

create_rewrite_test! {
  // Tests cascading file delete based on enum and type alias.
  // This scenario is "derived" from plugin cleanup.
  // This cleanup requires the concept of global tags
  test_cascading_delete_file: cascading_delete_file(), "test-resources/swift/cascade_file_delete/expected", 3,
  test_cascading_delete_file_custom_global_tag: cascading_delete_file_custom_global_tag(), "test-resources/swift/cascade_file_delete_custom_global_tag/expected", 3,
}
