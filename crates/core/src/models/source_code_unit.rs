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
use std::{
  collections::{HashMap, VecDeque},
  path::{Path, PathBuf},
};

use colored::Colorize;
use itertools::Itertools;
use log::{debug, error};

use tree_sitter::{InputEdit, Node, Parser, Tree};

use crate::{
  models::capture_group_patterns::CGPattern,
  models::rule_graph::{GLOBAL, PARENT, PARENT_ITERATIVE},
  utilities::tree_sitter_utilities::{
    get_node_for_range, get_replace_range, get_tree_sitter_edit, number_of_errors,
  },
};

use super::{
  edit::Edit,
  matches::{Match, Range},
  piranha_arguments::PiranhaArguments,
  rule::InstantiatedRule,
  rule_store::RuleStore,
};
use getset::{CopyGetters, Getters, MutGetters, Setters};
// Maintains the updated source code content and AST of the file
#[derive(Clone, Getters, CopyGetters, MutGetters, Setters)]
pub(crate) struct SourceCodeUnit {
  // The tree representing the file
  ast: Tree,
  // The original content of a file
  #[get = "pub"]
  #[set = "pub(crate)"]
  original_content: String,
  // The content of a file
  #[get = "pub"]
  #[set = "pub(crate)"]
  code: String,
  // The tag substitution cache.
  // This map is looked up to instantiate new rules.
  #[get = "pub"]
  substitutions: HashMap<String, String>,
  // The path to the source code.
  #[get = "pub"]
  path: PathBuf,

  // Rewrites applied to this source code unit
  #[get = "pub"]
  #[get_mut = "pub"]
  rewrites: Vec<Edit>,
  // Matches for the read_only rules in this source code unit
  #[get = "pub"]
  #[get_mut = "pub"]
  matches: Vec<(String, Match)>,
  // Piranha Arguments passed by the user
  #[get = "pub"]
  piranha_arguments: PiranhaArguments,
}

impl SourceCodeUnit {
  pub(crate) fn new(
    parser: &mut Parser, code: String, substitutions: &HashMap<String, String>, path: &Path,
    piranha_arguments: &PiranhaArguments,
  ) -> Self {
    let ast = parser.parse(&code, None).expect("Could not parse code");
    let source_code_unit = Self {
      ast,
      original_content: code.to_string(),
      code,
      substitutions: substitutions.clone(),
      path: path.to_path_buf(),
      rewrites: Vec::new(),
      matches: Vec::new(),
      piranha_arguments: piranha_arguments.clone(),
    };
    // Panic if allow dirty ast is false and the tree is syntactically incorrect
    if !piranha_arguments.allow_dirty_ast() && source_code_unit._number_of_errors() > 0 {
      error!("{}: {}", "Syntax Error".red(), path.to_str().unwrap().red());
      _ = &source_code_unit._panic_for_syntax_error();
    }

    source_code_unit
  }

