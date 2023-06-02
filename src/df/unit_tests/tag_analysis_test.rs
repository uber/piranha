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

use crate::df::df::DataflowAnalysis;
use crate::df::df::Direction;
use crate::df::tag_analysis::{DefiniteAssignmentSigma, ForwardDefiniteAssignment};
use crate::models::rule::RuleBuilder;
use glob::Pattern;

use crate::{
  edges, execute_piranha, filter,
  models::{
    default_configs::JAVA, language::PiranhaLanguage, piranha_arguments::PiranhaArgumentsBuilder,
    rule_graph::RuleGraphBuilder,
  },
  piranha_rule,
  utilities::eq_without_whitespace,
};
use std::collections::HashSet;
use std::{collections::HashMap, path::PathBuf};

#[test]
fn test_forward_analysis_simple() {
  let rules = vec![
    piranha_rule! {
      name = "add_inner_class",
      query = "(
        (class_declaration name: (_)@class_name
            body : (class_body ((_)*) @class_members)  @class_body
        ) @class_declaration
        (#eq? @class_name \"FooBar\")
        )"
    },
    piranha_rule! {
    name = "add_field_declaration",
    query = "(
        (class_declaration name: (_)@class_name
            body : (class_body ((_)*) @class_members)  @class_body
         ) @class_declaration
        )"
      },
  ];

  let edges = vec![edges! {
    from = "add_inner_class",
    to = ["add_field_declaration"],
    scope = "Class"
  }];

  let graph = RuleGraphBuilder::default()
    .rules(rules.clone())
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment { graph };
  let mut analysis = DataflowAnalysis::new(forward);

  // Get the rules in post order for optimal performance of the analysis
  let mut rules_post_order = rules.clone();
  rules_post_order.reverse();
  // The entry point of the rule graph
  let entry_rule = &rules[0];
  analysis.run_analysis(rules_post_order, entry_rule.clone());

  // Check the sigma in of the 2nd rule
  let sigma = analysis.sigma_in().get(&rules[1]).unwrap().variables.clone();
  let expected = vec!["@class_name", "@class_members", "@class_body", "@class_declaration"]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<HashSet<String>>();
  assert_eq!(sigma, expected);
}

#[test]
fn test_transfer_function() {
  let rule = piranha_rule! {
    name = "add_inner_class",
    query = "(
        (class_declaration name: (_)@class_name
            body : (class_body ((_)*) @class_members) @class_body
        ) @class_declaration
        (#eq? @class_name_x \"@other_undefined_variable\")
        (#eq? @class_name_1 \"@_undefined_variable\")
        )"
  };

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_top: false,
  };

  let new_sigma = ForwardDefiniteAssignment::transfer(&rule, &sigma);
  assert_eq!(new_sigma.variables.len(), 4);
}

#[test]
fn test_transfer_function_0() {
  let rule = piranha_rule! {
    name = "add_inner_class",
    query = "(
        (class_declaration name: (_)
            body : (class_body ((_)*))
        )
        (#eq? @class_name_x \"@other_undefined_variable\")
        (#eq? @class_name_1 \"@_undefined_variable\")
        )"
  };

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_top: false,
  };

  let new_sigma = ForwardDefiniteAssignment::transfer(&rule, &sigma);
  assert_eq!(new_sigma.variables.len(), 0);
}
