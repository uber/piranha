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

use getset::Getters;
use std::collections::HashMap;
use std::hash::Hash;

/// Trait for states (sigma), which store the states of the analysis at each program point
pub trait Sigma: Clone {
  type Node;

  /// Merges two sigma into a new one (join operator).
  fn join(&self, other: &Self) -> Self;

  /// Check if two sigmas are equal (this is used to terminate the analysis).
  fn is_equal(&self, other: &Self) -> bool;
}

/// The direction of the analysis (forwards or backwards)
pub trait Direction {
  type Node: Hash + Eq + Clone;
  type Sigma: Sigma<Node = Self::Node>;

  fn successors(&self, node: &Self::Node) -> Vec<Self::Node>;

  /// Initial abstract value for all other nodes.
  fn initial_value(&self) -> Self::Sigma;

  /// Initial abstract value for the entry point (e.g., first rule).
  fn entry_value(&self) -> Self::Sigma;

  /// Transforms the sigma based on the direction of the analysis.
  /// For now we don't consider instructions, since our analysis is straightforward.
  fn flow(&self, node: &Self::Node, input: &Self::Sigma) -> Self::Sigma;
}

// The results of a dataflow analysis is a mapping from program points to states (sigma)
pub type DfResults<N, S> = HashMap<N, S>;

// Struct for a dataflow analysis
#[derive(Debug, Getters)]
pub struct DataflowAnalysis<D: Direction> {
  direction: D,

  #[get = "pub"]
  sigma_in: DfResults<D::Node, D::Sigma>,
  #[get = "pub"]
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
  pub fn run_analysis(&mut self, blocks: Vec<D::Node>, entry_points: Vec<D::Node>) {
    let mut work_list = blocks.clone();
    blocks.iter().for_each(|block| {
      self
        .sigma_in
        .insert(block.clone(), self.direction.initial_value());
    });
    entry_points.iter().for_each(|entry| {
      self
        .sigma_in
        .insert(entry.clone(), self.direction.entry_value());
    });

    while !work_list.is_empty() {
      let cur_node = work_list.pop().unwrap();
      if let Some(sigma_in) = self.sigma_in.get(&cur_node) {
        let transferred_sigma = self.direction.flow(&cur_node, sigma_in);
        self.sigma_out.insert(cur_node.clone(), transferred_sigma);
      }

      let cur_sigma_out = self.sigma_out.get(&cur_node).unwrap();
      let successors = self.direction.successors(&cur_node);
      successors.iter().for_each(|succ| {
        let sigma_in = self.sigma_in.get(succ).unwrap();
        let new_sigma_in = sigma_in.join(cur_sigma_out);
        if !sigma_in.is_equal(&new_sigma_in) {
          self.sigma_in.insert(succ.clone(), new_sigma_in);
          work_list.push(succ.clone());
        }
      });
    }
  }
}
