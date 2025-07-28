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
#![allow(deprecated)] // This prevents cargo clippy throwing warning for deprecated use.
use models::{
  edit::Edit, filter::Filter, matches::Match, outgoing_edges::OutgoingEdges,
  piranha_arguments::PiranhaArguments, piranha_output::PiranhaOutputSummary, rule::Rule,
  rule_graph::RuleGraph, source_code_unit::SourceCodeUnit,
};

#[macro_use]
extern crate lazy_static;
pub mod df;
pub mod models;

#[cfg(test)]
mod tests;
pub mod utilities;

use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use itertools::Itertools;
use log::{debug, info};

use crate::models::rule_store::RuleStore;

use pyo3::prelude::{pyfunction, pymodule, wrap_pyfunction, PyModule, PyResult, Python};
use tempdir::TempDir;

#[pymodule]
fn polyglot_piranha(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  m.add_function(wrap_pyfunction!(execute_piranha, m)?)?;
  m.add_class::<PiranhaArguments>()?;
  m.add_class::<PiranhaOutputSummary>()?;
  m.add_class::<Edit>()?;
  m.add_class::<Match>()?;
  m.add_class::<RuleGraph>()?;
  m.add_class::<Rule>()?;
  m.add_class::<OutgoingEdges>()?;
  m.add_class::<Filter>()?;
  Ok(())
}

/// Executes piranha for the given `piranha_arguments`.
///
/// # Arguments:
/// * piranha_arguments: Piranha Arguments
///
/// Returns Piranha Output Summary for each file touched or analyzed by Piranha.
/// For each file, it reports its content after the rewrite, the list of matches and the list of rewrites.
#[pyfunction]
pub fn execute_piranha(piranha_arguments: &PiranhaArguments) -> Vec<PiranhaOutputSummary> {
  info!("Executing Polyglot Piranha !!!");

  let mut piranha = Piranha::new(piranha_arguments);
  piranha.perform_cleanup();

  let summaries = piranha
    .get_updated_files()
    .iter()
    .map(PiranhaOutputSummary::new)
    .collect_vec();
  log_piranha_output_summaries(&summaries);
  summaries
}

fn log_piranha_output_summaries(summaries: &Vec<PiranhaOutputSummary>) {
  let mut total_number_of_matches: usize = 0;
  let mut total_number_of_rewrites: usize = 0;
  for summary in summaries {
    let number_of_rewrites = &summary.rewrites().len();
    let number_of_matches = &summary.matches().len();
    info!("File : {:?}", &summary.path());
    info!("  # Rewrites : {}", number_of_rewrites);
    info!("  # Matches : {}", number_of_matches);
    total_number_of_rewrites += number_of_rewrites;
    total_number_of_matches += number_of_matches;
  }
  info!("Total files affected/matched {}", &summaries.len());
  info!("Total number of matches {}", total_number_of_matches);
  info!("Total number of rewrites {}", total_number_of_rewrites);
}

// Maintains the state of Piranha and the updated content of files in the source code.
struct Piranha {
  // Maintains Piranha's state
  rule_store: RuleStore,
  // Files updated by Piranha.
  relevant_files: HashMap<PathBuf, SourceCodeUnit>,
  // Piranha Arguments
  piranha_arguments: PiranhaArguments,
}

impl Piranha {
  fn get_updated_files(&self) -> Vec<SourceCodeUnit> {
    self
      .relevant_files
      .values()
      .filter(|r| !r.matches().is_empty() || !r.rewrites().is_empty())
      .cloned()
      .collect_vec()
  }

  /// Performs cleanup related to stale flags
  fn perform_cleanup(&mut self) {
    // Setup the parser for the specific language
    let piranha_args = &self.piranha_arguments;

    let mut parser = piranha_args.language().parser();

    let mut paths_to_codebase = self.piranha_arguments.paths_to_codebase().clone();

    let temp_dir = if !self.piranha_arguments.code_snippet().is_empty() {
      let td = self.write_code_snippet_to_temp();
      paths_to_codebase = vec![td.path().to_str().unwrap_or_default().to_string()];
      Some(td)
    } else {
      None
    };

    let mut current_global_substitutions = piranha_args.input_substitutions();
    // Keep looping until new `global` rules are added.
    loop {
      let current_rules = self.rule_store.global_rules().clone();

      debug!("\n # Global rules {}", current_rules.len());
      // Iterate over each file containing the usage of the feature flag API

      for (path, content) in self.rule_store.get_relevant_files(
        &paths_to_codebase,
        piranha_args.include(),
        piranha_args.exclude(),
      ) {
        // Get the `SourceCodeUnit` for the file `path` from the cache `relevant_files`.
        // In case of miss, lazily insert a new `SourceCodeUnit`.
        if let Some(file_name) = path.file_name() {
          if file_name.to_string_lossy().ends_with(".erb") {
            debug!("Processing ERB file: {}", path.display());
          }
        }
        let source_code_unit = self
          .relevant_files
          .entry(path.to_path_buf())
          .or_insert_with(|| {
            SourceCodeUnit::new(
              &mut parser,
              content,
              &current_global_substitutions,
              path.as_path(),
              piranha_args,
            )
          });

        // Apply the rules in this `SourceCodeUnit`
        source_code_unit.apply_rules(&mut self.rule_store, &current_rules, &mut parser, None);

        // Add the substitutions for the global tags to the `current_global_substitutions`
        current_global_substitutions.extend(source_code_unit.global_substitutions());

        // Break when a new `global` rule is added
        if self.rule_store.global_rules().len() > current_rules.len() {
          debug!("Found a new global rule. Will start scanning all the files again.");
          break;
        }
      }
      // If no new `global_rules` were added, break.
      if self.rule_store.global_rules().len() == current_rules.len() {
        break;
      }
    }
    // Delete the temp dir inside which the input code snippet was copied
    if let Some(t) = temp_dir {
      _ = t.close();
    } else {
      let source_code_units = self.get_updated_files();

      for scu in source_code_units.iter() {
        scu.persist();
      }
    }
  }

  /// Instantiate Flag-cleaner
  fn new(piranha_arguments: &PiranhaArguments) -> Self {
    let graph_rule_store = RuleStore::new(piranha_arguments);
    Self {
      rule_store: graph_rule_store,
      relevant_files: HashMap::new(),
      piranha_arguments: piranha_arguments.clone(),
    }
  }

  /// Write the input code snippet into a temp directory.
  /// Returns: A temporary directory containing the created input code snippet as a file
  /// This function panics if it finds that neither `code_snippet` nor `path_to_configuration` are provided  
  fn write_code_snippet_to_temp(&self) -> TempDir {
    let temp_dir = TempDir::new_in(".", "tmp").unwrap();
    let temp_dir_path = temp_dir.path();
    let sample_file = temp_dir_path.join(format!(
      "sample.{}",
      self.piranha_arguments.language().extension()
    ));
    let mut file = File::create(sample_file).unwrap();
    file
      .write_all(self.piranha_arguments.code_snippet().as_bytes())
      .unwrap();
    temp_dir
  }
}
