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
pub trait Sigma: Clone + Eq + PartialEq {
  type Node;

  /// Merges two sigma into a new one (join operator).
  fn join(&self, other: &Self) -> Self;
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

  /// This function performs a dataflow analysis over a control flow graph represented by blocks
  /// and entry_points. It iteratively propagates the dataflow facts (sigma) through the graph
  /// until it reaches a fixed point, i.e., when no new information can be obtained.
  /// This function implements the Worklist algorithm, for computing the fixed points in a dataflow analysis.
  /// It converges if the lattice is finite and the flow function is monotone.
  ///
  /// # Arguments
  ///
  /// * `blocks`: The set of all nodes in the control flow graph.
  /// * `entry_points`: The entry point nodes of the control flow graph. The analysis starts from these nodes.
  ///
  pub fn run_analysis(&mut self, blocks: Vec<D::Node>, entry_points: Vec<D::Node>) {
    // Create a copy of the blocks to use as a work list. The work list keeps track of nodes
    // that need to be processed because their sigma might have changed.
    let mut work_list = blocks.clone();

    // Initialize the sigma_in map with the initial value for all blocks.
    blocks.iter().for_each(|block| {
      self
        .sigma_in
        .insert(block.clone(), self.direction.initial_value());
    });

    // For entry points, we initialize them with the entry_value.
    entry_points.iter().for_each(|entry| {
      self
        .sigma_in
        .insert(entry.clone(), self.direction.entry_value());
    });

    // While there are still nodes to process in the work list
    while let Some(cur_node) = work_list.pop() {
      // Get a node from the work list, and apply the flow function to it.

      if let Some(sigma_in) = self.sigma_in.get(&cur_node) {
        let transferred_sigma = self.direction.flow(&cur_node, sigma_in);
        self.sigma_out.insert(cur_node.clone(), transferred_sigma);
      }
      let cur_sigma_out = self.sigma_out.get(&cur_node).unwrap();

      // For each successor of the current node
      let successors = self.direction.successors(&cur_node);
      successors.iter().for_each(|succ| {
        // Join the currents sigma with the sigma_in of the successor.
        let sigma_in = self.sigma_in.get(succ).unwrap();
        let new_sigma_in = sigma_in.join(cur_sigma_out);

        // If the sigma_in of the successor has changed, then it might affect its successors.
        // Therefore we add it back to the list
        if !sigma_in.eq(&new_sigma_in) {
          self.sigma_in.insert(succ.clone(), new_sigma_in);
          work_list.push(succ.clone());
        }
      });
    }
  }
}
