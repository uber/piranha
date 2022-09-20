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

use super::{initialize, run_rewrite_test};

static LANGUAGE: &str = "kt";

#[test]
fn test_kotlin_scenarios_treated_ff1() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_1", "treated"),
    2,
  );
}

#[test]
fn test_kotlin_scenarios_treated_ff2() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_2", "treated"),
    4,
  );
}

#[test]
fn test_kotlin_scenarios_control_ff1() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_1", "control"),
    2,
  );
}

#[test]
fn test_kotlin_scenarios_control_ff2() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}/{}", LANGUAGE, "feature_flag_system_2", "control"),
    4,
  );
}


#[test]
fn test_kt_scenarios_file_scoped_chain_rule() {
  initialize();
  run_rewrite_test(
    &format!("{}/{}", LANGUAGE, "file_scoped_chain_rules"),
    1,
  );
}
