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

use super::{check_result, copy_folder, create_rewrite_tests, initialize, substitutions};
use crate::{
  execute_piranha,
  models::{
    default_configs::JAVA,
    piranha_arguments::{piranha_arguments, PiranhaArgumentsBuilder},
    rule::{piranha_rule, RuleBuilder},
    rule_graph::RuleGraph,
  },
};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

create_rewrite_tests! {
  JAVA,
  test_feature_flag_system_1_treated: "feature_flag_system_1/treated/", 2,
    substitutions = substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    };
  test_feature_flag_system_1_control: "feature_flag_system_1/control", 2,
    substitutions =  substitutions! {
      "stale_flag_name"=> "STALE_FLAG",
      "treated"=> "false",
      "treated_complement" => "true",
      "namespace" => "some_long_name"
    },cleanup_comments = true , delete_file_if_empty= false;
  test_feature_flag_system_2_treated: "feature_flag_system_2/treated", 4,
    substitutions= substitutions! {
      "stale_flag_name" => "STALE_FLAG",
      "treated"=>  "true",
      "treated_complement" => "false",
      "namespace" => "some_long_name"
    }, cleanup_comments = true;
  test_feature_flag_system_2_control: "feature_flag_system_2/control/", 4,
    substitutions = substitutions! {
      "stale_flag_name"=> "STALE_FLAG",
      "treated"=> "false",
      "treated_complement" => "true",
      "namespace" => "some_long_name"
    }, cleanup_comments = true , delete_file_if_empty= false;
  test_scenarios_find_and_propagate:  "find_and_propagate", 2, substitutions = substitutions! {"super_interface_name" => "SomeInterface"},  delete_file_if_empty= false;
  test_non_seed_user_rule:  "non_seed_user_rule", 1, substitutions = substitutions! {"input_type_name" => "ArrayList"};
  test_new_line_character_used_in_string_literal:  "new_line_character_used_in_string_literal",   1;
  test_insert_field_and_initializer:  "insert_field_and_initializer", 1;
  test_consecutive_scope_level_rules: "consecutive_scope_level_rules",1;
}

#[test]
fn test_java_match_only() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("structural_find");
  let path_to_codebase = _path.join("input").to_str().unwrap().to_string();
  let rule = piranha_rule! {
    name= "replace_isToggleEnabled_with_boolean_literal".to_string(),
    query= "((
        (method_invocation 
          name : (_) @name
          arguments: ((argument_list 
                          ([
                            (field_access field: (_)@argument)
                            (_) @argument
                           ])) )
              
        ) @method_invocation
      )
      (#eq? @name \"isToggleEnabled\")
      (#eq? @argument \"STALE_FLAG\")
      )".to_string(),
  };

  let rule_graph = RuleGraph::new(&vec![], &vec![rule]);
  let piranha_arguments = piranha_arguments! {
    path_to_codebase = path_to_codebase,
    rule_graph = rule_graph,
    language= JAVA.to_string(),
  };

  let output_summaries = execute_piranha(&piranha_arguments);
  assert_eq!(
    output_summaries
      .iter()
      .flat_map(|os| os.matches().iter())
      .count(),
    20
  );
}

#[test]
#[should_panic(
  expected = "Could not instantiate the rule Rule { name: \"find_interface_extension\""
)]
fn test_scenarios_find_and_propagate_panic() {
  initialize();
  let _path = PathBuf::from("test-resources")
    .join(JAVA)
    .join("find_and_propagate");
  let path_to_codebase = _path.join("input").to_str().unwrap().to_string();
  let path_to_configurations = _path.join("configurations").to_str().unwrap().to_string();
  let piranha_arguments = piranha_arguments! {
    path_to_codebase = path_to_codebase,
    path_to_configurations = path_to_configurations,
    language= JAVA.to_string(),
  };

  let _ = execute_piranha(&piranha_arguments);
}
