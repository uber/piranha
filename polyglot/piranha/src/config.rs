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
  utilities::{parse_toml, read_toml},
};

use std::path::Path;

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
  /// Disables in-place rewriting of code
  #[clap(short = 'd', long, parse(try_from_str), default_value_t = false)]
  pub(crate) dry_run: bool,
}

fn read_language_specific_rules(language_name: &str) -> Rules {
  match language_name {
    "java" => parse_toml(include_str!("cleanup_rules/java/rules.toml")),
    "kt" => parse_toml(include_str!("cleanup_rules/kt/rules.toml")),
    "swift" => parse_toml(include_str!("cleanup_rules/kt/rules.toml")),
    _ => Rules::default(),
  }
}

fn read_language_specific_edges(language_name: &str) -> Edges {
  match language_name {
    "java" => parse_toml(include_str!("cleanup_rules/java/edges.toml")),
    "kt" => parse_toml(include_str!("cleanup_rules/kt/edges.toml")),
    _ => Edges::default(),
  }
}

fn read_scope_config(language_name: &str) -> Vec<ScopeGenerator> {
  match language_name {
    "java" => parse_toml::<ScopeConfig>(include_str!("cleanup_rules/java/scope_config.toml"))
      .scopes()
      .to_vec(),
    "kt" => parse_toml::<ScopeConfig>(include_str!("cleanup_rules/kt/scope_config.toml"))
      .scopes()
      .to_vec(),
    "swift" => parse_toml::<ScopeConfig>(include_str!("cleanup_rules/swift/scope_config.toml"))
      .scopes()
      .to_vec(),
    _ => Vec::new(),
  }
}

pub(crate) fn read_config_files(
  args: &PiranhaArguments,
) -> (Vec<Rule>, Vec<OutgoingEdges>, Vec<ScopeGenerator>) {
  let path_to_config = Path::new(args.path_to_configurations());
  // Read the language specific cleanup rules and edges
  let language_rules: Rules = read_language_specific_rules(args.language_name());
  let language_edges: Edges = read_language_specific_edges(args.language_name());
  let scopes = read_scope_config(args.language_name());

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
