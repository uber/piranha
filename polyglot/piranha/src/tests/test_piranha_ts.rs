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

use super::{initialize, run_match_test};

static LANGUAGE: &str = "ts";

#[test]
fn test_ts_match_only_find_fors() {
  initialize();
  run_match_test(&format!("{}/{}/{}", LANGUAGE, "structural_find", "find_fors"), 3);
}

#[test]
fn test_ts_match_only_find_fors_within_functions() {
  initialize();
  run_match_test(&format!("{}/{}/{}", LANGUAGE, "structural_find", "find_fors_within_functions"), 2);
}

#[test]
fn test_ts_match_only_find_fors_within_functions_not_within_whiles() {
  initialize();
  run_match_test(&format!("{}/{}/{}", LANGUAGE, "structural_find", "find_fors_within_functions_not_within_whiles"), 1);
}
