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

use crate::{models::edit::Edit, utilities::read_toml};

use super::{
  default_configs::{
    default_cleanup_comments, default_cleanup_comments_buffer,
    default_delete_consecutive_new_lines, default_delete_file_if_empty, default_dry_run,
    default_global_tag_prefix, default_input_substitutions, default_language,
    default_name_of_piranha_argument_toml, default_number_of_ancestors_in_parent_scope,
    default_path_to_codebase, default_path_to_configurations, default_path_to_output_summaries,
    default_piranha_language, default_substitutions,
  },
  language::PiranhaLanguage,
  source_code_unit::SourceCodeUnit,
};
use clap::Parser;
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use log::debug;
use pyo3::{prelude::*, types::PyDict};
use regex::Regex;
use serde_derive::Deserialize;
use tree_sitter::{InputEdit, Range};
use tree_sitter_traversal::{traverse, Order};

use std::{collections::HashMap, fs, path::PathBuf};

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

  // Input arguments provided to Piranha, mapped to tag names -
  // @stale_flag_name, @namespace, @treated, @treated_complement
  // These substitutions instantiate the initial set of feature flag rules
  #[get = "pub"]
  #[builder(default = "default_input_substitutions()")]
  #[clap(skip)]
  #[serde(skip)]
  input_substitutions: HashMap<String, String>,

  // Substitutions to instantiate the initial set of feature flag rules
  #[builder(default = "default_substitutions()")]
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

  // the target language
  #[builder(default = "default_language()")]
  #[clap(skip)]
  #[serde(default = "default_language")]
  language: String,

  #[get = "pub"]
  #[builder(default = "default_piranha_language()")]
  #[clap(skip)]
  #[serde(skip)]
  piranha_language: PiranhaLanguage,

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
      .map(|(k, v)| vec![k.to_string(), v.to_string()])
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
      language= language,
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
    self.language.clone()
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
      input_substitutions: merge!(input_substitutions, default_input_substitutions),
      path_to_configurations: merge!(path_to_configurations, default_path_to_configurations),
      path_to_output_summary: merge!(path_to_output_summary, default_path_to_output_summaries),
      language: merge!(language, default_language),
      piranha_language: merge!(piranha_language, default_piranha_language),
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
}

impl PiranhaArgumentsBuilder {
  /// Builds PiranhaArguments from PiranhaBuilder
  /// * create PiranhaArgument from the builder
  /// * parse `piranha_arguments.toml` (if it exists)
  /// * merge the two PiranhaArguments
  pub fn build(&self) -> PiranhaArguments {
    let _arg = &self.create().unwrap();
    let input_substitutions = _arg
      .substitutions
      .iter()
      .map(|x| (x[0].clone(), x[1].clone()))
      .collect();

    let piranha_language = PiranhaLanguage::from(_arg.language.as_str());

    let created_args = _arg.merge(
      PiranhaArgumentsBuilder::default()
        .input_substitutions(input_substitutions)
        .piranha_language(piranha_language)
        .create()
        .unwrap(),
    );

    let path_to_toml = PathBuf::from(created_args.path_to_configurations())
      .join(default_name_of_piranha_argument_toml());
    if path_to_toml.exists() {
      let args_from_file = read_toml::<PiranhaArguments>(&path_to_toml, false);
      return created_args.merge(args_from_file);
    }
    created_args
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

impl SourceCodeUnit {
  /// Delete the comment associated to the deleted code element
  pub(crate) fn _delete_associated_comment(
    &mut self, edit: &Edit, applied_edit: &mut InputEdit, parser: &mut tree_sitter::Parser,
  ) {
    // Check if the edit kind is "DELETE something"
    if *self.piranha_arguments().cleanup_comments() && edit.replacement_string().is_empty() {
      let deleted_at = edit.p_match().range().start_point.row;
      if let Some(comment_range) = self._get_comment_at_line(
        deleted_at,
        *self.piranha_arguments().cleanup_comments_buffer(),
        edit.p_match().range().start_byte,
      ) {
        debug!("Deleting an associated comment");
        *applied_edit = self._apply_edit(&Edit::delete_range(self.code(), comment_range), parser);
      }
    }
  }

  /// This function reports the range of the comment associated to the deleted element.
  ///
  /// # Arguments:
  /// * row : The row number where the deleted element started
  /// * buffer: Number of lines that we want to look up to find associated comment
  ///
  /// # Algorithm :
  /// Get all the nodes that either start and end at [row]
  /// If **all** nodes are comments
  /// * return the range of the comment
  /// If the [row] has no node that either starts/ends there:
  /// * recursively call this method for [row] -1 (until buffer is positive)
  fn _get_comment_at_line(
    &mut self, row: usize, buffer: usize, start_byte: usize,
  ) -> Option<Range> {
    // Get all nodes that start or end on `updated_row`.
    let mut relevant_nodes_found = false;
    let mut relevant_nodes_are_comments = true;
    let mut comment_range = None;
    // Since the previous edit was a delete, the start and end of the replacement range is [start_byte].
    let node = self
      .root_node()
      .descendant_for_byte_range(start_byte, start_byte)
      .unwrap_or_else(|| self.root_node());

    for node in traverse(node.walk(), Order::Post) {
      if node.start_position().row == row || node.end_position().row == row {
        relevant_nodes_found = true;
        let is_comment: bool = self
          .piranha_arguments()
          .piranha_language()
          .is_comment(node.kind().to_string());
        relevant_nodes_are_comments = relevant_nodes_are_comments && is_comment;
        if is_comment {
          comment_range = Some(node.range());
        }
      }
    }

    if relevant_nodes_found {
      if relevant_nodes_are_comments {
        return comment_range;
      }
    } else if buffer > 0 {
      // We pass [start_byte] itself, because we know that parent of the current row is the parent of the above row too.
      // If that's not the case, its okay, because we will not find any comments in these scenarios.
      return self._get_comment_at_line(row - 1, buffer - 1, start_byte);
    }
    None
  }

  /// Replaces three consecutive newline characters with two
  pub(crate) fn perform_delete_consecutive_new_lines(&mut self) {
    if *self.piranha_arguments().delete_consecutive_new_lines() {
      let regex = Regex::new(r"\n(\s*\n)+(\s*\n)").unwrap();
      let x = &regex.replace_all(self.code(), "\n${2}").into_owned();
      self.set_code(x.clone());
    }
  }

  /// Writes the current contents of `code` to the file system and deletes a file if empty.
  pub(crate) fn persist(&self) {
    if *self.piranha_arguments().dry_run() {
      return;
    }
    if self.code().as_str().is_empty() && *self.piranha_arguments().delete_file_if_empty() {
      fs::remove_file(self.path()).expect("Unable to Delete file");
      return;
    }
    fs::write(self.path(), self.code()).expect("Unable to Write file");
  }
}
