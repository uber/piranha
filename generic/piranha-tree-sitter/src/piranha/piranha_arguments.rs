use std::{collections::HashMap, path::PathBuf};

use colored::Colorize;
use log::info;
use tree_sitter::Language;

use crate::{
  config::CommandLineArguments,
  models::piranha_config::PiranhaConfig,
  utilities::{read_toml, tree_sitter_utilities::TreeSitterHelpers},
};

#[derive(Clone)]
/// Captures the processed Piranha arguments (PiranhaArgsFromConfig) that are parsed from `path_to_feature_flag_rules`.
pub struct PiranhaArguments {
  /// Path to source code folder.
  path_to_code_base: String,
  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules.
  input_substitutions: HashMap<String, String>,
  /// Folder containing the API specific rules
  path_to_configurations: String,
  /// Tree-sitter language model
  language: Language,
  // The language name is file the extension used for files in particular language.
  language_name: String,
}

impl PiranhaArguments {
  pub(crate) fn new(args: CommandLineArguments) -> Self {
    let path_to_piranha_argument_file = PathBuf::from(args.path_to_piranha_arguments.as_str());

    let piranha_args_from_config: PiranhaConfig = read_toml(&path_to_piranha_argument_file, false);

    let input_substitutions = piranha_args_from_config.substitutions();

    #[rustfmt::skip]
      info!("{}",  format!("Piranha arguments are :\n {:?}", input_substitutions).purple());

    Self {
      path_to_code_base: args.path_to_codebase.to_string(),
      input_substitutions,
      path_to_configurations: args.path_to_feature_flag_rules.to_string(),
      language_name: String::from(&piranha_args_from_config.language()),
      language: piranha_args_from_config.language().get_language(),
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
}
