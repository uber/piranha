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

use std::collections::HashMap;

use super::{create_match_tests, create_rewrite_tests, substitutions};

use crate::models::default_configs::GO;

create_match_tests! {
  GO,
  test_match_only_for_loop: "structural_find/go_stmt_for_loop", HashMap::from([("find_go_stmt_for_loop", 1)]);
  test_match_only_go_stmt_for_loop:"structural_find/for_loop", HashMap::from([("find_for", 4)]);
  test_match_expression_with_string_literal:"structural_find/expression_with_string_literal", HashMap::from([("match_bool_value", 1)]);
}

create_rewrite_tests! {
  GO,
  test_builtin_boolean_expression_simplify:  "feature_flag/builtin_rules/boolean_expression_simplify", 1,
    substitutions= substitutions! {
      "true_flag_name" => "true",
      "false_flag_name" => "false",
      "nil_flag_name" => "nil"
    };
  test_builtin_statement_cleanup: "feature_flag/builtin_rules/statement_cleanup", 1,
    substitutions= substitutions! {
      "treated" => "true",
      "treated_complement" => "false"
    }, cleanup_comments = true;
  test_const_same_file: "feature_flag/system_1/const_same_file", 1,
    substitutions= substitutions! {
      "stale_flag_name" => "staleFlag",
      "treated" => "false"
    };
}
