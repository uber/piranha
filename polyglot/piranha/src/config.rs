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

//! This module contains all the `structs` and implementations required for - (i) handling Piranha's run-time arguments,
//! (ii) reading language specific configurations, and (iii) API specific configurations.
//! This module defines all basic building block `structs` used by Piranha.

use crate::{
  models::piranha_arguments::PiranhaArguments,
  models::{
    outgoing_edges::{Edges, OutgoingEdges},
    rule::{Rule, Rules},
    scopes::{ScopeConfig, ScopeGenerator},
  },
  utilities::read_toml,
};

use std::path::{Path, PathBuf};

use clap::Parser;

/// A refactoring tool that eliminates dead code related to stale feature flags.
#[derive(Clone, Parser, Debug)]
#[clap(name = "Piranha")]
pub(crate) struct CommandLineArguments {
  /// Path to source code folder
  #[clap(short = 'c', long)]
  pub(crate) path_to_codebase: String,
  /// Directory containing the configuration files - `piranha_arguments.toml`, `rules.toml`,  and  `edges.toml` (optional)
  #[clap(short = 'f', long)]
  pub(crate) path_to_configurations: String,
  /// Path to output summary json
  #[clap(short = 'j', long)]
  pub(crate) path_to_output_summary: Option<String>,
}

pub(crate) fn read_config_files(
  args: &PiranhaArguments,
) -> (Vec<Rule>, Vec<OutgoingEdges>, Vec<ScopeGenerator>) {
  let path_to_config = Path::new(args.path_to_configurations());
  let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path_to_language_specific_cleanup_config =
    &project_root.join(format!("src/cleanup_rules/{}", args.language_name()));

  // Read the language specific cleanup rules and edges
  let language_rules: Rules = read_toml(
    &path_to_language_specific_cleanup_config.join("rules.toml"),
    true,
  );
  let language_edges: Edges = read_toml(
    &path_to_language_specific_cleanup_config.join("edges.toml"),
    true,
  );
  let scopes = read_toml::<ScopeConfig>(
    &path_to_language_specific_cleanup_config.join("scope_config.toml"),
    true,
  )
  .scopes();

  // Read the API specific cleanup rules and edges
  let mut input_rules: Rules = read_toml(&path_to_config.join("rules.toml"), true);
  let input_edges: Edges = read_toml(&path_to_config.join("edges.toml"), true);

  for r in input_rules.rules.iter_mut() {
    r.add_to_seed_rules_group();
  }

  let all_rules = [language_rules.rules, input_rules.rules].concat();
  let all_edges = [language_edges.edges, input_edges.edges].concat();

  (all_rules, all_edges, scopes)
}
