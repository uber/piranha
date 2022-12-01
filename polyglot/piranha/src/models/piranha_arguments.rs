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

use super::default_configs::{default_delete_file_if_empty, default_global_tag_prefix, default_number_of_ancestors_in_parent_scope, default_cleanup_comments_buffer, default_language, default_dry_run};



#[derive(Clone, Builder, Getters, CopyGetters, Debug)]
// #[derivative(Default)]
/// Captures the processed Piranha arguments (Piranha-Configuration) parsed from `path_to_feature_flag_rules`.
pub struct PiranhaArguments {
  /// Path to source code folder.
  #[get = "pub"]
  #[builder(default = "String::new()")]
  path_to_code_base: String,
  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules.
  #[get = "pub"]
  #[builder(default = "HashMap::new()")]
  input_substitutions: HashMap<String, String>,
  /// Folder containing the API specific rules
  #[get = "pub"]
  #[builder(default = "String::new()")]
  path_to_configurations: String,
  /// File to which the output summary should be written
  #[get = "pub"]
  #[builder(default)]
  path_to_output_summaries: Option<String>,
  // The language name is file the extension used for files in particular language.
  #[builder(default = "default_language()")]
  #[get = "pub"]
  language_name: String,
  /// Tree-sitter language model
  #[getset(get_copy = "pub")]
  #[builder(default = "default_language_model()")]
  // #[derivative(Default(value = "default_language_model()"))]
  language: Language,
  // User option that determines whether an empty file will be deleted
  #[get = "pub"]
  #[builder(default = "default_delete_file_if_empty()")]
  delete_file_if_empty: bool,
  // User option that determines whether consecutive newline characters will be
  // replaced with a newline character
  #[get = "pub"]
  #[builder(default)]
  delete_consecutive_new_lines: bool,
  // User option that determines the prefix used for tag names that should be considered
  /// global i.e. if a global tag is found when rewriting a source code unit
  /// All source code units from this point will have access to this global tag.
  #[get = "pub"]
  #[builder(default = "default_global_tag_prefix()")]
  global_tag_prefix: String,
  /// Add a user option to configure the number of ancestors considered when applying
  /// parent scoped rules
  #[get = "pub"]
  #[builder(default = "default_number_of_ancestors_in_parent_scope()")]
  number_of_ancestors_in_parent_scope: u8,
  /// The number of lines to consider for cleaning up the comments
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments_buffer()")]
  cleanup_comments_buffer: usize,
  /// The AST Kinds for which comments should be deleted
  #[get = "pub"]
  #[builder(default = "false")]
  cleanup_comments: bool,
  /// Disables in-place rewriting of code
  #[get = "pub"]
  #[builder(default = "default_dry_run()")]
  dry_run: bool,
}


fn default_language_model()-> tree_sitter::Language {
  "java".to_string().get_language()
}

impl PiranhaArguments {
  pub fn from_command_line() -> Self {
    Self::new(CommandLineArguments::parse())
  }

  pub(crate) fn new(args: CommandLineArguments) -> Self {
    let path_to_piranha_argument_file =
      PathBuf::from(args.path_to_configurations.as_str()).join("piranha_arguments.toml");

    let config: PiranhaConfiguration =
      read_toml(&path_to_piranha_argument_file, false);

    let input_substitutions = config.substitutions();

    #[rustfmt::skip]
    info!("{}",  format!("Piranha arguments are :\n {:?}", input_substitutions).purple());

    PiranhaArgumentsBuilder::default()
      .path_to_code_base(args.path_to_codebase.to_string())
      .input_substitutions(input_substitutions)
      .path_to_configurations(args.path_to_configurations)
      .path_to_output_summaries(args.path_to_output_summary)
      .language_name(config.language())
      .language(config.language().get_language())
      .delete_file_if_empty(*config.delete_file_if_empty())
      .delete_consecutive_new_lines(*config.delete_consecutive_new_lines())
      .global_tag_prefix(config.global_tag_prefix().to_string())
      .cleanup_comments_buffer(*config.cleanup_comments_buffer())
      .cleanup_comments(*config.cleanup_comments())
      .number_of_ancestors_in_parent_scope(*config.number_of_ancestors_in_parent_scope())
      .dry_run(args.dry_run)
      .build()
      .unwrap()

    // args_builder.build().unwrap()
  }
}

