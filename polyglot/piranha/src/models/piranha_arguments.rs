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
use colored::Colorize;
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use log::info;
use std::{collections::HashMap, path::PathBuf};
use tree_sitter::Language;

use crate::{
  config::CommandLineArguments,
  models::piranha_config::PiranhaConfiguration,
  utilities::{read_toml, tree_sitter_utilities::TreeSitterHelpers},
};

#[derive(Clone, Builder, Getters, CopyGetters, Debug)]
#[builder(default)]
/// Captures the processed Piranha arguments (Piranha-Configuration) parsed from `path_to_feature_flag_rules`.
pub struct PiranhaArguments {
  /// Path to source code folder.
  #[getset(get = "pub")]
  path_to_code_base: String,
  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules.
  #[getset(get = "pub")]
  input_substitutions: HashMap<String, String>,
  /// Folder containing the API specific rules
  #[getset(get = "pub")]
  path_to_configurations: String,
  /// File to which the output summary should be written
  #[getset(get = "pub")]
  path_to_output_summaries: Option<String>,
  /// Tree-sitter language model
  #[getset(get_copy = "pub")]
  language: Language,
  // The language name is file the extension used for files in particular language.
  #[getset(get = "pub")]
  language_name: String,
  // User option that determines whether an empty file will be deleted
  #[getset(get = "pub")]
  delete_file_if_empty: bool,
  // User option that determines whether consecutive newline characters will be
  // replaced with a newline character
  #[getset(get = "pub")]
  delete_consecutive_new_lines: bool,
  // User option that determines the prefix used for tag names that should be considered
  /// global i.e. if a global tag is found when rewriting a source code unit
  /// All source code units from this point will have access to this global tag.
  #[getset(get = "pub")]
  global_tag_prefix: String,
  /// Add a user option to configure the number of ancestors considered when applying
  /// parent scoped rules
  #[getset(get = "pub")]
  number_of_ancestors_in_parent_scope: u8,
  /// The number of lines to consider for cleaning up the comments
  #[getset(get = "pub")]
  cleanup_comments_buffer: usize,
  /// The AST Kinds for which comments should be deleted
  #[getset(get = "pub")]
  cleanup_comments: bool,
}

impl PiranhaArguments {
  pub fn from_command_line() -> Self {
    Self::new(CommandLineArguments::parse())
  }

  pub(crate) fn new(args: CommandLineArguments) -> Self {
    let path_to_piranha_argument_file =
      PathBuf::from(args.path_to_configurations.as_str()).join("piranha_arguments.toml");

    let piranha_args_from_config: PiranhaConfiguration =
      read_toml(&path_to_piranha_argument_file, false);

    let input_substitutions = piranha_args_from_config.substitutions();

    #[rustfmt::skip]
    info!("{}",  format!("Piranha arguments are :\n {:?}", input_substitutions).purple());

    let mut args_builder = PiranhaArgumentsBuilder::default();

    args_builder
      .path_to_code_base(args.path_to_codebase.to_string())
      .input_substitutions(input_substitutions)
      .path_to_configurations(args.path_to_configurations)
      .path_to_output_summaries(args.path_to_output_summary)
      .language_name(piranha_args_from_config.language())
      .language(piranha_args_from_config.language().get_language());

    if let Some(v) = piranha_args_from_config.delete_file_if_empty() {
      args_builder.delete_file_if_empty(v);
    }
    if let Some(v) = piranha_args_from_config.delete_consecutive_new_lines() {
      args_builder.delete_consecutive_new_lines(v);
    }
    if let Some(v) = piranha_args_from_config.global_tag_prefix() {
      args_builder.global_tag_prefix(v);
    }

    if let Some(buffer_size) = piranha_args_from_config.cleanup_comments_buffer() {
      args_builder.cleanup_comments_buffer(buffer_size);
    }

    if let Some(ast_kinds) = piranha_args_from_config.cleanup_comments() {
      args_builder.cleanup_comments(ast_kinds);
    }

    args_builder.build().unwrap()
  }
}

impl Default for PiranhaArguments {
  fn default() -> Self {
    let language_name = String::from("java");
    PiranhaArguments {
      path_to_code_base: String::new(),
      input_substitutions: HashMap::new(),
      path_to_configurations: String::new(),
      path_to_output_summaries: None,
      language: language_name.get_language(),
      language_name,
      delete_consecutive_new_lines: false,
      delete_file_if_empty: true,
      /// default Global prefix tag us "GLOBAL_TAG."
      /// i.e. it expects global tag names to look like
      /// @GLOBAL_TAG.class_name
      global_tag_prefix: "GLOBAL_TAG.".to_string(),
      number_of_ancestors_in_parent_scope: 4,
      cleanup_comments_buffer: 2,
      cleanup_comments: false,
    }
  }
}
