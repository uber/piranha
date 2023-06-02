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

use crate::df::df::{Direction, Sigma};
use crate::models::rule::Rule;
use crate::models::rule_graph::RuleGraph;
use std::collections::HashSet;

// This file implements a data flow analysis similar to the "Definite Assignment Analysis" problem
// in compilers. Instead of tracking variable definitions, it tracks "tags" as they propagate
// through a query, modeled as a directed graph of rules (`RuleGraph`).
// The goal of the analysis is to find, for each point in the query,
// the set of tags that could reach that point without any redefinitions.
// The result can then be used to check if the query contains any tag that was not reached.

#[derive(Debug, Clone)]
pub struct DefiniteAssignmentSigma {
  variables: HashSet<String>,
}

pub struct ForwardDefiniteAssignment {
  graph: RuleGraph,
}

impl Sigma for DefiniteAssignmentSigma {
  type Node = Rule;
  type LatticeValue = Vec<String>;

  // The `merge` function computes the intersection of two sets of reaching tags.
  // This is a conservative approach that ensures that a tag is considered "reaching"
  // only if it can reach a point along all paths leading to that point.
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

impl Direction for ForwardDefiniteAssignment {
  type Node = Rule;
  type Sigma = DefiniteAssignmentSigma;

  fn successors(&self, _rule: &Rule) -> Vec<Rule> {
    let result: Vec<String> = self
      .graph
      .graph()
      .get(_rule.name())
      .map(|v| v.iter().map(|(k, _)| k.clone()).collect())
      .unwrap_or_default();

    self
      .graph
      .rules()
      .iter()
      .filter(|r| result.contains(r.name()))
      .cloned()
      .collect()
  }

  fn initial_value(&self) -> DefiniteAssignmentSigma {
     // this should be the universal set
    todo!()
  }

  fn entry_value(&self) -> DefiniteAssignmentSigma {
    // this is the input to the rule graph
    todo!()
  }

  // The `transfer` function takes a rule and the current set of reaching tags
  // (represented by `DefiniteAssignmentSigma`). It then computes the new set of reaching tags
  // after the rule is applied. This is done by inserting into the set all the tags
  // that are defined in the rule.
  fn transfer(&self, _node: &Rule, _input: &DefiniteAssignmentSigma) -> DefiniteAssignmentSigma {
    let mut result = _input.clone();
    result.variables.insert(_node.name().to_string());
    drop(result);
    todo!()
  }
}

#[cfg(test)]
#[path = "unit_tests/basic_analysis_test.rs"]
mod basic_analysis_test;
