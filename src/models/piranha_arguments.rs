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

use super::{
  default_configs::{
    default_cleanup_comments, default_cleanup_comments_buffer,
    default_delete_consecutive_new_lines, default_delete_file_if_empty, default_dry_run,
    default_global_tag_prefix, default_name_of_piranha_argument_toml,
    default_number_of_ancestors_in_parent_scope, default_path_to_codebase,
    default_path_to_configurations, default_path_to_output_summaries, default_piranha_language,
    default_substitutions, GO, JAVA, KOTLIN, PYTHON, SWIFT, TSX, TYPESCRIPT,
  },
  language::PiranhaLanguage,
};
use crate::utilities::{parse_key_val, read_toml};
use clap::builder::TypedValueParser as _;
use clap::Parser;
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use pyo3::{prelude::*, types::PyDict};
use serde_derive::Deserialize;

use std::{collections::HashMap, path::PathBuf};

/// A refactoring tool that eliminates dead code related to stale feature flags
#[derive(Deserialize, Clone, Getters, CopyGetters, Debug, Parser, Default, Builder)]
#[clap(name = "Piranha")]
#[pyclass]
#[builder(build_fn(name = "create"))]
pub struct PiranhaArguments {
  /// Path to source code folder or file
  #[get = "pub"]
  #[builder(default = "default_path_to_codebase()")]
  #[clap(short = 'c', long)]
  #[serde(skip)]
  path_to_codebase: String,

  /// These substitutions instantiate the initial set of rules.
  /// Usage : -s stale_flag_name=SOME_FLAG -s namespace=SOME_NS1
  #[builder(default = "default_substitutions()")]
  #[clap(short= 's',value_parser = parse_key_val, default_values_t=default_substitutions())]
  #[serde(default = "default_substitutions")]
  substitutions: Vec<Substitution>,

  /// Directory containing the configuration files -  `rules.toml` and  `edges.toml` (optional)
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

  /// The target language
  #[get = "pub"]
  #[builder(default = "default_piranha_language()")]
  #[clap(short= 'l', value_parser = clap::builder::PossibleValuesParser::new([JAVA, SWIFT, PYTHON, KOTLIN, GO, TSX, TYPESCRIPT])
  .map(|s| s.parse::<PiranhaLanguage>().unwrap()))]
  #[serde(skip)]
  language: PiranhaLanguage,

  /// User option that determines whether an empty file will be deleted
  #[get = "pub"]
  #[builder(default = "default_delete_file_if_empty()")]
  #[clap(long, default_value_t = default_delete_file_if_empty())]
  #[serde(default = "default_delete_file_if_empty")]
  delete_file_if_empty: bool,

  /// Replaces consecutive `\n`s  with a `\n`
  #[get = "pub"]
  #[builder(default = "default_delete_consecutive_new_lines()")]
  #[clap(long, default_value_t = default_delete_consecutive_new_lines())]
  #[serde(default = "default_delete_consecutive_new_lines")]
  delete_consecutive_new_lines: bool,

  /// the prefix used for global tag names
  #[get = "pub"]
  #[builder(default = "default_global_tag_prefix()")]
  #[clap(long, default_value_t = default_global_tag_prefix())]
  #[serde(default = "default_global_tag_prefix")]
  global_tag_prefix: String,

  /// The number of ancestors considered when `PARENT` rules
  #[get = "pub"]
  #[builder(default = "default_number_of_ancestors_in_parent_scope()")]
  #[clap(long, default_value_t = default_number_of_ancestors_in_parent_scope())]
  #[serde(default = "default_number_of_ancestors_in_parent_scope")]
  number_of_ancestors_in_parent_scope: u8,
  /// The number of lines to consider for cleaning up the comments
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments_buffer()")]
  #[clap(long, default_value_t = default_cleanup_comments_buffer())]
  #[serde(default = "default_cleanup_comments_buffer")]
  cleanup_comments_buffer: usize,

  /// Enables deletion of associated comments
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments()")]
  #[clap(long, default_value_t = default_cleanup_comments())]
  #[serde(default = "default_cleanup_comments")]
  cleanup_comments: bool,

  /// Disables in-place rewriting of code
  #[get = "pub"]
  #[builder(default = "default_dry_run()")]
  #[clap(long, default_value_t = false)]
  #[serde(default = "default_dry_run")]
  dry_run: bool,
}

