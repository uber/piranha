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

use std::fmt;

use colored::Colorize;
use getset::{Getters, MutGetters};
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};
use tree_sitter::{Node, Range};

use super::{
  matches::Match, rule::InstantiatedRule, rule_store::RuleStore, source_code_unit::SourceCodeUnit,
};
use crate::utilities::{
  gen_py_str_methods,
  tree_sitter_utilities::{get_context, get_node_for_range},
  Instantiate,
};
use pyo3::{prelude::pyclass, pymethods};

#[derive(Serialize, Debug, Clone, Getters, MutGetters, Deserialize)]
#[pyclass]
pub(crate) struct Edit {
  // The match representing the target site of the edit
  #[pyo3(get)]
  #[get = "pub"]
  #[get_mut]
  p_match: Match,
  // The string to replace the substring encompassed by the match
  #[pyo3(get)]
  #[get = "pub"]
  replacement_string: String,
  // The rule used for creating this match-replace
  #[pyo3(get)]
  #[get = "pub"]
  matched_rule: String,
}

gen_py_str_methods!(Edit);

impl Edit {
  pub(crate) fn new(
    p_match: Match, replacement_string: String, matched_rule: String, code: &String,
  ) -> Self {
    let mut edit = Self {
      p_match,
      replacement_string,
      matched_rule,
    };
    if edit.is_delete() {
      edit.p_match_mut().expand_to_associated_matches(code);
    }
    edit
  }
  #[cfg(test)]
  pub(crate) fn delete_range(code: &str, replacement_range: Range) -> Self {
    use std::collections::HashMap;
    Self {
      p_match: Match::new(
        code[replacement_range.start_byte..replacement_range.end_byte].to_string(),
        replacement_range,
        HashMap::new(),
      ),
      replacement_string: String::new(),
      matched_rule: "Delete Range".to_string(),
    }
  }

  pub(crate) fn is_delete(&self) -> bool {
    self.replacement_string.trim().is_empty()
  }
}

impl fmt::Display for Edit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let replace_range: Range = self.p_match().range();
    let replacement = self.replacement_string();
    let replaced_code_snippet = self.p_match().matched_string();
    let mut edit_kind = "Delete code".red();
    let mut replacement_snippet_fmt = format!("{} ", replaced_code_snippet.italic());
    if !replacement.is_empty() {
      edit_kind = "Update code".green();
      replacement_snippet_fmt.push_str(&format!("\n to \n{}", replacement.italic()))
    }
    write!(
      f,
      "\n {} at ({:?}) -\n {}",
      edit_kind, &replace_range, replacement_snippet_fmt
    )
  }
}

// Implements instance methods related to getting edits for rule(s)
impl SourceCodeUnit {
  // Apply all the `rules` to the node, parent, grand parent and great grand parent.
  // Short-circuit on the first match.
  pub(crate) fn get_edit_for_context(
    &self, previous_edit_start: usize, previous_edit_end: usize, rules_store: &mut RuleStore,
    rules: &Vec<InstantiatedRule>,
  ) -> Option<Edit> {
    let number_of_ancestors_in_parent_scope = *self
      .piranha_arguments()
      .number_of_ancestors_in_parent_scope();
    let changed_node = get_node_for_range(self.root_node(), previous_edit_start, previous_edit_end);
    debug!(
      "\n{}",
      format!("Changed node kind {}", changed_node.kind()).blue()
    );
    // Context contains -  the changed node in the previous edit, its's parent, grand parent and great grand parent
    let context = || {
      get_context(
        changed_node,
        self.code().to_string(),
        number_of_ancestors_in_parent_scope,
      )
    };
    for rule in rules {
      for ancestor in &context() {
        if let Some(edit) = self.get_edit(rule, rules_store, *ancestor, false) {
          return Some(edit);
        }
      }
    }
    None
  }

  /// Gets the first match for the rule in `self`
  pub(crate) fn get_edit(
    &self, rule: &InstantiatedRule, rule_store: &mut RuleStore, node: Node, recursive: bool,
  ) -> Option<Edit> {
    // Get all matches for the query in the given scope `node`.

    return self
      .get_matches(rule, rule_store, node, recursive)
      .first()
      .map(|p_match| {
        let replacement_string = rule.replace().instantiate(p_match.matches());
        let edit = Edit::new(
          p_match.clone(),
          replacement_string,
          rule.name(),
          self.code(),
        );
        trace!("Rewrite found : {:#?}", edit);
        edit
      });
  }
}
