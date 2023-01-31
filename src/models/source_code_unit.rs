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
use std::{
  collections::{HashMap, VecDeque},
  fs,
  path::{Path, PathBuf},
};

use colored::Colorize;
use itertools::Itertools;
use log::{debug, error, trace};
use regex::Regex;
use tree_sitter::{InputEdit, Node, Parser, Range, Tree};
use tree_sitter_traversal::{traverse, Order};

use crate::{
  models::rule_store::{GLOBAL, PARENT},
  utilities::tree_sitter_utilities::{
    get_context, get_node_for_range, get_replace_range, get_tree_sitter_edit, substitute_tags,
    PiranhaHelpers,
  },
};

use super::{
  constraint::Constraint, edit::Edit, matches::Match, piranha_arguments::PiranhaArguments,
  rule::InstantiatedRule, rule_store::RuleStore,
};
use getset::{CopyGetters, Getters, MutGetters};
// Maintains the updated source code content and AST of the file
#[derive(Clone, Getters, CopyGetters, MutGetters)]
pub(crate) struct SourceCodeUnit {
  // The tree representing the file
  ast: Tree,
  // The content of a file
  #[get = "pub"]
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
  piranha_arguments: PiranhaArguments,
}

impl SourceCodeUnit {
  pub(crate) fn new(
    parser: &mut Parser, code: String, substitutions: &HashMap<String, String>, path: &Path,
    piranha_arguments: &PiranhaArguments,
  ) -> Self {
    let ast = parser.parse(&code, None).expect("Could not parse code");
    Self {
      ast,
      code,
      substitutions: substitutions.clone(),
      path: path.to_path_buf(),
      rewrites: Vec::new(),
      matches: Vec::new(),
      piranha_arguments: piranha_arguments.clone(),
    }
  }

