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

use std::collections::HashSet;
use crate::df::df::{Direction, Sigma};
use crate::models::rule::Rule;
use crate::models::rule_graph::RuleGraph;

#[derive(Debug, Clone)]
pub struct ReachingSigma {
  variables: HashSet<String>,
}

pub struct ForwardReachingDirection {
  graph: RuleGraph,
}

impl Sigma for ReachingSigma {
  type Node = Rule;
  type LatticeValue = Vec<String>;

  fn merge(&self, _other: &Self) -> Self {
    todo!()
  }

  fn is_equal(&self, _other: &Self) -> bool {
    todo!()
  }

  fn lookup(&self, _var: &Self::Node) -> Option<&Self::LatticeValue> {
    todo!()
  }

  fn set(&mut self, _var: Self::Node, _value: Self::LatticeValue) {
    todo!()
  }
}

impl Direction for ForwardReachingDirection {
  type Node = Rule;
  type Sigma = ReachingSigma;

  fn successors(&self, _rule: &Rule) -> Vec<Rule> {
    let result: Vec<String> = self
      .graph
      .graph()
      .get(_rule.name())
      .map(|v| v.iter().map(|(k, _)| k.clone()).collect()).unwrap_or_default();

    self
      .graph
      .rules()
      .iter()
      .filter(|r| result.contains(r.name()))
      .cloned()
      .collect()
  }

  fn initial_value(&self) -> ReachingSigma {
    ReachingSigma { variables: HashSet::new() }
  }

  fn transfer(&self, _node: &Rule, _input: &ReachingSigma) -> ReachingSigma {
    // Get the variables that are defined in the rule
    // Append them to input

    let mut result = _input.clone();
    result.variables.insert(_node.name().to_string());
    result
  }
}

#[cfg(test)]
#[path = "unit_tests/basic_analysis_test.rs"]
mod basic_analysis_test;