#[pymethods]
impl PiranhaArguments {
  /// Constructs PiranhaArguments
  ///
  /// # Arguments:
  /// * path_to_codebase: Path to the root of the code base that Piranha will update
  /// * path_to_configuration: Path to the directory that contains - `piranha_arguments.toml`, `rules.toml` and optionally `edges.toml`
  /// * language: Target language
  /// * substitutions : Substitutions to instantiate the initial set of feature flag rules
  /// * kw_args: Keyword arguments to capture the following piranha argument options
  ///   * dry_run (bool) : Disables in-place rewriting of code
  ///   * cleanup_comments (bool) : Enables deletion of associated comments
  ///   * cleanup_comments_buffer (usize): The number of lines to consider for cleaning up the comments
  ///   * number_of_ancestors_in_parent_scope (usize): The number of ancestors considered when `PARENT` rules
  ///   * delete_file_if_empty (bool): User option that determines whether an empty file will be deleted
  ///   * delete_consecutive_new_lines (bool) : Replaces consecutive `\n`s  with a `\n`
  /// Returns PiranhaArgument.
  #[new]
  #[args(keyword_arguments = "**")]
  fn py_new(
    path_to_codebase: String, path_to_configurations: String, language: String,
    substitutions: &PyDict, keyword_arguments: Option<&PyDict>,
  ) -> Self {
    let subs = substitutions
      .iter()
      .map(|(key, value)| Substitution {
        key: key.to_string(),
        value: value.to_string(),
      })
      .collect_vec();

    // gets `$arg_name` from `keyword_arguments` else invokes `$default_fn`
    // It also converts this string value to the appropriate data type.
    macro_rules! get_keyword_arg {
      ($arg_name:literal, $default_fn:ident, "bool") => {
        keyword_arguments
          .and_then(|x| x.get_item($arg_name))
          .map_or_else($default_fn, |x| x.is_true().unwrap())
      };
      ($arg_name:literal, $default_fn:ident, "num") => {
        keyword_arguments
          .and_then(|x| x.get_item($arg_name))
          .map_or_else($default_fn, |x| x.to_string().parse().unwrap())
      };
      ($arg_name:literal, $default_fn:ident, "string") => {
        keyword_arguments
          .and_then(|x| x.get_item($arg_name))
          .map_or_else($default_fn, |x| x.to_string())
      };
      ($arg_name:literal, $default_fn:ident, "option") => {
        keyword_arguments
          .and_then(|x| x.get_item($arg_name))
          .map_or_else($default_fn, |x| Some(x.to_string()))
      };
    }

    piranha_arguments! {
      path_to_codebase= path_to_codebase,
      path_to_configurations = path_to_configurations,
      language= PiranhaLanguage::from(language.as_str()),
      substitutions= subs,
      dry_run= get_keyword_arg!("dry_run", default_dry_run, "bool"),
      cleanup_comments= get_keyword_arg!("cleanup_comments", default_cleanup_comments, "bool"),
      cleanup_comments_buffer= get_keyword_arg!(
        "cleanup_comments_buffer",
        default_cleanup_comments_buffer,
        "num"
      ),
      number_of_ancestors_in_parent_scope= get_keyword_arg!(
        "number_of_ancestors_in_parent_scope",
        default_number_of_ancestors_in_parent_scope,
        "num"
      ),
      delete_consecutive_new_lines= get_keyword_arg!(
        "delete_consecutive_new_lines",
        default_delete_consecutive_new_lines,
        "bool"
      ),
      global_tag_prefix= get_keyword_arg!("global_tag_prefix", default_global_tag_prefix, "string"),
      delete_file_if_empty= get_keyword_arg!(
        "delete_file_if_empty",
        default_delete_file_if_empty,
        "bool"
      ),
      path_to_output_summary= get_keyword_arg!(
        "path_to_output_summary",
        default_path_to_output_summaries,
        "option"
      ),
    }
  }
}

impl PiranhaArguments {
  pub fn get_language(&self) -> String {
    self.language.name().to_string()
  }

