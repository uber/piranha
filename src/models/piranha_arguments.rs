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

use crate::models::edit::Edit;

use super::{
  default_configs::{
    default_cleanup_comments, default_cleanup_comments_buffer, default_code_snippet,
    default_delete_consecutive_new_lines, default_delete_file_if_empty, default_dry_run,
    default_global_tag_prefix, default_number_of_ancestors_in_parent_scope,
    default_path_to_codebase, default_path_to_configurations, default_path_to_output_summaries,
    default_piranha_language, default_rule_graph, default_substitutions, GO, JAVA, KOTLIN, PYTHON,
    SWIFT, TSX, TYPESCRIPT,
  },
  language::PiranhaLanguage,
  rule_graph::{read_user_config_files, RuleGraph, RuleGraphBuilder},
  source_code_unit::SourceCodeUnit,
};
use crate::utilities::parse_key_val;
use clap::builder::TypedValueParser;
use clap::Parser;
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use itertools::Itertools;
use log::{debug, info, warn};
use pyo3::{
  prelude::{pyclass, pymethods},
  types::PyDict,
};
use regex::Regex;
use tree_sitter::{InputEdit, Range};
use tree_sitter_traversal::{traverse, Order};

use std::collections::HashMap;

/// A refactoring tool that eliminates dead code related to stale feature flags
#[derive(Clone, Getters, CopyGetters, Debug, Parser, Builder)]
#[clap(name = "Piranha")]
#[pyclass]
#[builder(build_fn(name = "create"))]
pub struct PiranhaArguments {
  /// Path to source code folder or file
  #[get = "pub"]
  #[builder(default = "default_path_to_codebase()")]
  #[clap(short = 'c', long, default_value_t = default_path_to_codebase())]
  path_to_codebase: String,

  /// Code snippet to transform
  #[get = "pub"]
  #[builder(default = "default_code_snippet()")]
  #[clap(short = 't', long, default_value_t = default_code_snippet())]
  code_snippet: String,

  /// These substitutions instantiate the initial set of rules.
  /// Usage : -s stale_flag_name=SOME_FLAG -s namespace=SOME_NS1
  #[builder(default = "default_substitutions()")]
  #[clap(short = 's',value_parser = parse_key_val)]
  substitutions: Vec<(String, String)>,

  /// Directory containing the configuration files -  `rules.toml` and  `edges.toml` (optional)
  #[get = "pub"]
  #[builder(default = "default_path_to_configurations()")]
  #[clap(short = 'f', long)]
  path_to_configurations: String,

  /// Path to output summary json file
  #[get = "pub"]
  #[builder(default = "default_path_to_output_summaries()")]
  #[clap(short = 'j', long)]
  path_to_output_summary: Option<String>,
  /// The target language
  #[get = "pub"]
  #[builder(default = "default_piranha_language()")]
  #[clap(short = 'l', value_parser = clap::builder::PossibleValuesParser::new([JAVA, SWIFT, PYTHON, KOTLIN, GO, TSX, TYPESCRIPT])
  .map(|s| s.parse::<PiranhaLanguage>().unwrap()))]
  language: PiranhaLanguage,

  /// User option that determines whether an empty file will be deleted
  #[get = "pub"]
  #[builder(default = "default_delete_file_if_empty()")]
  #[clap(long, default_value_t = default_delete_file_if_empty())]
  delete_file_if_empty: bool,

  /// Replaces consecutive `\n`s  with a `\n`
  #[get = "pub"]
  #[builder(default = "default_delete_consecutive_new_lines()")]
  #[clap(long, default_value_t = default_delete_consecutive_new_lines())]
  delete_consecutive_new_lines: bool,

  /// the prefix used for global tag names
  #[get = "pub"]
  #[builder(default = "default_global_tag_prefix()")]
  #[clap(long, default_value_t = default_global_tag_prefix())]
  global_tag_prefix: String,

  /// The number of ancestors considered when `PARENT` rules
  #[get = "pub"]
  #[builder(default = "default_number_of_ancestors_in_parent_scope()")]
  #[clap(long, default_value_t = default_number_of_ancestors_in_parent_scope())]
  number_of_ancestors_in_parent_scope: u8,
  /// The number of lines to consider for cleaning up the comments
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments_buffer()")]
  #[clap(long, default_value_t = default_cleanup_comments_buffer())]
  cleanup_comments_buffer: usize,

  /// Enables deletion of associated comments
  #[get = "pub"]
  #[builder(default = "default_cleanup_comments()")]
  #[clap(long, default_value_t = default_cleanup_comments())]
  cleanup_comments: bool,

  /// Disables in-place rewriting of code
  #[get = "pub"]
  #[builder(default = "default_dry_run()")]
  #[clap(long, default_value_t = false)]
  dry_run: bool,

  // A graph that captures the flow amongst the rules
  #[get = "pub"]
  #[builder(default = "default_rule_graph()")]
  #[clap(skip)]
  rule_graph: RuleGraph,
}

