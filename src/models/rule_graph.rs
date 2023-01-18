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

use colored::Colorize;
use getset::Getters;
use itertools::Itertools;
use log::{debug, info};

use crate::{
  models::{outgoing_edges::OutgoingEdges, rule::Rule},
  utilities::read_toml,
};
use std::{collections::HashMap, path::Path};

use super::{
  outgoing_edges::Edges,
  piranha_arguments::PiranhaArguments,
  rule::{InstantiatedRule, Rules},
};

#[derive(Debug, Default, Getters, Clone)]
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

    // Add all rules as nodes of the graph
    let mut graph = HashMap::new();
    for (key, _value) in rules_by_name.iter() {
      graph.insert(key.to_string(), vec![]);
    }
    // Add the edge(s) to the graph. Multiple edges will be added
    // when either edge endpoint is a group name.
    for edge in edges {
      for from_rule in get_rules_for_tag_or_name(edge.get_from()) {
        for outgoing_edge in edge.get_to() {
          for to_rule in get_rules_for_tag_or_name(outgoing_edge) {
            // Add edge to the adjacency list
            if let Some(x) = graph.get_mut(from_rule) {
              x.push((String::from(edge.get_scope()), to_rule.clone()))
            }
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
  pub(crate) fn log_graph_cardinality(&self) {
    let total_edges: usize = self.graph().iter().map(|x| x.1.len()).sum();
    info!(
      "Number of rules and edges loaded : ({}, {})",
      self.graph.len(),
      total_edges
    );
  }

  /// Get the number of nodes and edges in the rule graph
  pub(crate) fn get_rules(&self) -> Vec<Rule> {
    self
      .graph()
      .keys()
      .map(|k| self.rules_by_name[k].clone())
      .collect_vec()
  }
}

#[derive(Debug, Getters, Default)]
pub(crate) struct InstantiatedRuleGraph {
  // A graph that captures the flow amongst the rules
  #[get = "pub"]
  pub(crate) rule_graph: RuleGraph,
  #[get = "pub"]
  pub(crate) seed_rules: Vec<InstantiatedRule>,
}

impl From<PiranhaArguments> for InstantiatedRuleGraph {
  fn from(piranha_arguments: PiranhaArguments) -> Self {
    let (rules, edges) = _read_config_files(&piranha_arguments);
    let rule_graph = RuleGraph::new(&edges, &rules);
    let seed_rules = rule_graph
      .get_rules()
      .iter()
      .filter(|x| x.is_seed_rule())
      .map(|x| InstantiatedRule::new(x, piranha_arguments.input_substitutions()))
      .collect();

    rule_graph.log_graph_cardinality();

    InstantiatedRuleGraph {
      rule_graph,
      seed_rules,
    }
  }
}

impl InstantiatedRuleGraph {
  pub(crate) fn add_seed_rule(&self, rule: InstantiatedRule) -> Self {
    #[rustfmt::skip]
    debug!("{}", format!("Added Global Rule - {:?}", self).bright_blue());
    InstantiatedRuleGraph {
      rule_graph: self.rule_graph.clone(),
      seed_rules: vec![self.seed_rules().clone(), vec![rule]].concat(),
    }
  }
}

fn _read_config_files(args: &PiranhaArguments) -> (Vec<Rule>, Vec<OutgoingEdges>) {
  let path_to_config = Path::new(args.path_to_configurations());
  // Read the language specific cleanup rules and edges
  let language_rules: Rules = args.piranha_language().rules().clone().unwrap_or_default();
  let language_edges: Edges = args.piranha_language().edges().clone().unwrap_or_default();

  // Read the API specific cleanup rules and edges
  let mut input_rules: Rules = read_toml(&path_to_config.join("rules.toml"), true);
  let input_edges: Edges = read_toml(&path_to_config.join("edges.toml"), true);

  for r in input_rules.rules.iter_mut() {
    r.add_to_seed_rules_group();
  }

  let all_rules = [language_rules.rules, input_rules.rules].concat();
  let all_edges = [language_edges.edges, input_edges.edges].concat();

  (all_rules, all_edges)
}
