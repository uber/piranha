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

#![allow(deprecated)] // This prevents cargo clippy throwing warning for deprecated use.
use models::{
  edit::Edit, matches::Match, piranha_arguments::PiranhaArguments,
  piranha_output::PiranhaOutputSummary, source_code_unit::SourceCodeUnit,
};

pub mod models;
#[cfg(test)]
mod tests;
pub mod utilities;

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;
use log::{debug, info};
use tree_sitter::Parser;

use crate::models::{piranha_arguments::PiranhaArgumentsBuilder, rule_store::RuleStore};

use pyo3::prelude::{pyfunction, pymodule, wrap_pyfunction, PyModule, PyResult, Python};

/// Executes piranha for the provided configuration at {path_to_configurations} upon the given {path_to_codebase}.
///
/// # Arguments:
/// * path_to_codebase: Path to the root of the code base that Piranha will update
/// * path_to_configuration: Path to the directory that contains - `piranha_arguments.toml`, `rules.toml` and optionally `edges.toml`
/// * dry_run: determines if Piranha should actually update the code.
///
/// Returns Piranha Output Summary for each file touched or analyzed by Piranha.
/// For each file, it reports its content after the rewrite, the list of matches and the list of rewrites.
#[pyfunction]
#[deprecated(since = "0.2.1", note = "please use `execute_piranha` instead")]
pub fn run_piranha_cli(
  path_to_codebase: String, path_to_configurations: String, dry_run: bool,
) -> Vec<PiranhaOutputSummary> {
  let args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(path_to_codebase)
    .path_to_configurations(path_to_configurations)
    .dry_run(dry_run)
    .build();

  debug!("{:?}", args);
  execute_piranha(&args)
}

#[pymodule]
fn polyglot_piranha(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  m.add_function(wrap_pyfunction!(run_piranha_cli, m)?)?;
  m.add_function(wrap_pyfunction!(execute_piranha, m)?)?;
  m.add_class::<PiranhaArguments>()?;
  m.add_class::<PiranhaOutputSummary>()?;
  m.add_class::<Edit>()?;
  m.add_class::<Match>()?;
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

  let source_code_units = piranha.get_updated_files();

  for scu in source_code_units.iter() {
    scu.persist(piranha_arguments);
  }
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
  // Path to source code folder
  path_to_codebase: String,
  // Files updated by Piranha.
  relevant_files: HashMap<PathBuf, SourceCodeUnit>,
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
    let mut parser = Parser::new();
    let piranha_args = self.rule_store.piranha_args().clone();
    parser
      .set_language(*piranha_args.piranha_language().language())
      .expect("Could not set the language for the parser.");

    let mut current_global_substitutions = piranha_args.input_substitutions().clone();
    // Keep looping until new `global` rules are added.
    loop {
      let current_rules = self.rule_store.global_rules().clone();

      debug!("\n # Global rules {}", current_rules.len());
      // Iterate over each file containing the usage of the feature flag API

      for (path, content) in self.rule_store.get_relevant_files(&self.path_to_codebase) {
        // Get the `SourceCodeUnit` for the file `path` from the cache `relevant_files`.
        // In case of miss, lazily insert a new `SourceCodeUnit`.
        let source_code_unit = self
          .relevant_files
          .entry(path.to_path_buf())
          .or_insert_with(|| {
            SourceCodeUnit::new(
              &mut parser,
              content,
              &current_global_substitutions,
              path.as_path(),
              &piranha_args,
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
  }
  /// Instantiate Flag-cleaner
  fn new(piranha_arguments: &PiranhaArguments) -> Self {
    let graph_rule_store = RuleStore::new(piranha_arguments);
    Self {
      rule_store: graph_rule_store,
      path_to_codebase: String::from(piranha_arguments.path_to_codebase()),
      relevant_files: HashMap::new(),
    }
  }
}
