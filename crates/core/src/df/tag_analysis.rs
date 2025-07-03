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

use crate::df::analysis::{Direction, Sigma};
use crate::df::utils::get_capture_groups_from_matcher;

use crate::models::rule::Rule;
use crate::models::rule_graph::RuleGraph;

use getset::Getters;
use std::collections::HashSet;
use std::string::String;

// This file implements a data flow analysis similar to the "Definite Assignment Analysis" problem
// in compilers. Instead of tracking variable definitions, it tracks "capture groups" as they propagate
// through the graph. The goal of the analysis is to find, for each point in the graph,
// the set of capture groups that will always reach that point. The result can then be used to check if the
// query contains any variable capture group that was not defined.

#[derive(Debug, Clone, Getters, PartialEq, Eq)]
pub struct DefiniteAssignmentSigma {
  #[get = "pub"]
  variables: HashSet<String>,
  is_bottom: bool, // A boolean flag indicating if this state represents the "bottom" element
                   // in the lattice of our dataflow analysis. The bottom element usually
                   // represents a state of maximum uncertainty (e.g., all variables may
                   // or may not be defined). It's a practical way to handle this special
                   // state without needing to actually track all possible variables.
}

// The partial order x <= y is defined as x.contains(y)
// Therefore \bot <= x, meaning \bot is the universal set
// and x <= T, meaning \top is the empty set
impl Sigma for DefiniteAssignmentSigma {
  type Node = Rule;

  // The `merge` function computes the intersection of two sets of reaching capture groups.
  // This is a conservative approach that ensures that a capture group is considered "reaching"
  // only if it can reach a point along all paths leading to that point.
  fn join(&self, _other: &Self) -> Self {
    if self.is_bottom {
      return _other.clone();
    }
    if _other.is_bottom {
      return self.clone();
    }
    DefiniteAssignmentSigma {
      variables: self
        .variables
        .intersection(&_other.variables)
        .cloned()
        .collect(),
      is_bottom: false,
    }
  }
}

pub(crate) struct ForwardDefiniteAssignment {
  pub(crate) graph: RuleGraph,
  pub(crate) initial_substitutions: HashSet<String>,
}

impl Direction for ForwardDefiniteAssignment {
  type Node = Rule;
  type Sigma = DefiniteAssignmentSigma;

  fn successors(&self, _rule: &Rule) -> Vec<Rule> {
    // Get the rules and discard the edge type from the tuple
    // (we don't care if it's a Parent, File, etc)
    let child_rules: Vec<String> = self
      .graph
      .get_neighbors(_rule.name())
      .iter()
      .map(|(_, v)| v.clone())
      .collect();

    // Get the actual rule objects that we are going to use in the analysis.
    self
      .graph
      .rules()
      .iter()
      .filter(|r| child_rules.contains(r.name()))
      .cloned()
      .collect()
  }

  // Since the join operator or this analysis is intersection, the initial value
  // all intermediate points need to be set to the universal set (bottom)
  fn initial_value(&self) -> DefiniteAssignmentSigma {
    DefiniteAssignmentSigma {
      variables: HashSet::new(),
      is_bottom: true,
    }
  }

  // For each seed rule, the only set of capture groups defined are those provided as substitutions by
  // the user
  fn entry_value(&self) -> DefiniteAssignmentSigma {
    DefiniteAssignmentSigma {
      variables: self.initial_substitutions.clone(),
      is_bottom: false,
    }
  }

  // The `flow` function takes a rule and the current set of reaching capture groups
  // (represented by `DefiniteAssignmentSigma`). It then computes the new set of reaching capture groups
  // after the rule is applied. This is done by inserting into the set all the capture groups
  // that are defined in the rule.
  fn flow(&self, _node: &Rule, _input: &DefiniteAssignmentSigma) -> DefiniteAssignmentSigma {
    let mut result = _input.clone();
    let res = get_capture_groups_from_matcher(_node);
    // insert res to result.variables
    result.variables.extend(res.iter().cloned());
    result
  }
}

#[cfg(test)]
#[path = "unit_tests/tag_analysis_test.rs"]
mod tag_analysis_test;