  pub fn from_cli() -> Self {
    let p = PiranhaArguments::parse();
    piranha_arguments! {
      path_to_codebase= p.path_to_codebase().to_string(),
      substitutions = p.substitutions.clone(),
      language = p.language().clone(),
      path_to_configurations = p.path_to_configurations().to_string(),
      path_to_output_summary = p.path_to_output_summary().clone(),
      delete_file_if_empty= *p.delete_file_if_empty(),
      delete_consecutive_new_lines= *p.delete_consecutive_new_lines(),
      global_tag_prefix= p.global_tag_prefix().to_string(),
      number_of_ancestors_in_parent_scope= *p.number_of_ancestors_in_parent_scope(),
      cleanup_comments_buffer= *p.cleanup_comments_buffer(),
      cleanup_comments= *p.cleanup_comments(),
      dry_run= *p.dry_run(),
    }
  }

  pub fn merge(&self, other: PiranhaArguments) -> Self {
    /// Accepts field name (e.g. `language`) and function name (e.g. `default_language`) which returns default value for that field.
    /// It checks if the value `self.language` is same as value returned by `default_language()`.
    /// If the value is same it returns the value from `other (e.g. `other.language`) else it returns it from `self`
    macro_rules! merge {
      ($field_name:ident, $default_fn:ident) => {
        if self.$field_name != $default_fn() {
          self.$field_name.clone()
        } else {
          other.$field_name.clone()
        }
      };
    }

    Self {
      path_to_codebase: merge!(path_to_codebase, default_path_to_codebase),
      substitutions: merge!(substitutions, default_substitutions),
      path_to_configurations: merge!(path_to_configurations, default_path_to_configurations),
      path_to_output_summary: merge!(path_to_output_summary, default_path_to_output_summaries),
      language: merge!(language, default_piranha_language),
      delete_file_if_empty: merge!(delete_file_if_empty, default_delete_file_if_empty),
      delete_consecutive_new_lines: merge!(
        delete_consecutive_new_lines,
        default_delete_consecutive_new_lines
      ),
      global_tag_prefix: merge!(global_tag_prefix, default_global_tag_prefix),
      number_of_ancestors_in_parent_scope: merge!(
        number_of_ancestors_in_parent_scope,
        default_number_of_ancestors_in_parent_scope
      ),
      cleanup_comments_buffer: merge!(cleanup_comments_buffer, default_cleanup_comments_buffer),
      cleanup_comments: merge!(cleanup_comments, default_cleanup_comments),
      dry_run: merge!(dry_run, default_dry_run),
    }
  }

  pub(crate) fn input_substitutions(&self) -> HashMap<String, String> {
    self
      .substitutions
      .iter()
      .map(|s| (s.key.to_string(), s.value.to_string()))
      .collect()
  }
}

impl PiranhaArgumentsBuilder {
  /// Builds PiranhaArguments from PiranhaBuilder
  /// * create PiranhaArgument from the builder
  /// * parse `piranha_arguments.toml` (if it exists)
  /// * merge the two PiranhaArguments
  pub fn build(&self) -> PiranhaArguments {
    let _arg = &self.create().unwrap();

    let path_to_toml =
      PathBuf::from(_arg.path_to_configurations()).join(default_name_of_piranha_argument_toml());
    if path_to_toml.exists() {
      let args_from_file = read_toml::<PiranhaArguments>(&path_to_toml, false);
      return _arg.merge(args_from_file);
    }
    _arg.clone()
  }
}

#[macro_export]
/// This macro can be used to construct a PiranhaArgument (via the builder).'
/// Allows to use builder pattern more "dynamically"
///
/// Usage:
///
/// ```ignore
/// piranha_arguments! {
///   path_to_codebase = "path/to/code/base".to_string(),
///   language = "Java".to_string(),
///   path_to_configurations = "path/to/configurations".to_string(),
/// }
/// ```
///
/// expands to
///
/// ```ignore
/// PiranhaArgumentsBuilder::default()
///      .path_to_codebase("path/to/code/base".to_string())
///      .language("Java".to_string())
///      .path_to_configurations("path/to/configurations".to_string())
///      .build()
/// ```
///
macro_rules! piranha_arguments {
    ($($kw: ident = $value: expr,)*) => {
      $crate::models::piranha_arguments::PiranhaArgumentsBuilder::default()
      $(
        .$kw($value)
       )*
      .build()
    };
}

pub use piranha_arguments;

#[derive(Deserialize, Clone, Debug, Parser, Default, PartialEq)]
pub struct Substitution {
  pub key: String,
  pub value: String,
}

impl std::fmt::Display for Substitution {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{0} => {1}", self.key, self.key)
  }
}
