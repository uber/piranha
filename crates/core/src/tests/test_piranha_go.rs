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



#[test]
fn test_package_scope_delete_method() {
  super::initialize();
  let _path= std::path::PathBuf::from("test-resources").join(GO).join("feature_flag/delete_method/private_pkg_fn");
  let paths_to_codebase = vec![_path.join("input").to_str().unwrap().to_string()];
  let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
  let piranha_arguments =  crate::models::piranha_arguments::PiranhaArgumentsBuilder::default()
      .paths_to_codebase(paths_to_codebase)
      .path_to_configurations(path_to_configurations)
      .language(crate::models::language::PiranhaLanguage::from(GO))
      .build();
  let output_summaries = crate::execute_piranha(&piranha_arguments);
  
  // Check if expected files match transformed files
  use std::fs;
  use crate::utilities::read_file;
  
  let expected_dir = _path.join("expected");
  let input_dir = _path.join("input");
  
  for entry in fs::read_dir(&expected_dir).unwrap() {
    let entry = entry.unwrap();
    if entry.path().is_file() {
      let file_name = entry.file_name();
      let expected_content = read_file(&entry.path()).unwrap();
      let actual_file = input_dir.join(&file_name);
      
      if actual_file.exists() {
        let actual_content = read_file(&actual_file).unwrap();
        assert_eq!(
          actual_content.trim(), 
          expected_content.trim(),
          "File {:?} doesn't match expected output", file_name
        );
      }
    }
  }
  
  println!("output_summaries: {:?}", output_summaries);
}
