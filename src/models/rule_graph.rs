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

use derive_builder::Builder;
use getset::Getters;
use itertools::Itertools;

use crate::{
  models::{outgoing_edges::OutgoingEdges, rule::Rule},
  utilities::MapOfVec,
};
use std::collections::HashMap;

use super::default_configs::default_rule_graph;

#[derive(Debug, Default, Getters, Builder)]
#[builder(build_fn(name = "create"))]
pub(crate) struct RuleGraph {
  /// All the rules in the graph
  #[get = "pub(crate)"]
  rules: Vec<Rule>,
  /// Edges of the rule graph
  #[get = "pub(crate)"]
  edges: Vec<OutgoingEdges>,

  /// The graph itself
  #[builder(default = "default_rule_graph()")]
  #[get = "pub(crate)"]
  graph: HashMap<String, Vec<(String, String)>>,
}

impl RuleGraphBuilder {
  pub fn build(&self) -> RuleGraph {
    let _rule_graph = self.create().unwrap();

    let mut graph = HashMap::new();
    // Add the edge(s) to the graph. Multiple edges will be added
    // when either edge endpoint is a group name.
    for edge in _rule_graph.edges() {
      for from_rule in _rule_graph.get_rules_for_group(edge.get_from()) {
        for outgoing_edge in edge.get_to() {
          for to_rule in _rule_graph.get_rules_for_group(outgoing_edge) {
            // Add edge to the adjacency list
            graph.collect(
              from_rule.to_string(),
              (edge.get_scope().to_string(), to_rule.to_string()),
            );
          }
        }
      }
    }

    RuleGraphBuilder::default()
      .edges(_rule_graph.edges().clone())
      .rules(_rule_graph.rules().clone())
      .graph(graph)
      .create()
      .unwrap()
  }
}

impl RuleGraph {
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

  /// Returns a rule named `name` (if found)
  pub(crate) fn get_rule_named(&self, name: &String) -> Option<&Rule> {
    self.rules().iter().find(|x| x.name() == name)
  }

  // Returns rule(s) with name or group as given `group`
  pub(crate) fn get_rules_for_group(&self, group: &String) -> Vec<&String> {
    if let Some(r) = self.get_rule_named(group) {
      return vec![r.name()];
    }
    self
      .rules()
      .iter()
      .filter(|x| x.groups().contains(group))
      .map(|x| x.name())
      .collect_vec()
  }

  pub(crate) fn merge(&self, rule_graph: &RuleGraph) -> Self {
    let all_rules = [rule_graph.rules().clone(), self.rules().clone()].concat();
    let all_edges = [rule_graph.edges().clone(), self.edges().clone()].concat();
    RuleGraphBuilder::default()
      .rules(all_rules)
      .edges(all_edges)
      .build()
  }
}
