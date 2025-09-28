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

use std::path::PathBuf;

use crate::{
  execute_piranha,
  models::{
    default_configs::TS_SCHEME, language::PiranhaLanguage,
    piranha_arguments::PiranhaArgumentsBuilder,
  },
  tests::initialize,
  utilities::eq_without_whitespace,
};

use super::create_rewrite_tests;

create_rewrite_tests! {
  TS_SCHEME,
  test_simple_rename: "simple_rename", 1;

}

#[test]
fn test_simple_rename_code_snippet() {
  initialize();
  let path_to_scenario = PathBuf::from("test-resources")
    .join(TS_SCHEME)
    .join("simple_rename");
  let code_snippet = "(method_declaration name: (_) @name) @md";
  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .path_to_configurations(
      path_to_scenario
        .join("configurations")
        .to_str()
        .unwrap()
        .to_string(),
    )
    .language(PiranhaLanguage::from(TS_SCHEME))
    .code_snippet(code_snippet.to_string())
    .build();

  let expected = "(method_declaration name: (_) @method_name) @md";
  let output_summaries = execute_piranha(&piranha_arguments);
  assert!(output_summaries.len() == 1);
  assert!(eq_without_whitespace(
    output_summaries[0].content(),
    expected
  ));
  assert!(output_summaries[0].original_content().eq(code_snippet));
}
