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

use crate::models::default_configs::KOTLIN;

use super::{create_rewrite_tests, substitutions};

create_rewrite_tests! {
  KOTLIN,
  // test_feature_flag_system_1_treated: "feature_flag_system_1/treated", 2, substitutions= substitutions! {
  //   "stale_flag_name" => "STALE_FLAG",
  //   "treated"=>  "true",
  //   "treated_complement" => "false"
  // }, delete_file_if_empty= false;
  // test_feature_flag_system_2_treated: "feature_flag_system_2/treated",4,
  //   substitutions= substitutions! {
  //     "stale_flag_name" => "STALE_FLAG",
  //     "treated"=>  "true",
  //     "treated_complement" => "false",
  //     "namespace" => "some_long_name"
  //   },
  //   cleanup_comments= true;
  // test_feature_flag_system_1_control:  "feature_flag_system_1/control", 2,
  //   substitutions= substitutions! {
  //     "stale_flag_name" => "STALE_FLAG",
  //     "treated"=>  "false",
  //     "treated_complement" => "true"
  //   }, delete_file_if_empty= false;
  // test_feature_flag_system_2_control: "feature_flag_system_2/control", 4,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "false",
  //       "treated_complement" => "true",
  //       "namespace" => "some_long_name"
  //     }, cleanup_comments= true;
  // test_file_scoped_chain_rules: "file_scoped_chain_rules",  1;
  tests_delete: "feature_flag_system_tests/tests_delete/control", 6,
      substitutions= substitutions! {
        "stale_flag_name" => "STALE_FLAG",
        "treated" => "true",
        "untreated" => "false"
      }, cleanup_comments = true;
  // unused_vars_cleanup: "feature_flag_system_tests/unused_vars_cleanup/control", 5,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_simple_enum_control: "feature_flag_system_tests/simple_enum/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_simple_enum_false_control: "feature_flag_system_tests/simple_enum_false/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "false",
  //       "untreated" => "true"
  //     }, cleanup_comments = true;
  // test_string_expresssion_control: "feature_flag_system_tests/string_expressions/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_property_imports_control: "feature_flag_system_tests/property_imports/control", 3,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_imports: "feature_flag_system_tests/imports/control", 2,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_class_string_constant_control: "feature_flag_system_tests/class_string_constant/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_identifiers_control: "feature_flag_system_tests/identifiers/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_expressions_control: "feature_flag_system_tests/expressions/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_infix_expressions_control: "feature_flag_system_tests/infix_expressions/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_sealed_class_control: "feature_flag_system_tests/sealed_class/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_sealed_class_parameterized_control: "feature_flag_system_tests/sealed_class_parameterized/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_simple_parameterized_enum_control: "feature_flag_system_tests/simple_parameterized_enum/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_simple_parameterized_enum_with_enum_control: "feature_flag_system_tests/simple_parameterized_enum_with_enum/control", 3,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_simple_var_assignments_control: "feature_flag_system_tests/var_assignments/control", 2,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_class_feature_object_control: "feature_flag_system_tests/class_feature_object/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  // test_lambdas_control: "feature_flag_system_tests/lambdas/control", 1,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true, allow_dirty_ast = true;
  //

  // todo: not used anymore
  // tests_cleanup: "feature_flag_system_tests/tests_cleanup/control", 6,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated" => "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;

  // todo: not used anymore
  // unused_vars_delete: "feature_flag_system_tests/unused_vars_delete/control", 4,
  //     substitutions= substitutions! {
  //       "stale_flag_name" => "STALE_FLAG",
  //       "treated"=>  "true",
  //       "untreated" => "false"
  //     }, cleanup_comments = true;
  //
}
