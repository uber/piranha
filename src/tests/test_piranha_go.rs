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
  default_configs::GO,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
};
use std::path::Path;
use tempdir::TempDir;

fn match_only_go_stmt_for_loop() -> PiranhaArguments {
  PiranhaArguments::new(
    GO,
    "test-resources/go/structural_find/go_stmt_for_loop/input/",
    "test-resources/go/structural_find/go_stmt_for_loop/configurations",
  )
}

fn match_only_for_loop() -> PiranhaArguments {
  PiranhaArguments::new(
    GO,
    "test-resources/go/structural_find/for_loop/input/",
    "test-resources/go/structural_find/for_loop/configurations",
  )
}

create_match_test! {
  test_match_only_for_loop: match_only_for_loop(), 4,
  test_match_only_go_stmt_for_loop: match_only_go_stmt_for_loop(), 1,
}

fn builtin_boolean_expression_simplify() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    GO,
    "test-resources/go/feature_flag/builtin_rules/boolean_expression_simplify/input",
    "test-resources/go/feature_flag/builtin_rules/boolean_expression_simplify/configurations",
    substitutions! {
      "true_flag_name" => "true",
      "false_flag_name" => "false",
      "nil_flag_name" => "nil"
    },
  )
}

fn builtin_statement_cleanup() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    GO,
    "test-resources/go/feature_flag/builtin_rules/statement_cleanup/input",
    "test-resources/go/feature_flag/builtin_rules/statement_cleanup/configurations",
    substitutions! {
      "treated" => "true",
      "treated_complement" => "false"
    },
  )
}

fn const_same_file() -> PiranhaArguments {
  PiranhaArguments::new_substitutions(
    GO,
    "test-resources/go/feature_flag/system_1/const_same_file/input",
    "test-resources/go/feature_flag/system_1/const_same_file/configurations",
    substitutions! {
      "stale_flag_name" => "staleFlag",
      "treated" => "false"
    },
  )
}

create_rewrite_test! {
  test_builtin_boolean_expression_simplify: builtin_boolean_expression_simplify(), "test-resources/go/feature_flag/builtin_rules/boolean_expression_simplify/expected", 1,
  test_builtin_statement_cleanup: builtin_statement_cleanup(), "test-resources/go/feature_flag/builtin_rules/statement_cleanup/expected", 1,
  test_const_same_file: const_same_file(), "test-resources/go/feature_flag/system_1/const_same_file/expected", 1,
}
