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

use super::{
  check_result, copy_folder, create_match_test, create_rewrite_test, initialize, substitutions,
};
use crate::execute_piranha;
use crate::models::{
  default_configs::PYTHON,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
};
use std::path::Path;
use tempdir::TempDir;

fn delete_modify_str_literal_from_list() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    PYTHON,
    "test-resources/python/delete_cleanup_str_in_list/input",
    "test-resources/python/delete_cleanup_str_in_list/configurations",
    substitutions! {
      "str_literal" => "dependency2",
      "str_to_replace"=>  "dependency1",
      "str_replacement" => "dependency1_1"
    },
  )
}

create_rewrite_test!(
  test_delete_modify_str_literal_from_list: delete_modify_str_literal_from_list(),
  "test-resources/python/delete_cleanup_str_in_list/expected",
  1,
);

fn match_only() -> PiranhaArguments {
  PiranhaArguments::new(
    PYTHON,
    "test-resources/python/structural_find/input/",
    "test-resources/python/structural_find/configurations",
  )
}

create_match_test!(test_match_only: match_only(), 3,);
