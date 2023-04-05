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

use derive_builder::Builder;
use getset::{Getters, MutGetters};
use itertools::Itertools;

use crate::{
  models::{outgoing_edges::OutgoingEdges, rule::Rule},
  utilities::{gen_py_str_methods, read_toml, MapOfVec},
};
use std::{collections::HashMap, path::Path};

use super::{
  default_configs::{default_edges, default_rule_graph_map, default_rules},
  outgoing_edges::Edges,
  rule::{InstantiatedRule, Rules},
};
use pyo3::prelude::{pyclass, pymethods};

pub(crate) static GLOBAL: &str = "Global";
pub(crate) static PARENT: &str = "Parent";

#[derive(Debug, Default, Getters, MutGetters, Builder, Clone, PartialEq)]
#[builder(build_fn(name = "create"))]
#[pyclass]
pub struct RuleGraph {
  /// All the rules in the graph
  #[get_mut = "pub(crate)"]
  #[get = "pub(crate)"]
  #[pyo3(get)]
  #[builder(default = "default_rules()")]
  rules: Vec<Rule>,
  /// Edges of the rule graph
  #[get = "pub(crate)"]
  #[builder(default = "default_edges()")]
  #[pyo3(get)]
  edges: Vec<OutgoingEdges>,

  /// The graph itself
  #[builder(default = "default_rule_graph_map()")]
  #[get = "pub(crate)"]
  #[pyo3(get)]
  graph: HashMap<String, Vec<(String, String)>>,
}

#[pymethods]
impl RuleGraph {
  #[new]
  fn py_new(rules: Vec<Rule>, edges: Vec<OutgoingEdges>) -> Self {
    RuleGraphBuilder::default()
      .rules(rules)
      .edges(edges)
      .build()
  }
  gen_py_str_methods!();
}

impl RuleGraphBuilder {
  /// Build the rule graph.
  pub fn build(&self) -> RuleGraph {
    let _rule_graph = self.create().unwrap();

    let mut graph = HashMap::new();

    for r in _rule_graph.rules() {
      graph.insert(r.name().to_string(), vec![]);
    }
    // Add the edge(s) to the graph. Multiple edges will be added
    // when either edge endpoint is a group name.
    for edge in _rule_graph.edges() {
      for from_rule in _rule_graph.get_rules_for_group(edge.get_frm()) {
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

  /// Get the next rules to be applied grouped by the scope in which they should be performed.
  pub(crate) fn get_next(
    &self, rule_name: &String, tag_matches: &HashMap<String, String>,
  ) -> HashMap<String, Vec<InstantiatedRule>> {
    // let rule_name = rule.name();
    let mut next_rules: HashMap<String, Vec<InstantiatedRule>> = HashMap::new();
    // Iterate over each entry (Edge) in the adjacency list corresponding to `rule_name`
    for (scope, to_rule) in self.get_neighbors(rule_name) {
      let to_rule_name = &self.get_rule_named(&to_rule).unwrap();
      // If the to_rule_name is a dummy rule, skip it and rather return it's next rules.
      if to_rule_name.is_dummy_rule() {
        // Call this method recursively on the dummy node
        for (next_next_rules_scope, next_next_rules) in
          self.get_next(to_rule_name.name(), tag_matches)
        {
          for next_next_rule in next_next_rules {
            // Group the next rules based on the scope
            next_rules.collect(String::from(&next_next_rules_scope), next_next_rule)
          }
        }
      } else {
        // Group the next rules based on the scope
        next_rules.collect(
          String::from(&scope),
          InstantiatedRule::new(to_rule_name, tag_matches),
        );
      }
    }
    // Add empty entry, incase no next rule was found for a particular scope
    for scope in [PARENT, GLOBAL] {
      next_rules.entry(scope.to_string()).or_default();
    }
    next_rules
  }
}

pub(crate) fn read_user_config_files(path_to_configurations: &String) -> RuleGraph {
  let path_to_config = Path::new(path_to_configurations);
  // Read the rules and edges provided by the user
  let input_rules: Rules = read_toml(&path_to_config.join("rules.toml"), true);
  let input_edges: Edges = read_toml(&path_to_config.join("edges.toml"), true);
  RuleGraphBuilder::default()
    .rules(input_rules.rules)
    .edges(input_edges.edges)
    .build()
}
