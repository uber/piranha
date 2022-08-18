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

use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use colored::Colorize;
use log::info;
use tree_sitter::Language;

use crate::{
  config::CommandLineArguments,
  models::piranha_config::PiranhaConfiguration,
  utilities::{read_toml, tree_sitter_utilities::TreeSitterHelpers},
};

#[derive(Clone)]
/// Captures the processed Piranha arguments (Piranha-Configuration) parsed from `path_to_feature_flag_rules`.
pub struct PiranhaArguments {
  /// Path to source code folder.
  path_to_code_base: String,
  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules.
  input_substitutions: HashMap<String, String>,
  /// Folder containing the API specific rules
  path_to_configurations: String,
  /// File to which the output summary should be written
  path_to_output_summaries: Option<String>,
  /// Tree-sitter language model
  language: Language,
  // The language name is file the extension used for files in particular language.
  language_name: String,
  // User option that determines whether an empty file will be deleted
  delete_file_if_empty: bool,
  // User option that determines whether consecutive newline characters will be
  // replaced with a newline character
  delete_consecutive_new_lines: bool,
  // User option that determines the prefix used for tag names that should be considered 
  /// global i.e. if a global tag is found when rewriting a source code unit
  /// All source code units from this point will have access to this global tag. 
  global_tag_prefix: Option<String>,
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

    Self {
      path_to_code_base: args.path_to_codebase.to_string(),
      input_substitutions,
      path_to_configurations: args.path_to_configurations,
      path_to_output_summaries: args.path_to_output_summary,
      language_name: String::from(&piranha_args_from_config.language()),
      language: piranha_args_from_config.language().get_language(),
      delete_file_if_empty: piranha_args_from_config
        .delete_file_if_empty()
        .unwrap_or(true),
      delete_consecutive_new_lines: piranha_args_from_config
        .delete_consecutive_new_lines()
        .unwrap_or(false),
      global_tag_prefix: piranha_args_from_config.global_tag_prefix(),
    }
  }

  pub(crate) fn path_to_code_base(&self) -> &str {
    self.path_to_code_base.as_ref()
  }

  pub(crate) fn input_substitutions(&self) -> &HashMap<String, String> {
    &self.input_substitutions
  }

  pub(crate) fn path_to_configurations(&self) -> &str {
    self.path_to_configurations.as_ref()
  }

  pub(crate) fn language(&self) -> Language {
    self.language
  }

  pub(crate) fn language_name(&self) -> &str {
    self.language_name.as_ref()
  }

  pub(crate) fn delete_file_if_empty(&self) -> bool {
    self.delete_file_if_empty
  }

  pub(crate) fn delete_consecutive_new_lines(&self) -> bool {
    self.delete_consecutive_new_lines
  }

  pub fn path_to_output_summaries(&self) -> Option<&String> {
        self.path_to_output_summaries.as_ref()
    }
  
  /// Returns default Global prefix tag as "GLOBAL_TAG."
  /// i.e. it expects global tag names to lok like 
  /// @GLOBAL_TAG.class_name
  pub(crate) fn global_tag_prefix(&self) -> &str {
    if let Some(t) = &self.global_tag_prefix{
      return t.as_str();
    }
    "GLOBAL_TAG."
  }
}

#[cfg(test)]
impl PiranhaArguments {
  pub(crate) fn set_delete_file_if_empty(&mut self, value: bool) {
    self.delete_file_if_empty = value;
  }

  pub(crate) fn set_delete_consecutive_new_lines(&mut self, value: bool) {
    self.delete_consecutive_new_lines = value;
  }

  pub(crate) fn dummy() -> Self {
    let language_name = String::from("java");
    PiranhaArguments {
      path_to_code_base: String::new(),
      input_substitutions: HashMap::new(),
      path_to_configurations: String::new(),
      path_to_output_summaries: None,
      language: language_name.get_language(),
      language_name,
      delete_consecutive_new_lines: false,
      delete_file_if_empty: false,
      global_tag_prefix: None
    }
  }

  pub(crate) fn dummy_with_user_opt(
    delete_file_if_empty: bool, delete_consecutive_new_lines: bool,
  ) -> Self {
    let language_name = String::from("java");
    let mut args = PiranhaArguments {
      path_to_code_base: String::new(),
      input_substitutions: HashMap::new(),
      path_to_configurations: String::new(),
      path_to_output_summaries: None,
      language: language_name.get_language(),
      language_name,
      delete_consecutive_new_lines: false,
      delete_file_if_empty: false,
      global_tag_prefix: None
    };
    args.set_delete_consecutive_new_lines(delete_consecutive_new_lines);
    args.set_delete_file_if_empty(delete_file_if_empty);
    args
  }
}
