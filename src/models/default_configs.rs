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

use std::collections::{HashMap, HashSet};

use glob::Pattern;

use super::{
  filter::Filter, language::PiranhaLanguage, outgoing_edges::OutgoingEdges, rule::Rule,
  rule_graph::RuleGraph,
};
use crate::utilities::tree_sitter_utilities::TSQuery;

pub const JAVA: &str = "java";
pub const KOTLIN: &str = "kt";
pub const GO: &str = "go";
pub const PYTHON: &str = "py";
pub const SWIFT: &str = "swift";
pub const TYPESCRIPT: &str = "ts";
pub const TSX: &str = "tsx";
pub const THRIFT: &str = "thrift";

#[cfg(test)]
//FIXME: Remove this  hack by not passing PiranhaArguments to SourceCodeUnit
pub(crate) const UNUSED_CODE_PATH: &str = "/dev/null";
pub(crate) const DEFAULT_ENCLOSING_QUERY: &str = "";

pub fn default_number_of_ancestors_in_parent_scope() -> u8 {
  4
}

pub fn default_language() -> String {
  JAVA.to_string()
}

pub fn default_substitutions() -> Vec<(String, String)> {
  vec![]
}

pub fn default_delete_file_if_empty() -> bool {
  true
}

pub fn default_cleanup_comments_buffer() -> i32 {
  2
}

pub fn default_cleanup_comments() -> bool {
  false
}

pub fn default_global_tag_prefix() -> String {
  "GLOBAL_TAG.".to_string()
}

pub fn default_dry_run() -> bool {
  false
}

pub fn default_path_to_codebase() -> String {
  String::new()
}

pub fn default_code_snippet() -> String {
  String::new()
}

pub fn default_include() -> Vec<Pattern> {
  Vec::new()
}

pub fn default_exclude() -> Vec<Pattern> {
  Vec::new()
}

pub fn default_path_to_configurations() -> String {
  String::new()
}

pub fn default_path_to_output_summaries() -> Option<String> {
  None
}

pub fn default_piranha_language() -> PiranhaLanguage {
  PiranhaLanguage::default()
}

pub fn default_delete_consecutive_new_lines() -> bool {
  false
}

pub(crate) fn default_query() -> TSQuery {
  TSQuery::new(String::new())
}

pub fn default_replace_node() -> String {
  String::new()
}

pub fn default_replace() -> String {
  String::new()
}

pub fn default_rule_graph_map() -> HashMap<String, Vec<(String, String)>> {
  HashMap::new()
}

pub(crate) fn default_holes() -> HashSet<String> {
  HashSet::new()
}

pub(crate) fn default_groups() -> HashSet<String> {
  HashSet::new()
}

pub(crate) fn default_filters() -> HashSet<Filter> {
  HashSet::new()
}

pub(crate) fn default_rules() -> Vec<Rule> {
  Vec::new()
}

pub(crate) fn default_edges() -> Vec<OutgoingEdges> {
  vec![]
}

pub(crate) fn default_not_contains_queries() -> Vec<TSQuery> {
  Vec::new()
}

pub(crate) fn default_contains_query() -> TSQuery {
  TSQuery::new(String::from(""))
}

pub(crate) fn default_contains_at_least() -> u32 {
  1
}

pub(crate) fn default_contains_at_most() -> u32 {
  u32::MAX
}

pub(crate) fn default_enclosing_node() -> TSQuery {
  TSQuery::new(String::new())
}

pub(crate) fn default_rule_name() -> String {
  String::new()
}

pub(crate) fn default_rule_graph() -> RuleGraph {
  RuleGraph::default()
}

pub(crate) fn default_is_seed_rule() -> bool {
  true
}

pub(crate) fn default_allow_dirty_ast() -> bool {
  false
}
