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

pub(crate) mod flag_cleaner;
pub(crate) mod piranha_arguments;
// pub(crate) mod source_code_unit;

use crate::models::scopes::ScopeGenerator;
use crate::utilities::tree_sitter_utilities::PiranhaHelpers;
use crate::{
  models::{
    rule::Rule,
    rule_store::{RuleStore, GLOBAL, PARENT},
    source_code_unit::SourceCodeUnit,
  },
  utilities::tree_sitter_utilities::get_node_for_range,
};
use colored::Colorize;
use log::info;
use std::collections::HashMap;
use std::collections::VecDeque;
use tree_sitter::{InputEdit, Node, Parser};

impl SourceCodeUnit {
  /// Will apply the `rule` to all of its occurrences in the source code unit.
  fn apply_rule(
    &mut self, rule: Rule, rules_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<String>,
  ) {
    loop {
      if !self._apply_rule(rule.clone(), rules_store, parser, scope_query) {
        break;
      }
    }
  }

  // Applies the rule to the first match in the source code
  // This is implements the main algorithm of piranha.
  fn _apply_rule(
    &mut self, rule: Rule, rules_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<String>,
  ) -> bool {
    let scope_node = self.get_scope_node(scope_query, rules_store);

    let mut any_match = false;

    // Match the rule "anywhere" inside the scope_node
    if let Some(edit_1) = rule.get_edit(&self.clone(), rules_store, scope_node, true) {
      any_match = true;
      // Get the edit for applying the matched rule to the source code
      let ts_edit = self.apply_edit(&edit_1, parser);

      // Add all the (code_snippet, tag) mapping to the substitution table.
      self.add_to_substitutions(&edit_1.matches());

      let mut current_edit = ts_edit;
      let mut current_rule = rule.clone();
      let mut next_rules_stack: VecDeque<(String, Rule)> = VecDeque::new();

      // Perform the parent edits, while queueing the Method and Class level edits.
      // let file_level_scope_names = [METHOD, CLASS];
      loop {
        // Get all the (next) rules that could be after applying the current rule (`rule`).
        let next_rules_by_scope = rules_store.get_next(&current_rule, self.substitutions());

        // Adds "Method" and "Class" rules to the stack
        self.add_rules_to_stack(
          &next_rules_by_scope,
          current_edit,
          rules_store,
          &mut next_rules_stack,
        );

        // Add Global rules as seed rules
        for r in &next_rules_by_scope[GLOBAL] {
          rules_store.add_to_global_rules(r, self.substitutions());
        }

        // Process the parent
        // Find the rules to be applied in the "Parent" scope that match any parent (context) of the changed node in the previous edit
        if let Some(edit) = Rule::get_rewrite_rule_for_context(
          &self.clone(),
          current_edit,
          rules_store,
          &next_rules_by_scope[PARENT],
        ) {
          #[rustfmt::skip]
          info!( "{}", format!( "Cleaning up the context, by applying the rule - {}", edit.matched_rule().name()).green());
          // Apply the matched rule to the parent
          current_edit = self.apply_edit(&edit, parser);
          current_rule = edit.matched_rule();
          // Add the (tag, code_snippet) mapping to substitution table.
          self.add_to_substitutions(edit.matches());
        } else {
          // No more parents found for cleanup
          break;
        }
      }

      // Apply the next rules from the stack
      for (sq, rle) in next_rules_stack {
        self.apply_rule(rle, rules_store, parser, &Some(sq.to_string()));
      }
    }
    any_match
  }

  /// Adds the "Method" and "Class" scoped next rules to the queue.
  fn add_rules_to_stack(
    &mut self, next_rules_by_scope: &HashMap<String, Vec<Rule>>, current_edit: InputEdit,
    rules_store: &mut RuleStore, stack: &mut VecDeque<(String, Rule)>,
  ) {
    for (scope_level, rules) in next_rules_by_scope {
      // Scope level is not "PArent" or "Global"
      if ![PARENT, GLOBAL].contains(&scope_level.as_str()) {
        for rule in rules {
          let scope_query =
            ScopeGenerator::get_scope_query(self.clone(), scope_level, current_edit, rules_store);
          // Add Method and Class scoped rules to the queue
          stack.push_front((scope_query, rule.instantiate(self.substitutions())));
        }
      }
    }
  }

  fn get_scope_node(&self, scope_query: &Option<String>, rules_store: &mut RuleStore) -> Node {
    // Get scope node
    // let mut scope_node = self.root_node();
    if let Some(query_str) = scope_query {
      // Apply the scope query in the source code and get the appropriate node
      let tree_sitter_scope_query = rules_store.get_query(query_str);
      if let Some((range, _)) =
        &self
          .root_node()
          .get_match_for_query(&self.code(), tree_sitter_scope_query, true)
      {
        return get_node_for_range(self.root_node(), range.start_byte, range.end_byte);
      }
    }
    self.root_node()
  }

  /// Apply all `rules` sequentially.
  pub(crate) fn apply_rules(
    &mut self, rules_store: &mut RuleStore, rules: &[Rule], parser: &mut Parser,
    scope_query: Option<String>,
  ) {
    for rule in rules {
      self.apply_rule(rule.to_owned(), rules_store, parser, &scope_query)
    }
  }
}