  pub(crate) fn root_node(&self) -> Node<'_> {
    self.ast.root_node()
  }

  /// Will apply the `rule` to all of its occurrences in the source code unit.
  fn apply_rule(
    &mut self, rule: InstantiatedRule, rules_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<CGPattern>,
  ) {
    loop {
      if !self._apply_rule(rule.clone(), rules_store, parser, scope_query) {
        break;
      }
    }
  }

  /// Applies the rule to the first match in the source code
  /// This is implements the main algorithm of piranha.
  /// Parameters:
  /// * `rule` : the rule to be applied
  /// * `rule_store`: contains the input rule graph.
  ///
  /// Algorithm:
  /// * check if the rule is match only
  /// ** IF not (i.e. it is a rewrite):
  /// *** Get the first match of the rule for the file
  ///  (We only get the first match because the idea is that we will apply this change, and keep calling this method `_apply_rule` until all
  /// matches have been exhaustively updated.
  /// *** Apply the rewrite
  /// *** Update the substitution table
  /// *** Propagate the change
  /// ** Else (i.e. it is a match only rule):
  /// *** Get all the matches, and for each match
  /// *** Update the substitution table
  /// *** Propagate the change
  fn _apply_rule(
    &mut self, rule: InstantiatedRule, rule_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<CGPattern>,
  ) -> bool {
    let scope_node = self.get_scope_node(scope_query, rule_store);

    let mut query_again = false;

    // When rule is a "rewrite" rule :
    // Update the first match of the rewrite rule
    // Add mappings to the substitution
    // Propagate each applied edit. The next rule will be applied relative to the application of this edit.
    if !rule.rule().is_match_only_rule() {
      if let Some(edit) = self.get_edit(&rule, rule_store, scope_node, true) {
        self.rewrites_mut().push(edit.clone());
        query_again = true;

        // Add all the (code_snippet, tag) mapping to the substitution table.
        self.substitutions.extend(edit.p_match().matches().clone());

        // Apply edit_1
        let applied_ts_edit = self.apply_edit(&edit, parser);

        self.propagate(get_replace_range(applied_ts_edit), rule, rule_store, parser);
      }
    }
    // When rule is a "match-only" rule :
    // Get all the matches
    // Add mappings to the substitution
    // Propagate each match. Note that,  we pass a identity edit (where old range == new range) in to the propagate logic.
    // The next edit will be applied relative to the identity edit.
    else {
      for m in self.get_matches(&rule, rule_store, scope_node, true) {
        self.matches_mut().push((rule.name(), m.clone()));

        // In this scenario we pass the match and replace range as the range of the match `m`
        // This is equivalent to propagating an identity rule
        //  i.e. a rule that replaces the matched code with itself
        // Note that, here we DO NOT invoke the `_apply_edit` method and only update the `substitutions`
        // By NOT invoking this we simulate the application of an identity rule
        //
        self.substitutions.extend(m.matches().clone());

        self.propagate(*m.range(), rule.clone(), rule_store, parser);
      }
    }
    query_again
  }

  /// This is the propagation logic of the Piranha's main algorithm.
  /// Parameters:
  ///  * `applied_ts_edit` -  it's(`rule`'s) application site (in terms of replacement range)
  ///  * `rule` - The `rule` that was just applied
  ///  * `rule_store` - contains the input "rule graph"
  ///  * `parser` - parser for the language
  /// Algorithm:
  ///
  /// (i) Lookup the `rule_store` and get all the (next) rules that could be after applying the current rule (`rule`).
  ///   * We will receive the rules grouped by scope:  `GLOBAL` and `PARENT` are applicable to each language. However, other scopes are determined
  ///     based on the `<language>/scope_config.toml`.
  /// (ii) Add the `GLOBAL` rule to the global rule list in the `rule_store` (This will be performed in the next iteration)
  /// (iii) Apply the local cleanup i.e. `PARENT` scoped rules
  ///  (iv) Go to step 1 (and repeat this for the applicable parent scoped rule. Do this until, no parent scoped rule is applicable.) (recursive)
  ///  (iv) Apply the rules based on custom language specific scopes (as defined in `<language>/scope_config.toml`) (recursive)
  ///
  fn propagate(
    &mut self, replace_range: Range, rule: InstantiatedRule, rules_store: &mut RuleStore,
    parser: &mut Parser,
  ) {
    let mut current_replace_range = replace_range;

    let mut current_rule = rule.name();
    let mut next_rules_stack: VecDeque<(CGPattern, InstantiatedRule)> = VecDeque::new();
    // Perform the parent edits, while queueing the Method and Class level edits.
    // let file_level_scope_names = [METHOD, CLASS];
    loop {
      debug!("Current Rule: {current_rule}");
      // Get all the (next) rules that could be after applying the current rule (`rule`).
      let next_rules_by_scope = self
        .piranha_arguments
        .rule_graph()
        .get_next(&current_rule, self.substitutions());

      debug!(
        "\n{}",
        &next_rules_by_scope
          .iter()
          .map(|(k, v)| {
            let rules = v.iter().map(|f| f.name()).join(", ");
            format!("Next Rules:\n- Scope {k} \n- Rules {rules}").blue()
          })
          .join("\n")
      );

      // Adds rules of scope != ["Parent", "Global"] to the stack
      self.add_rules_to_stack(
        &next_rules_by_scope,
        current_replace_range,
        rules_store,
        &mut next_rules_stack,
      );

      // Add Global rules as seed rules
      for r in &next_rules_by_scope[GLOBAL] {
        rules_store.add_to_global_rules(r);
      }

      // Process the parent
      // Find the rules to be applied in the "Parent" scope that match any parent (context) of the changed node in the previous edit
      if let Some(edit) =
        self.get_edit_for_ancestors(&current_replace_range, rules_store, &next_rules_by_scope)
      {
        self.rewrites_mut().push(edit.clone());
        // Apply the matched rule to the parent
        let applied_edit = self.apply_edit(&edit, parser);
        current_replace_range = get_replace_range(applied_edit);
        current_rule = edit.matched_rule().to_string();
        // Add the (tag, code_snippet) mapping to substitution table.
        self.substitutions.extend(edit.p_match().matches().clone());
      } else {
        break;
      }
    }
    // Apply the next rules from the stack
    for (sq, rle) in &next_rules_stack {
      self.apply_rule(rle.clone(), rules_store, parser, &Some(sq.clone()));
    }
  }

  /// Adds the "Method" and "Class" scoped next rules to the queue.
  fn add_rules_to_stack(
    &mut self, next_rules_by_scope: &HashMap<String, Vec<InstantiatedRule>>,
    current_match_range: Range, rules_store: &mut RuleStore,
    stack: &mut VecDeque<(CGPattern, InstantiatedRule)>,
  ) {
    for (scope_level, rules) in next_rules_by_scope {
      // Scope level is not "Parent", "ParentIterative" or "Global"
      if ![PARENT, PARENT_ITERATIVE, GLOBAL].contains(&scope_level.as_str()) {
        for rule in rules {
          let scope_query = self.get_scope_query(
            scope_level,
            *current_match_range.start_byte(),
            *current_match_range.end_byte(),
            rules_store,
          );
          // Add Method and Class scoped rules to the queue
          stack.push_front((scope_query, rule.clone()));
        }
      }
    }
  }

  fn get_scope_node(&self, scope_query: &Option<CGPattern>, rules_store: &mut RuleStore) -> Node {
    // Get scope node
    // let mut scope_node = self.root_node();
    if let Some(query_str) = scope_query {
      // Apply the scope query in the source code and get the appropriate node
      let scope_pattern = rules_store.query(query_str);
      if let Some(p_match) = scope_pattern.get_match(&self.root_node(), self.code(), true) {
        return get_node_for_range(
          self.root_node(),
          *p_match.range().start_byte(),
          *p_match.range().end_byte(),
        );
      }
    }
    self.root_node()
  }

  /// Apply all `rules` sequentially.
  pub(crate) fn apply_rules(
    &mut self, rules_store: &mut RuleStore, rules: &[InstantiatedRule], parser: &mut Parser,
    scope_query: Option<CGPattern>,
  ) {
    for rule in rules {
      self.apply_rule(rule.to_owned(), rules_store, parser, &scope_query)
    }
    self.perform_delete_consecutive_new_lines();
  }

  /// Applies an edit to the source code unit
  /// # Arguments
  /// * `replace_range` - the range of code to be replaced
  /// * `replacement_str` - the replacement string
  /// * `parser`
  ///
  /// # Returns
  /// The `edit:InputEdit` performed.
  ///
  /// Note - Causes side effect. - Updates `self.ast` and `self.code`
  pub(crate) fn apply_edit(&mut self, edit: &Edit, parser: &mut Parser) -> InputEdit {
    // Get the tree_sitter's input edit representation
    let (new_source_code, ts_edit) = get_tree_sitter_edit(self.code.clone(), edit);
    // Apply edit to the tree
    let number_of_errors = self._number_of_errors();
    self.ast.edit(&ts_edit);
    self._replace_file_contents_and_re_parse(&new_source_code, parser, true);

    // Panic if the number of errors increased after the edit
    if self._number_of_errors() > number_of_errors {
      self._panic_for_syntax_error();
    }
    ts_edit
  }

  fn _panic_for_syntax_error(&self) {
    let msg = format!(
      "Produced syntactically incorrect source code {}",
      self.code()
    );
    panic!("{}", msg);
  }

  /// Returns the number of errors in this source code unit
  fn _number_of_errors(&self) -> usize {
    number_of_errors(&self.root_node())
  }

  // Replaces the content of the current file with the new content and re-parses the AST
  /// # Arguments
  /// * `replacement_content` - new content of file
  /// * `parser`
  /// * `is_current_ast_edited` : have you invoked `edit` on the current AST ?
  /// Note - Causes side effect. - Updates `self.ast` and `self.code`
  pub(crate) fn _replace_file_contents_and_re_parse(
    &mut self, replacement_content: &str, parser: &mut Parser, is_current_ast_edited: bool,
  ) {
    let prev_tree = if is_current_ast_edited {
      Some(&self.ast)
    } else {
      None
    };
    // Create a new updated tree from the previous tree
    let new_tree = parser
      .parse(replacement_content, prev_tree)
      .expect("Could not generate new tree!");
    self.ast = new_tree;
    self.code = replacement_content.to_string();
  }

  pub(crate) fn global_substitutions(&self) -> HashMap<String, String> {
    self
      .substitutions()
      .iter()
      .filter(|e| e.0.starts_with(self.piranha_arguments.global_tag_prefix()))
      .map(|(a, b)| (a.to_string(), b.to_string()))
      .collect()
  }
}

#[cfg(test)]
#[path = "unit_tests/source_code_unit_test.rs"]
mod source_code_unit_test;