  pub(crate) fn root_node(&self) -> Node<'_> {
    self.ast.root_node()
  }

  /// Will apply the `rule` to all of its occurrences in the source code unit.
  fn apply_rule(
    &mut self, rule: InstantiatedRule, rules_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<String>,
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
    scope_query: &Option<String>,
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

        self.propagate(m.range(), rule.clone(), rule_store, parser);
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
    let mut next_rules_stack: VecDeque<(String, InstantiatedRule)> = VecDeque::new();
    // Perform the parent edits, while queueing the Method and Class level edits.
    // let file_level_scope_names = [METHOD, CLASS];
    loop {
      // Get all the (next) rules that could be after applying the current rule (`rule`).
      let next_rules_by_scope = rules_store.get_next(&current_rule, self.substitutions());

      debug!(
        "\n{}",
        &next_rules_by_scope
          .iter()
          .map(|(k, v)| {
            let rules = v.iter().map(|f| f.name()).join(", ");
            format!("Next Rules:\nScope {k} \nRules {rules}").blue()
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
      if let Some(edit) = self.get_edit_for_context(
        current_replace_range.start_byte,
        current_replace_range.end_byte,
        rules_store,
        &next_rules_by_scope[PARENT],
      ) {
        self.rewrites_mut().push(edit.clone());
        debug!(
          "\n{}",
          format!(
            "Cleaning up the context, by applying the rule - {}",
            edit.matched_rule()
          )
          .green()
        );
        // Apply the matched rule to the parent
        let applied_edit = self.apply_edit(&edit, parser);
        current_replace_range = get_replace_range(applied_edit);
        current_rule = edit.matched_rule().to_string();
        // Add the (tag, code_snippet) mapping to substitution table.
        self.substitutions.extend(edit.p_match().matches().clone());
      } else {
        // No more parents found for cleanup
        break;
      }
    }

    // Apply the next rules from the stack
    for (sq, rle) in &next_rules_stack {
      self.apply_rule(rle.clone(), rules_store, parser, &Some(sq.to_string()));
    }
  }

  /// Adds the "Method" and "Class" scoped next rules to the queue.
  fn add_rules_to_stack(
    &mut self, next_rules_by_scope: &HashMap<String, Vec<InstantiatedRule>>,
    current_match_range: Range, rules_store: &mut RuleStore,
    stack: &mut VecDeque<(String, InstantiatedRule)>,
  ) {
    for (scope_level, rules) in next_rules_by_scope {
      // Scope level is not "PArent" or "Global"
      if ![PARENT, GLOBAL].contains(&scope_level.as_str()) {
        for rule in rules {
          let scope_query = self.get_scope_query(
            scope_level,
            current_match_range.start_byte,
            current_match_range.end_byte,
            rules_store,
          );
          // Add Method and Class scoped rules to the queue
          stack.push_front((scope_query, rule.clone()));
        }
      }
    }
  }

  fn get_scope_node(&self, scope_query: &Option<String>, rules_store: &mut RuleStore) -> Node {
    // Get scope node
    // let mut scope_node = self.root_node();
    if let Some(query_str) = scope_query {
      // Apply the scope query in the source code and get the appropriate node
      let tree_sitter_scope_query = rules_store.query(query_str);
      if let Some(p_match) =
        &self
          .root_node()
          .get_match_for_query(self.code(), tree_sitter_scope_query, true)
      {
        return get_node_for_range(
          self.root_node(),
          p_match.range().start_byte,
          p_match.range().end_byte,
        );
      }
    }
    self.root_node()
  }

  /// Apply all `rules` sequentially.
  pub(crate) fn apply_rules(
    &mut self, rules_store: &mut RuleStore, rules: &[InstantiatedRule], parser: &mut Parser,
    scope_query: Option<String>,
  ) {
    for rule in rules {
      self.apply_rule(rule.to_owned(), rules_store, parser, &scope_query)
    }
    self.perform_delete_consecutive_new_lines();
  }

  /// Replaces three consecutive newline characters with two
  pub(crate) fn perform_delete_consecutive_new_lines(&mut self) {
    if *self.piranha_arguments.delete_consecutive_new_lines() {
      let regex = Regex::new(r"\n(\s*\n)+(\s*\n)").unwrap();
      let x = &regex.replace_all(self.code(), "\n${2}").into_owned();
      self.code = x.clone();
    }
  }

  /// Writes the current contents of `code` to the file system and deletes a file if empty.
  pub(crate) fn persist(&self, piranha_arguments: &PiranhaArguments) {
    if *piranha_arguments.dry_run() {
      return;
    }
    if self.code.as_str().is_empty() && *piranha_arguments.delete_file_if_empty() {
      fs::remove_file(&self.path).expect("Unable to Delete file");
      return;
    }
    fs::write(&self.path, self.code()).expect("Unable to Write file");
  }

  pub(crate) fn apply_edit(&mut self, edit: &Edit, parser: &mut Parser) -> InputEdit {
    // Get the tree_sitter's input edit representation
    let mut applied_edit = self._apply_edit(edit, parser);
    // Check if the edit kind is "DELETE something"
    if *self.piranha_arguments.cleanup_comments() && edit.replacement_string().is_empty() {
      let deleted_at = edit.p_match().range().start_point.row;
      if let Some(comment_range) = self.get_comment_at_line(
        deleted_at,
        *self.piranha_arguments.cleanup_comments_buffer(),
        edit.p_match().range().start_byte,
      ) {
        debug!("Deleting an associated comment");
        applied_edit = self._apply_edit(&Edit::delete_range(self.code(), comment_range), parser);
      }
    }
    applied_edit
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
  fn get_comment_at_line(&mut self, row: usize, buffer: usize, start_byte: usize) -> Option<Range> {
    // Get all nodes that start or end on `updated_row`.
    let mut relevant_nodes_found = false;
    let mut relevant_nodes_are_comments = true;
    let mut comment_range = None;
    // Since the previous edit was a delete, the start and end of the replacement range is [start_byte].
    let node = self
      .ast
      .root_node()
      .descendant_for_byte_range(start_byte, start_byte)
      .unwrap_or_else(|| self.ast.root_node());

    for node in traverse(node.walk(), Order::Post) {
      if node.start_position().row == row || node.end_position().row == row {
        relevant_nodes_found = true;
        let is_comment: bool = self
          .piranha_arguments
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
      return self.get_comment_at_line(row - 1, buffer - 1, start_byte);
    }
    None
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
  pub(crate) fn _apply_edit(&mut self, edit: &Edit, parser: &mut Parser) -> InputEdit {
    let mut edit_to_apply = edit.clone();
    // Check if the edit is a `Delete` operation then delete trailing comma
    if edit.replacement_string().trim().is_empty() {
      edit_to_apply = self.delete_trailing_comma(edit);
    }
    // Get the tree_sitter's input edit representation
    let (new_source_code, ts_edit) = get_tree_sitter_edit(self.code.clone(), &edit_to_apply);
    // Apply edit to the tree
    self.ast.edit(&ts_edit);
    self._replace_file_contents_and_re_parse(&new_source_code, parser, true);
    if self.ast.root_node().has_error() {
      let msg = format!(
        "Produced syntactically incorrect source code {}",
        self.code()
      );
      error!("{}", msg);
      panic!("{}", msg);
    }
    ts_edit
  }

  /// Deletes the trailing comma after the {deleted_range}
  /// # Arguments
  /// * `deleted_range` - the range of the deleted code
  ///
  /// # Returns
  /// code range of the closest node
  ///
  /// Algorithm:
  /// Get the node after the {deleted_range}'s end byte (heuristic 5 characters)
  /// Traverse this node and get the node closest to the range {deleted_range}'s end byte
  /// IF this closest node is a comma, extend the {new_delete_range} to include the comma.
  fn delete_trailing_comma(&self, edit: &Edit) -> Edit {
    let deleted_range: Range = edit.p_match().range();
    let mut new_deleted_range = deleted_range;

    // Get the node immediately after the to-be-deleted code
    if let Some(parent_node) = self
      .ast
      .root_node()
      .descendant_for_byte_range(deleted_range.end_byte, deleted_range.end_byte + 1)
      .and_then(|n| n.parent())
    {
      // Traverse this `parent_node` to find the closest next node after the `replace_range`
      if let Some(node_after_to_be_deleted_node) = traverse(parent_node.walk(), Order::Post)
        .filter(|n| n.start_byte() >= deleted_range.end_byte)
        .min_by(|a, b| {
          (a.start_byte() - deleted_range.end_byte).cmp(&(b.start_byte() - deleted_range.end_byte))
        })
      {
        // If the next closest node to the "to be deleted node" is a comma , extend the
        // the deletion range to include the comma
        if node_after_to_be_deleted_node
          .utf8_text(self.code().as_bytes())
          .unwrap()
          .trim()
          .eq(",")
        {
          new_deleted_range.end_byte = node_after_to_be_deleted_node.end_byte();
          new_deleted_range.end_point = node_after_to_be_deleted_node.end_position();
        }
      }
    }
    return Edit::new(
      Match::new(
        self.code()[new_deleted_range.start_byte..new_deleted_range.end_byte].to_string(),
        new_deleted_range,
        edit.p_match().matches().clone(),
      ),
      edit.replacement_string().to_string(),
      edit.matched_rule().to_string(),
    );
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

  // Apply all the `rules` to the node, parent, grand parent and great grand parent.
  // Short-circuit on the first match.
  pub(crate) fn get_edit_for_context(
    &self, previous_edit_start: usize, previous_edit_end: usize, rules_store: &mut RuleStore,
    rules: &Vec<InstantiatedRule>,
  ) -> Option<Edit> {
    let number_of_ancestors_in_parent_scope = *rules_store
      .piranha_args()
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
  pub(crate) fn get_matches(
    &self, rule: &InstantiatedRule, rule_store: &mut RuleStore, node: Node, recursive: bool,
  ) -> Vec<Match> {
    let mut output: Vec<Match> = vec![];
    // Get all matches for the query in the given scope `node`.
    let replace_node_tag = if rule.rule().is_match_only_rule() || rule.rule().is_dummy_rule() {
      None
    } else {
      Some(rule.replace_node())
    };
    let all_query_matches = node.get_all_matches_for_query(
      self.code().to_string(),
      rule_store.query(&rule.query()),
      recursive,
      replace_node_tag,
    );

    // Return the first match that satisfies constraint of the rule
    for p_match in all_query_matches {
      let matched_node = get_node_for_range(
        self.root_node(),
        p_match.range().start_byte,
        p_match.range().end_byte,
      );

      if self.is_satisfied(matched_node, rule, p_match.matches(), rule_store) {
        trace!("Found match {:#?}", p_match);
        output.push(p_match);
      }
    }
    debug!("Matches found {}", output.len());
    output
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
        let replacement_string = substitute_tags(&rule.replace(), p_match.matches(), false);
        let edit = Edit::new(p_match.clone(), replacement_string, rule.name());
        trace!("Rewrite found : {:#?}", edit);
        edit
      });
  }

  /// Generate a tree-sitter based query representing the scope of the previous edit.
  /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
  pub(crate) fn get_scope_query(
    &self, scope_level: &str, start_byte: usize, end_byte: usize, rules_store: &mut RuleStore,
  ) -> String {
    let root_node = self.root_node();
    let mut changed_node = get_node_for_range(root_node, start_byte, end_byte);
    // Get the scope matchers for `scope_level` from the `scope_config.toml`.
    let scope_matchers = rules_store.get_scope_query_generators(scope_level);

    // Match the `scope_matcher.matcher` to the parent
    loop {
      trace!(
        "Getting scope {} for node kind {}",
        scope_level,
        changed_node.kind()
      );
      for m in &scope_matchers {
        if let Some(p_match) =
          changed_node.get_match_for_query(self.code(), rules_store.query(m.matcher()), false)
        {
          // Generate the scope query for the specific context by substituting the
          // the tags with code snippets appropriately in the `generator` query.
          return substitute_tags(m.generator(), p_match.matches(), true);
        }
      }
      if let Some(parent) = changed_node.parent() {
        changed_node = parent;
      } else {
        break;
      }
    }
    panic!("Could not create scope query for {scope_level:?}");
  }

  fn is_satisfied(
    &self, node: Node, rule: &InstantiatedRule, substitutions: &HashMap<String, String>,
    rule_store: &mut RuleStore,
  ) -> bool {
    let mut updated_substitutions = rule_store.piranha_args().input_substitutions().clone();
    updated_substitutions.extend(substitutions.clone());
    rule.constraints().iter().all(|constraint| {
      self._is_satisfied(constraint.clone(), node, rule_store, &updated_substitutions)
    })
  }

  /// Checks if the node satisfies the constraints.
  /// Constraint has two parts (i) `constraint.matcher` (ii) `constraint.query`.
  /// This function traverses the ancestors of the given `node` until `constraint.matcher` matches
  /// i.e. finds scope for constraint.
  /// Within this scope it checks if the `constraint.query` DOES NOT MATCH any sub-tree.
  fn _is_satisfied(
    &self, constraint: Constraint, node: Node, rule_store: &mut RuleStore,
    substitutions: &HashMap<String, String>,
  ) -> bool {
    let mut current_node = node;
    // This ensures that the below while loop considers the current node too when checking for constraints.
    // It does not make sense to check for constraint if current node is a "leaf" node.
    if node.child_count() > 0 {
      current_node = node.child(0).unwrap();
    }
    // Get the scope_node of the constraint (`scope.matcher`)
    let mut matched_matcher = false;
    while let Some(parent) = current_node.parent() {
      let matcher_query_str = substitute_tags(constraint.matcher(), substitutions, true);
      if let Some(p_match) =
        parent.get_match_for_query(self.code(), rule_store.query(&matcher_query_str), false)
      {
        matched_matcher = true;
        let scope_node = get_node_for_range(
          self.root_node(),
          p_match.range().start_byte,
          p_match.range().end_byte,
        );
        for query_with_holes in constraint.queries() {
          let query_str = substitute_tags(query_with_holes, substitutions, true);
          let query = &rule_store.query(&query_str);
          // If this query matches anywhere within the scope, return false.
          if scope_node
            .get_match_for_query(self.code(), query, true)
            .is_some()
          {
            return false;
          }
        }
        break;
      }
      current_node = parent;
    }
    matched_matcher
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