impl Default for PiranhaArguments {
  fn default() -> Self {
    PiranhaArgumentsBuilder::default().build()
  }
}

#[pymethods]
impl PiranhaArguments {
  /// Constructs PiranhaArguments
  ///
  /// # Arguments:
  /// * language: Target language
  /// * substitutions : Substitutions to instantiate the initial set of feature flag rules
  /// * path_to_configuration: Path to the directory that contains - `piranha_arguments.toml`, `rules.toml` and optionally `edges.toml`
  /// * rule_graph: the graph constructed via the RuleGraph DSL
  /// * path_to_codebase: Path to the root of the code base that Piranha will update
  /// * code_snippet: Input code snippet to transform
  /// * dry_run (bool) : Disables in-place rewriting of code
  /// * cleanup_comments (bool) : Enables deletion of associated comments
  /// * cleanup_comments_buffer (usize): The number of lines to consider for cleaning up the comments
  /// * number_of_ancestors_in_parent_scope (usize): The number of ancestors considered when `PARENT` rules
  /// * delete_consecutive_new_lines (bool) : Replaces consecutive `\n`s  with a `\n`
  /// * global_tag_prefix (string): the prefix for global tags
  /// * delete_file_if_empty (bool): User option that determines whether an empty file will be deleted
  /// * path_to_output_summary : Path to the file where the Piranha output summary should be persisted
  /// Returns PiranhaArgument.
  #[new]
  fn py_new(
    language: String, substitutions: &PyDict, path_to_configurations: Option<String>,
    rule_graph: Option<RuleGraph>, path_to_codebase: Option<String>, code_snippet: Option<String>,
    dry_run: Option<bool>, cleanup_comments: Option<bool>, cleanup_comments_buffer: Option<usize>,
    number_of_ancestors_in_parent_scope: Option<u8>, delete_consecutive_new_lines: Option<bool>,
    global_tag_prefix: Option<String>, delete_file_if_empty: Option<bool>,
    path_to_output_summary: Option<String>,
  ) -> Self {
    let subs = substitutions
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect_vec();

    let rg = rule_graph.unwrap_or_else(|| RuleGraphBuilder::default().build());
    piranha_arguments! {
      path_to_codebase = path_to_codebase.unwrap_or_else(default_path_to_codebase),
      path_to_configurations = path_to_configurations.unwrap_or_else(default_path_to_configurations),
      rule_graph = rg,
      code_snippet = code_snippet.unwrap_or_else(default_code_snippet),
      language = PiranhaLanguage::from(language.as_str()),
      substitutions = subs,
      dry_run = dry_run.unwrap_or_else(default_dry_run),
      cleanup_comments = cleanup_comments.unwrap_or_else(default_cleanup_comments),
      cleanup_comments_buffer = cleanup_comments_buffer.unwrap_or_else(default_cleanup_comments_buffer),
      number_of_ancestors_in_parent_scope = number_of_ancestors_in_parent_scope.unwrap_or_else(default_number_of_ancestors_in_parent_scope),
      delete_consecutive_new_lines = delete_consecutive_new_lines.unwrap_or_else(default_delete_consecutive_new_lines),
      global_tag_prefix = global_tag_prefix.unwrap_or_else(default_global_tag_prefix),
      delete_file_if_empty = delete_file_if_empty.unwrap_or_else(default_delete_file_if_empty),
      path_to_output_summary = path_to_output_summary,
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
      path_to_codebase = p.path_to_codebase().to_string(),
      substitutions = p.substitutions.clone(),
      language = p.language().clone(),
      path_to_configurations = p.path_to_configurations().to_string(),
      path_to_output_summary = p.path_to_output_summary().clone(),
      delete_file_if_empty = *p.delete_file_if_empty(),
      delete_consecutive_new_lines = *p.delete_consecutive_new_lines(),
      global_tag_prefix = p.global_tag_prefix().to_string(),
      number_of_ancestors_in_parent_scope = *p.number_of_ancestors_in_parent_scope(),
      cleanup_comments_buffer = *p.cleanup_comments_buffer(),
      cleanup_comments = *p.cleanup_comments(),
      dry_run = *p.dry_run(),
    }
  }

