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

use crate::{
  models::{default_configs::JAVA, language::PiranhaLanguage},
  tests::substitutions,
};

use super::PiranhaArgumentsBuilder;

#[test]
#[should_panic(expected = "Invalid Piranha Argument. Missing `path_to_codebase` or `code_snippet`")]
fn piranha_argument_invalid_no_codebase_and_snippet() {
  let _ = PiranhaArgumentsBuilder::default()
    .path_to_configurations("some/path".to_string())
    .language(PiranhaLanguage::from(JAVA))
    .substitutions(substitutions! {"super_interface_name" => "SomeInterface"})
    .build();
}

#[test]
#[should_panic(
  expected = "Invalid Piranha arguments. Please either specify the `path_to_codebase` or the `code_snippet`. Not Both."
)]
fn piranha_argument_invalid_both_codebase_and_snippet() {
  let _ = PiranhaArgumentsBuilder::default()
    .path_to_configurations("some/path".to_string())
    .paths_to_codebase(vec!["dev/null".to_string()])
    .code_snippet("class A { }".to_string())
    .language(PiranhaLanguage::from(JAVA))
    .substitutions(substitutions! {"super_interface_name" => "SomeInterface"})
    .build();
}

#[test]
fn piranha_argument_with_custom_builtin_rules() {
  let path_to_custom_builtin_rules = format!(
    "{}/testdata/custom_builtin/",
    std::path::Path::new(file!()).parent().unwrap().display()
  );
  let args = PiranhaArgumentsBuilder::default()
    .path_to_configurations("some/path".to_string())
    .paths_to_codebase(vec!["dev/null".to_string()])
    .path_to_custom_builtin_rules(path_to_custom_builtin_rules.clone())
    .language(PiranhaLanguage::from(JAVA))
    .substitutions(substitutions! {"super_interface_name" => "SomeInterface"})
    .build();
  assert_eq!(args.rule_graph().get_number_of_rules_and_edges(), (2, 2));
  assert_eq!(
    args.path_to_custom_builtin_rules(),
    &path_to_custom_builtin_rules
  );
}
