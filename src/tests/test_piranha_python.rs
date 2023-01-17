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
  check_result, copy_folder, create_match_tests, create_rewrite_tests, initialize, substitutions,
};
use crate::execute_piranha;
use crate::models::{
  default_configs::PYTHON,
  piranha_arguments::{piranha_arguments, PiranhaArgumentsBuilder},
};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

create_rewrite_tests!(
  PYTHON,
  test_delete_modify_str_literal_from_list:  "delete_cleanup_str_in_list", 1,
  substitutions = substitutions! {
    "str_literal" => "dependency2",
    "str_to_replace"=>  "dependency1",
    "str_replacement" => "dependency1_1"
  };
);

create_match_tests!(PYTHON, test_match_only: "structural_find", 3;);