  pub(crate) fn input_substitutions(&self) -> HashMap<String, String> {
    self.substitutions.iter().cloned().collect()
  }
}

impl PiranhaArgumentsBuilder {
  /// Builds PiranhaArguments from PiranhaBuilder
  /// * create PiranhaArgument from the builder
  /// * parse `piranha_arguments.toml` (if it exists)
  /// * merge the two PiranhaArguments
  pub fn build(&self) -> PiranhaArguments {
    if let Err(e) = &self._validate() {
      panic!("{}", e);
    };

    let mut _arg = self.create().unwrap();

    let rule_graph = get_rule_graph(&_arg);
    _arg = PiranhaArguments { rule_graph, .._arg };
    #[rustfmt::skip]
    info!( "Number of rules and edges loaded : {:?}", _arg.rule_graph().get_number_of_rules_and_edges());
    _arg
  }

  fn _validate(&self) -> Result<bool, String> {
    let _arg: PiranhaArguments = self.create().unwrap();
    if _arg.code_snippet().is_empty() && _arg.path_to_codebase().is_empty() {
      return Err(
        "Invalid Piranha Argument. Missing `path_to_codebase` or `code_snippet`. 
      Please specify the `path_to_codebase` or `code_snippet` when creating PiranhaArgument !!!"
          .to_string(),
      );
    }

    if !_arg.code_snippet().is_empty() && !_arg.path_to_codebase().is_empty() {
      return Err(
        "Invalid Piranha arguments. Please either specify the `path_to_codebase` or the `code_snippet`. Not Both."
          .to_string(),
      );
    }

    Ok(true)
  }
}

/// Gets rule graph for PiranhaArguments
///   * Loads the language specific graphs
///   * Merges these with the user defined graphs
/// Returns this merged graph
fn get_rule_graph(_arg: &PiranhaArguments) -> RuleGraph {
  // Get the built-in rule -graph for the language
  let piranha_language = _arg.language();

  let built_in_rules = RuleGraphBuilder::default()
    .edges(piranha_language.edges().clone().unwrap_or_default().edges)
    .rules(piranha_language.rules().clone().unwrap_or_default().rules)
    .build();

  // TODO: Move to `PiranhaArgumentBuilder`'s _validate - https://github.com/uber/piranha/issues/387
  // Get the user-defined rule graph (if any) via the Python/Rust API
  let mut user_defined_rules = _arg.rule_graph().clone();
  // In the scenario when rules/edges are passed as toml files
  if !_arg.path_to_configurations().is_empty() {
    user_defined_rules = read_user_config_files(_arg.path_to_configurations())
  }

  if user_defined_rules.graph().is_empty() {
    warn!("NO RULES PROVIDED. Please provide rules via the RuleGraph API or as toml files");
  }

  built_in_rules.merge(&user_defined_rules)
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

#[cfg(test)]
#[path = "unit_tests/piranha_arguments_test.rs"]
mod piranha_arguments_test;

// Implements instance methods related to applying the user options provided in  piranha arguments
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
          .language()
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
      std::fs::remove_file(self.path()).expect("Unable to Delete file");
      return;
    }
    std::fs::write(self.path(), self.code()).expect("Unable to Write file");
  }
}
