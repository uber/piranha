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

static LANGUAGE: &str = "swift";

// Tests cascading file delete based on enum and type alias. 
// This scenario is "derived" from plugin cleanup.
// This cleanup requires the concept of global tags
#[test]
fn test_cascading_delete_file() {
  initialize();
  run_rewrite_test(&format!("{}/{}",LANGUAGE, "cascade_file_delete"), 3);
}


// Tests cascading file delete based on enum and type alias. 
// This scenario is "derived" from plugin cleanup.
// Checks custom global_tags
#[test]
fn test_cascading_delete_file_custom_global_tag() {
  initialize();
  run_rewrite_test(&format!("{}/{}",LANGUAGE, "cascade_file_delete_custom_global_tag"), 3);
}
