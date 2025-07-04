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

use crate::df::analysis::DataflowAnalysis;
use crate::df::analysis::Direction;
use crate::df::tag_analysis::{DefiniteAssignmentSigma, ForwardDefiniteAssignment};

use crate::models::rule::Rule;
use crate::{edges, models::rule_graph::RuleGraphBuilder, piranha_rule};
use std::collections::HashSet;

fn check_sigma_in(
  analysis: &DataflowAnalysis<ForwardDefiniteAssignment>, rule: &Rule, expected_vars: Vec<&str>,
) {
  let sigma = analysis.sigma_in().get(rule).unwrap().variables.clone();
  let expected: HashSet<_> = expected_vars.into_iter().map(|s| s.to_string()).collect();
  assert_eq!(sigma, expected);
}

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
      query = "rgx (?P<type>void|String)\\s+.*bar\\b",
      holes = ["class_name"]
    },
    piranha_rule! {
      name = "detect_methods_same_type",
      query = "(
      (method_declaration
         type: (_) @other_type
      )
      (#eq? @other_type @type)
      )",
      holes = ["type"]
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
      to = ["detect_methods_same_type"],
      scope = "Class"
    ),
  ];

  let graph = RuleGraphBuilder::default()
    .rules(rules.clone())
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment {
    graph,
    initial_substitutions: HashSet::new(),
  };
  let mut analysis = DataflowAnalysis::new(forward);

  // Get the rules in post order for optimal performance of the analysis
  let mut rules_post_order = rules.clone();
  rules_post_order.reverse();
  // The entry point of the rule graph
  let entry_rule = &rules[0];
  analysis.run_analysis(rules_post_order, vec![entry_rule.clone()]);

  // Call the inner function to check the sigma_in of each rule
  check_sigma_in(&analysis, &rules[0], vec![]);
  check_sigma_in(&analysis, &rules[1], vec!["@class_name"]);
  check_sigma_in(&analysis, &rules[2], vec!["@class_name", "@type"]);
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

  let forward = ForwardDefiniteAssignment {
    graph,
    initial_substitutions: HashSet::new(),
  };
  let mut analysis = DataflowAnalysis::new(forward);

  // Get the rules in post order for optimal performance of the analysis
  let mut rules_post_order = rules.clone();
  rules_post_order.reverse();
  // The entry point of the rule graph
  let entry_rule = &rules[0];
  analysis.run_analysis(rules_post_order, vec![entry_rule.clone()]);

  // Call the inner function to check the sigma_in of each rule
  check_sigma_in(&analysis, &rules[0], vec![]);
  check_sigma_in(
    &analysis,
    &rules[1],
    vec![
      "@class_name",
      "@class_members",
      "@class_body",
      "@class_declaration",
    ],
  );
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

  let forward = ForwardDefiniteAssignment {
    graph,
    initial_substitutions: HashSet::new(),
  };

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

  let forward = ForwardDefiniteAssignment {
    graph,
    initial_substitutions: HashSet::new(),
  };

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_bottom: false,
  };

  let new_sigma = forward.flow(&rule, &sigma);
  assert_eq!(new_sigma.variables.len(), 0);
}

#[test]
fn test_flow_function_rgx_2() {
  let rule = piranha_rule! {
    name = "replace_call_def_fed",
    query = "rgx (?P<n>abc\\(\\)\\.(?P<m_def>def)\\(\\)\\.ghi\\(\\))"
  };

  let edges = vec![];

  let graph = RuleGraphBuilder::default()
    .rules(vec![rule.clone()])
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment {
    graph,
    initial_substitutions: HashSet::new(),
  };

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_bottom: false,
  };

  let new_sigma = forward.flow(&rule, &sigma);
  // Convert HashSet to Vec and sort it
  let mut variables: Vec<_> = new_sigma.variables.into_iter().collect();
  variables.sort();

  // Assert there are two strings in the set @n and @m_def, check the strings
  assert_eq!(variables, vec!["@m_def", "@n"]);
}

#[test]
fn test_flow_function_rgx_0() {
  let rule = piranha_rule! {
    name = "replace_call_def_fed",
    query = "rgx @Something",
    holes = ["Something"]
  };

  let edges = vec![];

  let graph = RuleGraphBuilder::default()
    .rules(vec![rule.clone()])
    .edges(edges)
    .build();

  let forward = ForwardDefiniteAssignment {
    graph,
    initial_substitutions: HashSet::new(),
  };

  let sigma = DefiniteAssignmentSigma {
    variables: HashSet::new(),
    is_bottom: false,
  };

  let new_sigma = forward.flow(&rule, &sigma);
  assert_eq!(new_sigma.variables.len(), 0);
}
