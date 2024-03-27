/*
Copyright (c) 2023 Uber Technologies, Inc.

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
    default_allow_dirty_ast, default_cleanup_comments, default_cleanup_comments_buffer,
    default_code_snippet, default_delete_consecutive_new_lines, default_delete_file_if_empty,
    default_dry_run, default_exclude, default_global_tag_prefix, default_graph_validation,
    default_include, default_number_of_ancestors_in_parent_scope, default_path_to_configurations,
    default_path_to_output_summaries, default_paths_to_codebase, default_piranha_language,
    default_rule_graph, default_substitutions, GO, JAVA, KOTLIN, PYTHON, SWIFT, TSX, TYPESCRIPT,RUBY,
  },
  language::PiranhaLanguage,
  rule_graph::{read_user_config_files, RuleGraph, RuleGraphBuilder},
  source_code_unit::SourceCodeUnit,
};
use crate::utilities::{parse_glob_pattern, parse_key_val};
use clap::builder::TypedValueParser;
use clap::Parser;
use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use glob::Pattern;
use itertools::Itertools;
use log::{info, warn};
use pyo3::{
  prelude::{pyclass, pymethods},
  types::PyDict,
};
use regex::Regex;

use crate::models::default_configs::default_experiment_dyn;
use crate::models::Validator;
use std::collections::HashMap;

/// A refactoring tool that eliminates dead code related to stale feature flags
#[derive(Clone, Getters, CopyGetters, Debug, Parser, Builder)]
#[clap(name = "Piranha")]
#[pyclass]
#[builder(build_fn(name = "create"))]
pub struct PiranhaArguments {
  /// Path to source code folder or file
  #[get = "pub"]
  #[builder(default = "default_paths_to_codebase()")]
  #[clap(short = 'c', long, num_args = 0.., required = true)]
  paths_to_codebase: Vec<String>,

  /// Paths to include (as glob patterns)
  #[get = "pub"]
  #[builder(default = "default_include()")]
  #[clap(long, value_parser = parse_glob_pattern, num_args = 0.., required=false)]
  include: Vec<Pattern>,

  /// Paths to exclude (as glob patterns)
  #[get = "pub"]
  #[builder(default = "default_exclude()")]
  #[clap(long, value_parser = parse_glob_pattern, num_args = 0.., required=false)]
  exclude: Vec<Pattern>,

  /// Code snippet to transform
  #[get = "pub"]
  #[builder(default = "default_code_snippet()")]
  #[clap(short = 't', long, default_value_t = default_code_snippet())]
  code_snippet: String,

  /// These substitutions instantiate the initial set of rules.
  /// Usage : -s stale_flag_name=SOME_FLAG -s namespace=SOME_NS1
  #[builder(default = "default_substitutions()")]
  #[clap(short = 's', value_parser = parse_key_val)]
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
  #[clap(short = 'l', value_parser = clap::builder::PossibleValuesParser::new([JAVA, SWIFT, PYTHON, KOTLIN, GO, TSX, TYPESCRIPT, RUBY])
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
  cleanup_comments_buffer: i32,

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

  /// Allows syntax errors in the input source code
  #[get = "pub"]
  #[builder(default = "default_allow_dirty_ast()")]
  #[clap(long, default_value_t = default_allow_dirty_ast())]
  allow_dirty_ast: bool,

  /// To validate a graph
  #[get = "pub"]
  #[builder(default = "default_graph_validation()")]
  #[clap(long, default_value_t = default_graph_validation())]
  should_validate: bool,

  /// To validate a graph
  #[get = "pub"]
  #[builder(default = "default_experiment_dyn()")]
  #[clap(long, default_value_t = default_experiment_dyn())]
  experiment_dyn: bool,
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
  /// * allow_dirty_ast : Allows syntax errors in the input source code
  /// Returns PiranhaArgument.
  #[new]
  fn py_new(
    language: String, paths_to_codebase: Option<Vec<String>>, include: Option<Vec<String>>,
    exclude: Option<Vec<String>>, substitutions: Option<&PyDict>,
    path_to_configurations: Option<String>, rule_graph: Option<RuleGraph>,
    code_snippet: Option<String>, dry_run: Option<bool>, cleanup_comments: Option<bool>,
    cleanup_comments_buffer: Option<i32>, number_of_ancestors_in_parent_scope: Option<u8>,
    delete_consecutive_new_lines: Option<bool>, global_tag_prefix: Option<String>,
    delete_file_if_empty: Option<bool>, path_to_output_summary: Option<String>,
    allow_dirty_ast: Option<bool>, should_validate: Option<bool>, experiment_dyn: Option<bool>,
  ) -> Self {
    let subs = substitutions.map_or(vec![], |s| {
      s.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect_vec()
    });

    let rg = rule_graph.unwrap_or_else(|| RuleGraphBuilder::default().build());
    PiranhaArgumentsBuilder::default()
      .paths_to_codebase(paths_to_codebase.unwrap_or_else(default_paths_to_codebase))
      .include(
        include
          .unwrap_or_default()
          .iter()
          .map(|x| Pattern::new(x).unwrap())
          .collect_vec(),
      )
      .exclude(
        exclude
          .unwrap_or_default()
          .iter()
          .map(|x| Pattern::new(x).unwrap())
          .collect_vec(),
      )
      .path_to_configurations(path_to_configurations.unwrap_or_else(default_path_to_configurations))
      .rule_graph(rg)
      .code_snippet(code_snippet.unwrap_or_else(default_code_snippet))
      .language(PiranhaLanguage::from(language.as_str()))
      .substitutions(subs)
      .dry_run(dry_run.unwrap_or_else(default_dry_run))
      .cleanup_comments(cleanup_comments.unwrap_or_else(default_cleanup_comments))
      .cleanup_comments_buffer(
        cleanup_comments_buffer.unwrap_or_else(default_cleanup_comments_buffer),
      )
      .number_of_ancestors_in_parent_scope(
        number_of_ancestors_in_parent_scope
          .unwrap_or_else(default_number_of_ancestors_in_parent_scope),
      )
      .delete_consecutive_new_lines(
        delete_consecutive_new_lines.unwrap_or_else(default_delete_consecutive_new_lines),
      )
      .global_tag_prefix(global_tag_prefix.unwrap_or_else(default_global_tag_prefix))
      .delete_file_if_empty(delete_file_if_empty.unwrap_or_else(default_delete_file_if_empty))
      .path_to_output_summary(path_to_output_summary)
      .allow_dirty_ast(allow_dirty_ast.unwrap_or_else(default_allow_dirty_ast))
      .should_validate(should_validate.unwrap_or_else(default_graph_validation))
      .experiment_dyn(experiment_dyn.unwrap_or_else(default_experiment_dyn))
      .build()
  }

  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}

impl PiranhaArguments {
  pub fn get_language(&self) -> String {
    self.language.extension().to_string()
  }

  pub fn from_cli() -> Self {
    let p = PiranhaArguments::parse();
    PiranhaArgumentsBuilder::default()
      .paths_to_codebase(p.paths_to_codebase().clone())
      .substitutions(p.substitutions.clone())
      .language(p.language().clone())
      .path_to_configurations(p.path_to_configurations().to_string())
      .path_to_output_summary(p.path_to_output_summary().clone())
      .delete_file_if_empty(*p.delete_file_if_empty())
      .delete_consecutive_new_lines(*p.delete_consecutive_new_lines())
      .global_tag_prefix(p.global_tag_prefix().to_string())
      .number_of_ancestors_in_parent_scope(*p.number_of_ancestors_in_parent_scope())
      .cleanup_comments_buffer(*p.cleanup_comments_buffer())
      .cleanup_comments(*p.cleanup_comments())
      .dry_run(*p.dry_run())
      .experiment_dyn(default_experiment_dyn())
      .build()
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
  pub fn build(&mut self) -> PiranhaArguments {
    let _arg: PiranhaArguments = self.create().unwrap();

    // This code is for a feature flag
    let piranha_language = self.language.clone();
    if piranha_language
      .filter(|x| {
        x.extension() == ".java" && self.experiment_dyn.unwrap_or(default_experiment_dyn())
      })
      .is_some()
    {
      self.language = Option::from(PiranhaLanguage::from("java_cs"));
    }
    // Ends here

    if let Err(e) = _arg.validate() {
      panic!("{}", e);
    };

    let mut _arg = self.create().unwrap();

    let rule_graph = get_rule_graph(&_arg);
    _arg = PiranhaArguments { rule_graph, .._arg };
    #[rustfmt::skip]
    info!( "Number of rules and edges loaded : {:?}", _arg.rule_graph().get_number_of_rules_and_edges());
    _arg
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
  let mut user_defined_rules: RuleGraph = _arg.rule_graph().clone();
  // In the scenario when rules/edges are passed as toml files
  if !_arg.path_to_configurations().is_empty() {
    user_defined_rules = read_user_config_files(_arg.path_to_configurations());
  }

  if user_defined_rules.graph().is_empty() {
    warn!("NO RULES PROVIDED. Please provide rules via the RuleGraph API or as toml files");
  }

  built_in_rules.merge(&user_defined_rules)
}

impl Validator for PiranhaArguments {
  fn validate(&self) -> Result<(), String> {
    let _arg: PiranhaArguments = self.clone();
    if _arg.code_snippet().is_empty() && _arg.paths_to_codebase().is_empty() {
      return Err(
        "Invalid Piranha Argument. Missing `path_to_codebase` or `code_snippet`.
      Please specify the `path_to_codebase` or `code_snippet` when creating PiranhaArgument !!!"
          .to_string(),
      );
    }
    if !_arg.code_snippet().is_empty() && !_arg.paths_to_codebase().is_empty() {
      return Err(
        "Invalid Piranha arguments. Please either specify the `path_to_codebase` or the `code_snippet`. Not Both."
            .to_string(),
      );
    }
    if self.should_validate {
      self
        .rule_graph
        .analyze_and_panic(&self.input_substitutions());
    }
    Ok(())
  }
}

#[cfg(test)]
#[path = "unit_tests/piranha_arguments_test.rs"]
mod piranha_arguments_test;

// Implements instance methods related to applying the user options provided in  piranha arguments
impl SourceCodeUnit {
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
