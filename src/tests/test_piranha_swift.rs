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

use crate::models::{default_configs::SWIFT, piranha_arguments::PiranhaArgumentsBuilder};

use super::{get_piranha_arguments_for_test_with_substitutions, initialize, run_rewrite_test};

// Tests cascading file delete based on enum and type alias.
// This scenario is "derived" from plugin cleanup.
// This cleanup requires the concept of global tags
#[test]
fn test_cascading_delete_file() {
  initialize();

  let relative_path_to_tests = &format!("{}/{}", SWIFT, "cascade_file_delete");
  let substitutions = vec![vec![
    "stale_flag_name".to_string(),
    "Premium-Icon".to_string(),
  ]];
  let piranha_argument = PiranhaArgumentsBuilder::default()
    .cleanup_comments(true)
    .build()
    .merge(get_piranha_arguments_for_test_with_substitutions(
      relative_path_to_tests,
      SWIFT,
      substitutions,
    ));

  run_rewrite_test(piranha_argument, 3, relative_path_to_tests);

  // initialize();
  // run_rewrite_test(, 3);
}

// Tests cascading file delete based on enum and type alias.
// This scenario is "derived" from plugin cleanup.
// Checks custom global_tags
#[test]
fn test_cascading_delete_file_custom_global_tag() {
  initialize();

  let relative_path_to_tests = &format!("{}/{}", SWIFT, "cascade_file_delete_custom_global_tag");
  let substitutions = vec![vec![
    "stale_flag_name".to_string(),
    "Premium-Icon".to_string(),
  ]];
  let piranha_argument = PiranhaArgumentsBuilder::default()
    .cleanup_comments(true)
    .global_tag_prefix("universal_tag.".to_string())
    .cleanup_comments_buffer(3)
    .build()
    .merge(get_piranha_arguments_for_test_with_substitutions(
      relative_path_to_tests,
      SWIFT,
      substitutions,
    ));

  run_rewrite_test(piranha_argument, 3, relative_path_to_tests);
}
