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

use colored::Colorize;
use itertools::Itertools;
use jwalk::WalkDir;
use log::{debug, info};
use regex::Regex;
use tree_sitter::Parser;

use crate::{
  models::{rule_store::RuleStore, piranha_input::PiranhaInput},
  utilities::read_file,
};

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

pub fn execute_piranha(configuration: &PiranhaArguments) -> Vec<PiranhaOutputSummary> {
  info!("Executing Polyglot Piranha !!!");

  let mut flag_cleaner = FlagCleaner::new(configuration);
  flag_cleaner.perform_cleanup();

  let source_code_units = flag_cleaner.get_updated_files();

  if !*configuration.dry_run() {
    for scu in source_code_units {
      scu.persist(configuration);
    }
  }

  // flag_cleaner.relevant_files
  let summaries = flag_cleaner
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
struct FlagCleaner {
  // Maintains Piranha's state
  rule_store: RuleStore,
  // Path to source code folder
  path_to_codebase: String,
  // Files updated by Piranha.
  relevant_files: HashMap<PathBuf, SourceCodeUnit>,
}

impl FlagCleaner {
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
      for (path, content) in self.get_files_containing_feature_flag_api_usage() {
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

  /// Gets all the files from the code base that (i) have the language appropriate file extension, and (ii) contains the grep pattern.
  /// Note that `WalkDir` traverses the directory with parallelism.
  /// If all the global rules have no holes (i.e. we will have no grep patterns), we will try to find a match for each global rule in every file in the target.
  fn get_files_containing_feature_flag_api_usage(&self) -> HashMap<PathBuf, String> {
    let no_global_rules_with_holes = self
      .rule_store
      .global_rules()
      .iter()
      .any(|x| x.holes().is_empty());
    let pattern = self.get_grep_heuristics();
    let files: HashMap<PathBuf, String> = WalkDir::new(&self.path_to_codebase)
      // Walk over the entire code base
      .into_iter()
      // Ignore errors
      .filter_map(|e| e.ok())
      // Filter files with the desired extension
      .filter(|de| {
        de.path()
          .extension()
          .and_then(|e| {
            e.to_str()
              .filter(|x| x.eq(&self.rule_store.piranha_args().get_language()))
          })
          .is_some()
      })
      // Read the file
      .map(|f| (f.path(), read_file(&f.path()).unwrap()))
      // Filter the files containing the desired regex pattern
      .filter(|x| no_global_rules_with_holes || pattern.is_match(x.1.as_str()))
      .collect();
    #[rustfmt::skip]
    debug!("{}", format!("Will parse and analyze {} files.", files.len()).green());
    files
  }

  /// Instantiate Flag-cleaner
  fn new(args: &PiranhaArguments) -> Self {
    let graph_rule_store = RuleStore::new(args);
    Self {
      rule_store: graph_rule_store,
      path_to_codebase: String::from(args.path_to_code_base()),
      relevant_files: HashMap::new(),
    }
  }

  /// To create the current set of global rules, certain substitutions were applied.
  /// This method creates a regex pattern matching these substituted values.
  ///
  /// At the directory level, we would always look to perform global rules. However this is expensive because
  /// it requires parsing each file. To overcome this, we apply this simple
  /// heuristic to find the (upper bound) files that would match one of our current global rules.
  /// This heuristic reduces the number of files to parse.
  ///
  fn get_grep_heuristics(&self) -> Regex {
    let reg_x = self
      .rule_store
      .global_rules()
      .iter()
      .flat_map(|r| r.grep_heuristics())
      .sorted()
      //Remove duplicates
      .dedup()
      //FIXME: Dirty trick to remove true and false. Ideally, grep heuristic could be a field in itself for a rule.
      // Since not all "holes" could be used as grep heuristic.
      .filter(|x| {
        !x.is_empty() && !x.to_lowercase().eq("true") && !x.to_lowercase().as_str().eq("false")
      })
      .join("|");
    Regex::new(reg_x.as_str()).unwrap()
  }
}
