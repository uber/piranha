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
use crate::df::tag_analysis::ForwardDefiniteAssignment;
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
  let entry_rule = rules.get(0).unwrap().clone();
  analysis.run_analysis(rules_post_order, entry_rule);
}
