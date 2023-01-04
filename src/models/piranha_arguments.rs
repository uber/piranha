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

use clap::Parser;

use super::{
  default_configs::{
    default_cleanup_comments, default_cleanup_comments_buffer,
    default_delete_consecutive_new_lines, default_delete_file_if_empty, default_dry_run,
    default_global_tag_prefix, default_input_substitutions, default_languages,
    default_number_of_ancestors_in_parent_scope, default_path_to_codebase,
    default_path_to_configurations, default_path_to_output_summaries, default_piranha_language,
    default_substitutions,
  },
  language::PiranhaLanguage,
};
use crate::utilities::read_toml;
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use serde_derive::Deserialize;
use std::{collections::HashMap, path::PathBuf};

/// A refactoring tool that eliminates dead code related to stale feature flags
#[derive(Deserialize, Clone, Builder, Getters, CopyGetters, Debug, Parser, Default)]
#[clap(name = "Polyglot Piranha")]
pub struct PiranhaArguments {
  /// Path to source code folder or file
  #[get = "pub"]
  #[builder(default = "default_path_to_codebase()")]
  #[clap(short = 'c', long)]
  #[serde(skip)]
  path_to_codebase: String,
  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules
  #[get = "pub"]
  #[builder(default = "default_input_substitutions()")]
  #[clap(skip)]
  #[serde(skip)]
  input_substitutions: HashMap<String, String>,

  #[builder(setter(skip))]
  #[clap(skip)]
  #[serde(default = "default_substitutions")]
  substitutions: Vec<Vec<String>>,

  /// Directory containing the configuration files - `piranha_arguments.toml`, `rules.toml`,
  /// and  `edges.toml` (optional)
  #[get = "pub"]
  #[builder(default = "default_path_to_configurations()")]
  #[clap(short = 'f', long)]
  #[serde(skip)]
  path_to_configurations: String,
  /// Path to output summary json file
  #[get = "pub"]
  #[builder(default = "default_path_to_output_summaries()")]
  #[clap(short = 'j', long)]
  #[serde(skip)]
  path_to_output_summary: Option<String>,

  // a list of file extensions
  // #[get = "pub"]
  #[builder(default = "default_languages()")]
  #[clap(skip)]
  #[serde(default = "default_languages")]
  language: Vec<String>,

  #[get = "pub"]
  #[builder(default = "default_piranha_language()")]
  #[clap(skip)]
  #[serde(skip)]
  piranha_language: PiranhaLanguage,

  // User option that determines whether an empty file will be deleted
  #[get = "pub"]
  #[builder(default = "default_delete_file_if_empty()")]
  #[clap(skip)]
  #[serde(default = "default_delete_file_if_empty")]
  delete_file_if_empty: bool,
  // User option that determines whether consecutive newline characters will be
  // replaced with a newline character
  #[get = "pub"]
  #[builder(default = "default_delete_consecutive_new_lines()")]
  #[clap(skip)]
  #[serde(default = "default_delete_consecutive_new_lines")]
  delete_consecutive_new_lines: bool,
  // User option that determines the prefix used for tag names that should be considered
  /// global i.e. if a global tag is found when rewriting a source code unit
  /// All source code units from this point will have access to this global tag.
  #[get = "pub"]
  #[builder(default = "default_global_tag_prefix()")]
  #[clap(skip)]
  #[serde(default = "default_global_tag_prefix")]
  global_tag_prefix: String,
  /// Add a user option to configure the number of ancestors considered when applying
  /// parent scoped rules
  #[get = "pub"]
  #[builder(default = "default_number_of_ancestors_in_parent_scope()")]
  #[clap(skip)]
  #[serde(default = "default_number_of_ancestors_in_parent_scope")]
  number_of_ancestors_in_parent_scope: u8,
  /// The number of lines to consider for cleaning up the comments
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments_buffer()")]
  #[clap(skip)]
  #[serde(default = "default_cleanup_comments_buffer")]
  cleanup_comments_buffer: usize,
  /// The AST Kinds for which comments should be deleted
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments()")]
  #[clap(skip)]
  #[serde(default = "default_cleanup_comments")]
  cleanup_comments: bool,
  /// Disables in-place rewriting of code
  #[get = "pub"]
  #[builder(default = "default_dry_run()")]
  #[clap(short = 'd', long, default_value_t = false)]
  #[serde(default = "default_dry_run")]
  dry_run: bool,
}

impl PiranhaArguments {
  pub(crate) fn substitutions(&self) -> HashMap<String, String> {
    self
      .substitutions
      .iter()
      .map(|x| (x[0].clone(), x[1].clone()))
      .collect()
  }

  pub fn get_language(&self) -> String {
    self.language[0].clone()
  }

  pub(crate) fn new(path_to_piranha_arguments_toml: PathBuf) -> Self {
    let args: PiranhaArguments = read_toml(&path_to_piranha_arguments_toml, false);
    let input_substitutions = args.substitutions();
    let piranha_language = PiranhaLanguage::from(args.get_language().as_str());
    let derived_args = PiranhaArgumentsBuilder::default()
      .input_substitutions(input_substitutions)
      .piranha_language(piranha_language)
      .build()
      .unwrap();
    args.merge(derived_args)
  }

  // Returns non-default valued item when possible
  fn _merge<T: Clone + std::cmp::PartialEq>(x: T, y: T, default: T) -> T {
    if x != default {
      x
    } else {
      y
    }
  }

  pub(crate) fn merge(&self, other: PiranhaArguments) -> Self {
    Self {
      path_to_codebase: Self::_merge(
        self.path_to_codebase.clone(),
        other.path_to_codebase,
        default_path_to_codebase(),
      ),
      input_substitutions: Self::_merge(
        self.input_substitutions.clone(),
        other.input_substitutions,
        default_input_substitutions(),
      ),
      substitutions: Self::_merge(
        self.substitutions.clone(),
        other.substitutions,
        default_substitutions(),
      ),
      path_to_configurations: Self::_merge(
        self.path_to_configurations.clone(),
        other.path_to_configurations,
        default_path_to_configurations(),
      ),
      path_to_output_summary: Self::_merge(
        self.path_to_output_summary.clone(),
        other.path_to_output_summary,
        default_path_to_output_summaries(),
      ),
      language: Self::_merge(self.language.clone(), other.language, default_languages()),
      piranha_language: Self::_merge(
        self.piranha_language.clone(),
        other.piranha_language,
        default_piranha_language(),
      ),
      delete_file_if_empty: Self::_merge(
        self.delete_file_if_empty,
        other.delete_file_if_empty,
        default_delete_file_if_empty(),
      ),
      delete_consecutive_new_lines: Self::_merge(
        self.delete_consecutive_new_lines,
        other.delete_consecutive_new_lines,
        default_delete_consecutive_new_lines(),
      ),
      global_tag_prefix: Self::_merge(
        self.global_tag_prefix.clone(),
        other.global_tag_prefix,
        default_global_tag_prefix(),
      ),
      number_of_ancestors_in_parent_scope: Self::_merge(
        self.number_of_ancestors_in_parent_scope,
        other.number_of_ancestors_in_parent_scope,
        default_number_of_ancestors_in_parent_scope(),
      ),
      cleanup_comments_buffer: Self::_merge(
        self.cleanup_comments_buffer,
        other.cleanup_comments_buffer,
        default_cleanup_comments_buffer(),
      ),
      cleanup_comments: Self::_merge(
        self.cleanup_comments,
        other.cleanup_comments,
        default_cleanup_comments(),
      ),
      dry_run: Self::_merge(self.dry_run, other.dry_run, default_dry_run()),
    }
  }
}
