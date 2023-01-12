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

use crate::models::{
  default_configs::KOTLIN,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
};

use super::{
  check_result, copy_folder, create_rewrite_test, initialize, substitutions, PiranhaArguments::new,
};
use crate::execute_piranha;

use std::path::Path;
use tempdir::TempDir;

fn feature_flag_system_1_treated() -> PiranhaArguments {
  PiranhaArguments::new(
    KOTLIN,
    "test-resources/kt/feature_flag_system_1/treated/input",
    "test-resources/kt/feature_flag_system_1/treated/configurations",
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .substitutions(substitutions! {
        "stale_flag_name" => "STALE_FLAG",
        "treated"=>  "true",
        "treated_complement" => "false"
      })
      .build(),
  )
}

fn feature_flag_system_1_control() -> PiranhaArguments {
  PiranhaArguments::new(
    KOTLIN,
    "test-resources/kt/feature_flag_system_1/control/input",
    "test-resources/kt/feature_flag_system_1/control/configurations",
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .substitutions(substitutions! {
        "stale_flag_name" => "STALE_FLAG",
        "treated"=>  "false",
        "treated_complement" => "true"
      })
      .build(),
  )
}

fn feature_flag_system_2_treated() -> PiranhaArguments {
  PiranhaArguments::new(
    KOTLIN,
    "test-resources/kt/feature_flag_system_2/treated/input",
    "test-resources/kt/feature_flag_system_2/treated/configurations",
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .substitutions(substitutions! {
        "stale_flag_name" => "STALE_FLAG",
        "treated"=>  "true",
        "treated_complement" => "false",
        "namespace" => "some_long_name"
      })
      .cleanup_comments(true)
      .build(),
  )
}

fn feature_flag_system_2_control() -> PiranhaArguments {
  PiranhaArguments::new(
    KOTLIN,
    "test-resources/kt/feature_flag_system_2/control/input",
    "test-resources/kt/feature_flag_system_2/control/configurations",
  )
  .merge(
    PiranhaArgumentsBuilder::default()
      .substitutions(substitutions! {
        "stale_flag_name" => "STALE_FLAG",
        "treated"=>  "false",
        "treated_complement" => "true",
        "namespace" => "some_long_name"
      })
      .cleanup_comments(true)
      .build(),
  )
}

fn file_scoped_chain_rules() -> PiranhaArguments {
  PiranhaArguments::new(
    KOTLIN,
    "test-resources/kt/file_scoped_chain_rules/input",
    "test-resources/kt/file_scoped_chain_rules/configurations",
  )
}

create_rewrite_test!(
  test_feature_flag_system_1_treated,
  feature_flag_system_1_treated(),
  "test-resources/kt/feature_flag_system_1/treated/expected",
  2
);

create_rewrite_test!(
  test_feature_flag_system_2_treated,
  feature_flag_system_2_treated(),
  "test-resources/kt/feature_flag_system_2/treated/expected",
  4
);

create_rewrite_test!(
  test_feature_flag_system_1_control,
  feature_flag_system_1_control(),
  "test-resources/kt/feature_flag_system_1/control/expected",
  2
);

create_rewrite_test!(
  test_feature_flag_system_2_control,
  feature_flag_system_2_control(),
  "test-resources/kt/feature_flag_system_2/control/expected",
  4
);

create_rewrite_test!(
  test_file_scoped_chain_rules,
  file_scoped_chain_rules(),
  "test-resources/kt/file_scoped_chain_rules/expected",
  1
);
