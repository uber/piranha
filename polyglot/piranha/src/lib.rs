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

use models::{
  piranha_arguments::PiranhaArguments, piranha_output::PiranhaOutputSummary,
  source_code_unit::SourceCodeUnit,
};

pub mod models;
#[cfg(test)]
mod tests;
pub mod utilities;

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;
use log::{debug, info};
use tree_sitter::Parser;

use crate::models::{piranha_input::PiranhaInput, rule_store::RuleStore};

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
pub fn run_piranha_cli(
  path_to_codebase: String, path_to_configurations: String, dry_run: bool,
) -> Vec<PiranhaOutputSummary> {
  let args = PiranhaArguments::from(PiranhaInput::API {
    path_to_codebase,
    path_to_configurations,
    dry_run,
  });
  debug!("{:?}", args);
  execute_piranha(&args)
}

#[pymodule]
fn polyglot_piranha(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
  pyo3_log::init();
  m.add_function(wrap_pyfunction!(run_piranha_cli, m)?)?;
  Ok(())
}

pub fn execute_piranha(piranha_arguments: &PiranhaArguments) -> Vec<PiranhaOutputSummary> {
  info!("Executing Polyglot Piranha !!!");

  let mut piranha = Piranha::new(piranha_arguments);
  piranha.perform_cleanup();

  let source_code_units = piranha.get_updated_files();

  if !*piranha_arguments.dry_run() {
    for scu in source_code_units {
      scu.persist(piranha_arguments);
    }
  }

  // flag_cleaner.relevant_files
  let summaries = piranha
    .get_updated_files()
    .iter()
    .map(PiranhaOutputSummary::new)
    .collect_vec();
  log_piranha_output_summaries(&summaries);
  return summaries;
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
    parser
      .set_language(*self.rule_store.language())
      .expect("Could not set the language for the parser.");

    // Keep looping until new `global` rules are added.
    loop {
      let current_rules = self.rule_store.global_rules().clone();

      debug!("\n # Global rules {}", current_rules.len());
      // Iterate over each file containing the usage of the feature flag API

      for (path, content) in self.rule_store.get_relevant_files(&self.path_to_codebase) {
        self
          .relevant_files
          // Get the content of the file for `path` from the cache `relevant_files`
          .entry(path.to_path_buf())
          // Populate the cache (`relevant_files`) with the content, in case of cache miss (lazily)
          .or_insert_with(|| {
            // Create new source code unit
            SourceCodeUnit::new(
              &mut parser,
              content,
              &self.rule_store.default_substitutions(),
              path.as_path(),
              self.rule_store.piranha_args(),
            )
          })
          // Apply the rules to this file
          .apply_rules(&mut self.rule_store, &current_rules, &mut parser, None);

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
