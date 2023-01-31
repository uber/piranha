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

use getset::Getters;

use crate::{
  models::{outgoing_edges::OutgoingEdges, rule::Rule},
  utilities::MapOfVec,
};
use std::collections::HashMap;

#[derive(Debug, Default, Getters)]
pub(crate) struct RuleGraph {
  #[get = "pub(crate)"]
  graph: HashMap<String, Vec<(String, String)>>,
  // All the input rules stored by name
  #[get = "pub(crate)"]
  rules_by_name: HashMap<String, Rule>,
}

impl RuleGraph {
  // Constructs a graph of rules based on the input `edges` that represent the relationship between two rules or groups of rules.
  pub(crate) fn new(edges: &Vec<OutgoingEdges>, all_rules: &Vec<Rule>) -> Self {
    let (rules_by_name, rules_by_group) = Rule::group_rules(all_rules);

    // A closure that gets the rules corresponding to the given rule name or group name.
    let get_rules_for_tag_or_name = |val: &String| {
      rules_by_name
        .get(val)
        .map(|v| vec![v.name()])
        .unwrap_or_else(|| rules_by_group.get(val).cloned().unwrap_or_default())
    };

    let mut graph = HashMap::new();
    // Add the edge(s) to the graph. Multiple edges will be added
    // when either edge endpoint is a group name.
    for edge in edges {
      for from_rule in get_rules_for_tag_or_name(edge.get_from()) {
        for outgoing_edge in edge.get_to() {
          for to_rule in get_rules_for_tag_or_name(outgoing_edge) {
            // Add edge to the adjacency list
            graph.collect(
              from_rule.clone(),
              (String::from(edge.get_scope()), to_rule.clone()),
            );
          }
        }
      }
    }
    RuleGraph {
      graph,
      rules_by_name,
    }
  }

  /// Get all the outgoing edges for `rule_name`
  pub(crate) fn get_neighbors(&self, rule_name: &String) -> Vec<(String, String)> {
    self.graph.get(rule_name).cloned().unwrap_or_default()
  }

  /// Get the number of nodes and edges in the rule graph
  pub(crate) fn get_number_of_rules_and_edges(&self) -> (usize, usize) {
    let mut edges = 0;
    for destinations in self.graph.values() {
      edges += destinations.len();
    }
    (self.graph.len(), edges)
  }
}
