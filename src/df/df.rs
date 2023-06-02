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

use std::collections::HashMap;
use std::hash::Hash;

/// Trait for states (sigma), which store the states of the analysis at each program point
pub trait Sigma {
  type Node;
  type LatticeValue;

  /// Merges two sigma into a new one.
  fn merge(&self, other: &Self) -> Self;

  /// Check if two sigmas are equal
  fn is_equal(&self, other: &Self) -> bool;

  /// Fetches the abstract value associated with a variable.
  fn lookup(&self, var: &Self::Node) -> Option<&Self::LatticeValue>;

  /// Sets the abstract value for a variable.
  fn set(&mut self, var: Self::Node, value: Self::LatticeValue);
}

/// The direction of the analysis (forwards or backwards)
pub trait Direction {
  type Node: Hash + Eq + Clone;
  type Sigma: Sigma<Node = Self::Node>;

  /// Fetches the successors for a given node in the CFG (depends on the direction).
  fn successors(&self, node: &Self::Node) -> Vec<Self::Node>;

  /// Initial abstract value for all other nodes.
  fn initial_value(&self) -> Self::Sigma;

  /// Transforms the sigma based on the direction of the analysis.
  /// For now we don't consider instructions, since our analysis is straightforward.
  fn transfer(&self, node: &Self::Node, input: &Self::Sigma) -> Self::Sigma;
}

// The results of a dataflow analysis is a mapping from program points to states (sigma)
pub type DfResults<N, S> = HashMap<N, S>;

// Struct for a dataflow analysis
pub struct DataflowAnalysis<D: Direction> {
  direction: D,
  sigma_in: DfResults<D::Node, D::Sigma>,
  sigma_out: DfResults<D::Node, D::Sigma>,
}

impl<D: Direction> DataflowAnalysis<D> {
  pub fn new(direction: D) -> Self {
    DataflowAnalysis {
      direction,
      sigma_in: DfResults::new(),
      sigma_out: DfResults::new(),
    }
  }

  // From https://cmu-program-analysis.github.io/2023/index.html
  pub fn run_analysis(&mut self, blocks: Vec<D::Node>, entry_point: D::Node) {
    let mut work_list = blocks.clone();
    let mut sigmas = DfResults::<D::Node, D::Sigma>::new();
    blocks.iter().for_each(|block| {
      let sigma = self.direction.initial_value();
      sigmas.insert(block.clone(), sigma);
    });
    sigmas.insert(entry_point, self.direction.initial_value());

    while !work_list.is_empty() {
      let cur_node = work_list.pop().unwrap();
      if let Some(sigma_in) = self.sigma_in.get(&cur_node) {
        let transferred_sigma = self.direction.transfer(&cur_node, sigma_in);
        self.sigma_out.insert(cur_node.clone(), transferred_sigma);
      }

      let cur_sigma_out = self.sigma_out.get(&cur_node).unwrap();
      let successors = self.direction.successors(&cur_node);
      successors.iter().for_each(|succ| {
        let sigma_in = self.sigma_in.get(succ).unwrap();
        let new_sigma_in = sigma_in.merge(cur_sigma_out);
        if !sigma_in.is_equal(&new_sigma_in) {
          work_list.push(succ.clone());
        }
      });
    }
  }
}
