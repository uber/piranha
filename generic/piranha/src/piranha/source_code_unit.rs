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

use crate::utilities::tree_sitter_utilities::{substitute_tags, PiranhaHelpers, get_context};
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
use tree_sitter::{InputEdit, Node, Parser, Range};

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
    if let Some((range, rpl, code_snippets_by_tag)) =
      self.get_match_for_rule(&rule, rules_store, scope_node, true)
    {
      any_match = true;
      // Get the edit for applying the matched rule to the source code
      let edit = self.apply_edit(range, rpl, parser);

      // Add all the (code_snippet, tag) mapping to the substitution table.
      self.add_to_substitutions(code_snippets_by_tag);

      let mut current_edit = edit;
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
        if let Some((c_range, replacement_str, matched_rule, code_snippets_by_tag_parent_rule)) =
          self.match_rules_to_context(current_edit, rules_store, &next_rules_by_scope[PARENT])
        {
          #[rustfmt::skip]
          info!( "{}", format!( "Cleaning up the context, by applying the rule - {}", matched_rule.name()).green());
          // Apply the matched rule to the parent
          current_edit = self.apply_edit(c_range, replacement_str, parser);
          current_rule = matched_rule;
          // Add the (tag, code_snippet) mapping to substitution table.
          self.add_to_substitutions(code_snippets_by_tag_parent_rule);
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
          // Generate the scope query for the previously applied edit
          // This query will precisely capture the enclosing method / class.
          let scope_query = self.get_scope_query(scope_level, current_edit, rules_store);
          // Add Method and Class scoped rules to the queue
          stack.push_front((
            scope_query.to_string(),
            rule.instantiate(self.substitutions()),
          ));
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

  // /// Get the smallest node within `self` that spans the given range.
  // fn get_node_for_range(&self, start_byte: usize, end_byte: usize) -> Node {
  //   self
  //     .ast
  //     .root_node()
  //     .descendant_for_byte_range(start_byte, end_byte)
  //     .unwrap()
  // }

  /// Generate a tree-sitter based query representing the scope of the previous edit.
  /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
  fn get_scope_query(
    &self, scope_level: &str, previous_edit: InputEdit, rules_store: &mut RuleStore,
  ) -> String {
    let root_node = self.root_node();
    let mut changed_node = get_node_for_range(
      root_node,
      previous_edit.start_byte,
      previous_edit.new_end_byte,
    );

    // Get the scope matchers for `scope_level` from the `scope_config.toml`.
    let scope_matchers = rules_store.get_scope_query_generators(scope_level);

    // Match the `scope_matcher.matcher` to the parent
    while let Some(parent) = changed_node.parent() {
      for m in &scope_matchers {
        if let Some((_, captures_by_tag)) =
          parent.get_match_for_query(&self.code(), rules_store.get_query(&m.matcher()), false)
        {
          // Generate the scope query for the specific context by substituting the
          // the tags with code snippets appropriately in the `generator` query.
          return substitute_tags(m.generator(), &captures_by_tag);
        } else {
          changed_node = parent;
        }
      }
    }
    panic!("Could not create scope query for {:?}", scope_level);
  }

  // Apply all the `rules` to the node, parent, grand parent and great grand parent.
  // Short-circuit on the first match.
  fn match_rules_to_context(
    &mut self, previous_edit: InputEdit, rules_store: &mut RuleStore, rules: &Vec<Rule>,
  ) -> Option<(Range, String, Rule, HashMap<String, String>)> {
    // Context contains -  the changed node in the previous edit, its's parent, grand parent and great grand parent
    let context = || get_context(self.root_node(), previous_edit);
    for rule in rules {
      for ancestor in &context() {
        if let Some((range, replacement, captures_by_tag)) =
          self.get_match_for_rule(rule, rules_store, *ancestor, false)
        {
          return Some((range, replacement, rule.clone(), captures_by_tag));
        }
      }
    }
    None
  }

  /// Gets the first match for the rule in `self`
  fn get_match_for_rule(
    &self, rule: &Rule, rule_store: &mut RuleStore, node: Node, recursive: bool,
  ) -> Option<(Range, String, HashMap<String, String>)> {
    // Get all matches for the query in the given scope `node`.
    let all_query_matches = node.get_all_matches_for_query(
      self.code(),
      rule_store.get_query(&rule.get_query()),
      recursive,
      Some(rule.replace_node()),
    );

    // Return the first match that satisfies constraint of the rule
    for (range, tag_substitutions) in all_query_matches {
      let matched_node = get_node_for_range(self.root_node(), range.start_byte, range.end_byte);
      if self.satisfies_constraint(matched_node, rule, &tag_substitutions, rule_store) {
        let replacement = substitute_tags(rule.replace(), &tag_substitutions);
        return Some((range, replacement, tag_substitutions));
      }
    }
    None
  }

 
  fn satisfies_constraint(
    &self, node: Node, rule: &Rule, substitutions: &HashMap<String, String>,
    rule_store: &mut RuleStore,
  ) -> bool {
    rule
      .constraints()
      .iter()
      .all(|constraint| constraint.is_satisfied(node, self.clone(), rule_store, substitutions))
  }
}
