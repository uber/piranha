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

use crate::{utilities::read_toml};
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use std::{collections::HashMap, path::PathBuf};
use serde_derive::Deserialize;

use super::{default_configs::{
  default_cleanup_comments_buffer, default_delete_file_if_empty, default_dry_run,
  default_global_tag_prefix, default_number_of_ancestors_in_parent_scope, default_substitutions, default_languages, default_cleanup_comments,
}, language::{PiranhaLanguage, get_language}};


pub struct CommandLineInput {}

#[derive(Deserialize, Clone, Builder, Getters, CopyGetters, Debug, Parser, Default)]
#[clap(name = "Piranha1")]
/// Captures the processed Piranha arguments (Piranha-Configuration) parsed from `path_to_feature_flag_rules`.
pub struct PiranhaArguments {
  /// Path to source code folder.
  #[get = "pub"]
  #[builder(default = "String::new()")]
  #[clap(short = 'c', long = "path_to_code_base")]
  #[serde(skip)]
  path_to_code_base: String,
  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules.
  #[get = "pub"]
  #[builder(default = "HashMap::new()")]
  #[clap(skip)]
  #[serde(skip)]
  input_substitutions: HashMap<String, String>,
  
  #[builder(setter(skip))]
  #[clap(skip)]
  #[serde(default = "default_substitutions")]
  substitutions: Vec<Vec<String>>,

  /// Folder containing the API specific rules
  #[get = "pub"]
  #[builder(default = "String::new()")]
  #[clap(short = 'f', long = "path_to_configurations")]
  #[serde(skip)]
  path_to_configurations: String,
  /// File to which the output summary should be written
  #[get = "pub"]
  #[builder(default)]
  #[clap(short = 'j', long = "path_to_output_summaries")]
  #[serde(skip)]
  path_to_output_summaries: Option<String>,
  
  // a list of file extensions
  // #[get = "pub"]
  #[builder(default = "default_languages()")]
  #[clap(skip)]
  #[serde(default = "default_languages")]
  language: Vec<String>,

  #[get = "pub"]
  #[builder(default = "PiranhaLanguage::default()")]
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
  #[builder(default)]
  #[clap(skip)]
  #[serde(default)]
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
  #[clap(short = 'd', long, parse(try_from_str), default_value_t = false)]
  #[serde(default = "default_dry_run")]
  dry_run: bool,
}

impl From<PathBuf> for PiranhaArgumentsBuilder {

  fn from(path_to_arguments_toml: PathBuf) -> Self {
    let other: PiranhaArguments = read_toml(&path_to_arguments_toml, false);
    let subs = other.substitutions();
    let lang = get_language(other.get_language());
    PiranhaArgumentsBuilder::default()
      .path_to_code_base(other.path_to_code_base)
      .path_to_configurations(other.path_to_configurations)
      .path_to_output_summaries(other.path_to_output_summaries)
      .input_substitutions(subs)
      .dry_run(other.dry_run)
      .language(other.language)
      .delete_file_if_empty(other.delete_file_if_empty)
      .delete_consecutive_new_lines(other.delete_consecutive_new_lines)
      .global_tag_prefix(other.global_tag_prefix)
      .cleanup_comments_buffer(other.cleanup_comments_buffer)
      .cleanup_comments(other.cleanup_comments)
      .number_of_ancestors_in_parent_scope(other.number_of_ancestors_in_parent_scope)
      .piranha_language(lang)
      .clone()
  }
}



impl From<CommandLineInput> for PiranhaArgumentsBuilder {

  fn from(_:CommandLineInput) -> Self {
    let command_line_options: PiranhaArguments = PiranhaArguments::parse();

    let path_to_piranha_argument_file =
      PathBuf::from(command_line_options.path_to_configurations()).join("piranha_arguments.toml");

    PiranhaArgumentsBuilder::from(path_to_piranha_argument_file)
      .path_to_code_base(command_line_options.path_to_code_base)
      .path_to_configurations(command_line_options.path_to_configurations)
      .path_to_output_summaries(command_line_options.path_to_output_summaries)
      .dry_run(command_line_options.dry_run)
      .clone()
      
  }
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
}
