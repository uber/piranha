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

use super::execute_piranha_and_check_result;
use crate::{
  constraint,
  models::{
    default_configs::THRIFT, language::PiranhaLanguage, piranha_arguments::piranha_arguments,
    rule_graph::RuleGraphBuilder,
  },
  piranha_rule,
};

#[test]
fn test_consecutive_scope_level_rules() {
  super::initialize();
  let _path = std::path::PathBuf::from("test-resources").join(THRIFT);
  let temp_dir = super::copy_folder_to_temp_dir(&_path.join("input"));

  let rules = vec![piranha_rule! {
      name="Match Exception Definition",
      query="(
            (exception_definition (identifier) @exception_name) @exception_definition 
            (#match? @exception_name \"Internal\")
            )",
          replace_node="exception_definition",
      replace="@exception_definition (
   rpc.code = \"INTERNAL\"
)",
      constraints = [
      constraint! {
        matcher = "(exception_definition) @c_e",
        queries = ["(annotation_definition) @ad",]
      }
    ]

  }];

  let args = piranha_arguments! {
    path_to_codebase = temp_dir.path().to_str().unwrap().to_string(),
    language = PiranhaLanguage::from(THRIFT),
    rule_graph = RuleGraphBuilder::default()
                .rules(rules)
                .build(),
  };

  execute_piranha_and_check_result(&args, _path.join("expected").as_path(), 1, true)
}
