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
  test_feature_flag_system_1_treated: "feature_flag_system_1/treated", 2, substitutions= substitutions! {
    "stale_flag_name" => "STALE_FLAG",
    "treated"=>  "true",
    "treated_complement" => "false"
  }, delete_file_if_empty= false;
  test_feature_flag_system_2_treated: "feature_flag_system_2/treated",4,
    substitutions= substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    },
    cleanup_comments= true;

  test_feature_flag_system_1_control:  "feature_flag_system_1/control", 2,
    substitutions= substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "false",
      "treated_complement" => "true"
    }, delete_file_if_empty= false;
  test_feature_flag_system_2_control: "feature_flag_system_2/control", 4,
      substitutions= substitutions! {
        "stale_flag_name" => "STALE_FLAG",
        "treated"=>  "false",
        "treated_complement" => "true",
        "namespace" => "some_long_name"
      }, cleanup_comments= true;
  test_file_scoped_chain_rules: "file_scoped_chain_rules",  1;
}
