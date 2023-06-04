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

use crate::{edges, models::rule_graph::RuleGraphBuilder, piranha_rule};
use std::collections::HashSet;

#[test]
fn test_graph_2edges() {
  let rules = vec![
    piranha_rule! {
      name = "detect_class_foo",
      query = "(
      (class_declaration
          name: (identifier) @class_name
      )
      (#eq? @class_name \"Foo\")
    )"
    },
    piranha_rule! {
      name = "detect_method_bar_in_foo",
      query = "(
      (method_declaration
          name: (identifier) @method_name
          . (class_declaration
                name: (_) @detected_class_name
             )
      )
      (#eq? @detected_class_name @class_name)
      (#eq? @method_name \"bar\")
    )"
    },
    piranha_rule! {
      name = "detect_baz_in_bar",
      query = "(
      (call_expression
          function: (identifier) @function_name
          . (method_declaration
                name: (_) @detected_method_name
             )
      )
      (#eq? @detected_method_name @method_name)
      (#eq? @function_name \"baz\")
    )"
    },
  ];

  let edges = vec![
    edges! {
      from = "detect_class_foo",
      to = ["detect_method_bar_in_foo"],
      scope = "Class"
    },
    edges!(
      from = "detect_method_bar_in_foo",
      to = ["detect_baz_in_bar"],
      scope = "Class"
    ),
  ];

  let graph = RuleGraphBuilder::default()
    .rules(rules.clone())
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment::new(graph, HashSet::new());
  let mut analysis = DataflowAnalysis::new(forward);

  // Get the rules in post order for optimal performance of the analysis
  let mut rules_post_order = rules.clone();
  rules_post_order.reverse();
  // The entry point of the rule graph
  let entry_rule = &rules[0];
  analysis.run_analysis(rules_post_order, vec![entry_rule.clone()]);

  // Check the sigma in of the 2nd rule
  let sigma = analysis
    .sigma_in()
    .get(&rules[2])
    .unwrap()
    .variables
    .clone();
  let expected = vec!["@method_name", "@class_name", "@detected_class_name"]
    .into_iter()
    .map(|s| s.to_string())
    .collect::<HashSet<String>>();
  assert_eq!(sigma, expected);
}

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

  let forward = ForwardDefiniteAssignment::new(graph, HashSet::new());
  let mut analysis = DataflowAnalysis::new(forward);

  // Get the rules in post order for optimal performance of the analysis
  let mut rules_post_order = rules.clone();
  rules_post_order.reverse();
  // The entry point of the rule graph
  let entry_rule = &rules[0];
  analysis.run_analysis(rules_post_order, vec![entry_rule.clone()]);

  // Check the sigma in of the 2nd rule
  let sigma = analysis
    .sigma_in()
    .get(&rules[1])
    .unwrap()
    .variables
    .clone();
  let expected = vec![
    "@class_name",
    "@class_members",
    "@class_body",
    "@class_declaration",
  ]
  .into_iter()
  .map(|s| s.to_string())
  .collect::<HashSet<String>>();
  assert_eq!(sigma, expected);
}

#[test]
fn test_flow_function() {
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

  let edges = vec![];

  let graph = RuleGraphBuilder::default()
    .rules(vec![rule.clone()])
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment::new(graph, HashSet::new());

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_bottom: false,
  };

  let new_sigma = forward.flow(&rule, &sigma);
  assert_eq!(new_sigma.variables.len(), 4);
}

#[test]
fn test_flow_function_0() {
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

  let edges = vec![];

  let graph = RuleGraphBuilder::default()
    .rules(vec![rule.clone()])
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment::new(graph, HashSet::new());

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_bottom: false,
  };

  let new_sigma = forward.flow(&rule, &sigma);
  assert_eq!(new_sigma.variables.len(), 0);
}
