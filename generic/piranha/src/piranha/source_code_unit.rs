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

use crate::models::rule::{Rule, RuleHelper};
use crate::piranha::rule_store::{RuleStore, GLOBAL, PARENT};
use crate::utilities::tree_sitter_utilities::{PiranhaRuleMatcher, TreeSitterHelpers};
use colored::Colorize;
use log::info;
use std::collections::VecDeque;
use std::{collections::HashMap, fs, path::PathBuf};
use tree_sitter::{InputEdit, Node, Parser, Point, Range, Tree};

// Maintains the updated source code content and AST of the file
#[derive(Clone)]
pub struct SourceCodeUnit {
  // The tree representing the file
  ast: Tree,
  // The content of a file
  code: String,
  // The tag substitution cache.
  // This map is looked up to instantiate new rules.
  substitutions: HashMap<String, String>,
  // The path to the source code.
  path: PathBuf,
}

impl SourceCodeUnit {
  /// Writes the current contents of `code` to the file system.
  pub fn persist(&self) {
    fs::write(&self.path, self.code.as_str()).expect("Unable to Write file");
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
  fn apply_edit(
    &mut self, replace_range: Range, replacement_str: String, parser: &mut Parser,
  ) -> InputEdit {
    // Get the tree_sitter's input edit representation
    let (new_source_code, edit) = self.get_edit(replace_range, &replacement_str);
    // Apply edit to the tree
    self.ast.edit(&edit);
    // Create a new updated tree from the previous tree
    let new_tree = parser
      .parse(&new_source_code, Some(&self.ast))
      .expect("Could not generate new tree!");
    self.ast = new_tree;
    self.code = new_source_code;
    edit
  }

  /// Will apply the `rule` to all of its occurrences in the source code unit.
  fn apply_rule(
    &mut self, rule: Rule, rules_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<String>,
  ) {
    loop {
      if !self.apply_rule_to_first_match(rule.clone(), rules_store, parser, scope_query) {
        break;
      }
    }
  }

  // Applies the rule to the first match in the source code
  fn apply_rule_to_first_match(
    &mut self, rule: Rule, rules_store: &mut RuleStore, parser: &mut Parser,
    scope_query: &Option<String>,
  ) -> bool {
    let scope_node = self.get_scope_node(scope_query, rules_store);

    let mut any_match = false;

    // Match the rule "anywhere" inside the scope_node
    if let Some((range, rpl, code_snippets_by_tag)) =
      self.get_any_match_for_rule(&rule, rules_store, scope_node, true)
    {
      any_match = true;
      // Get the edit for applying the matched rule to the source code
      let edit = self.apply_edit(range, rpl, parser);

      // Add all the (code_snippet, tag) mapping to the substitution table.
      self.substitutions.extend(code_snippets_by_tag);

      let mut current_edit = edit;
      let mut current_rule = rule.clone();
      let mut next_rules_stack: VecDeque<(String, Rule)> = VecDeque::new();

      // Perform the parent edits, while queueing the Method and Class level edits.
      // let file_level_scope_names = [METHOD, CLASS];
      loop {
        // Get all the (next) rules that could be after applying the current rule (`rule`).
        let next_rules_by_scope = rules_store.get_next(&current_rule, &self.substitutions);

        // Adds "Method" and "Class" rules to the stack
        self.add_rules_to_stack(
          &next_rules_by_scope,
          current_edit,
          rules_store,
          &mut next_rules_stack,
        );

        // Add Global rules as seed rules
        for r in &next_rules_by_scope[GLOBAL] {
          rules_store.add_to_global_rules(r, &self.substitutions);
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
          self.substitutions.extend(code_snippets_by_tag_parent_rule);
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
            rule.instantiate(&self.substitutions),
          ));
        }
      }
    }
  }

  fn get_scope_node(&self, scope_query: &Option<String>, rules_store: &mut RuleStore) -> Node {
    // Get scope node
    let mut scope_node = self.ast.root_node();
    if let Some(query_str) = scope_query {
      // Apply the scope query in the source code and get the appropriate node
      let tree_sitter_scope_query = rules_store.get_query(query_str);
      if let Some((range, _)) =
        &self
          .ast
          .root_node()
          .get_match_for_query(&self.code, tree_sitter_scope_query, true)
      {
        scope_node = self.get_node_for_range(range.start_byte, range.end_byte);
      }
    }
    scope_node
  }

  /// Apply all `rules` sequentially.
  pub(crate) fn apply_rules(
    &mut self, rules_store: &mut RuleStore, rules: &Vec<Rule>, parser: &mut Parser,
    scope_query: Option<String>,
  ) {
    for rule in rules.clone() {
      self.apply_rule(rule.clone(), rules_store, parser, &scope_query)
    }
  }

  /// Get the smallest node within `self` that spans the given range.
  fn get_node_for_range(&self, start_byte: usize, end_byte: usize) -> Node {
    self
      .ast
      .root_node()
      .descendant_for_byte_range(start_byte, end_byte)
      .unwrap()
  }

  /// Generate a tree-sitter based query representing the scope of the previous edit.
  /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
  fn get_scope_query(
    &self, scope_level: &str, previous_edit: InputEdit, rules_store: &mut RuleStore,
  ) -> String {
    let mut changed_node =
      self.get_node_for_range(previous_edit.start_byte, previous_edit.new_end_byte);

    // Get the scope matchers for `scope_level` from the `scope_config.toml`.
    let scope_matchers = rules_store.get_scope_query_generators(scope_level);

    // Match the `scope_matcher.matcher` to the parent
    while let Some(parent) = changed_node.parent() {
      for m in &scope_matchers {
        if let Some((_, captures_by_tag)) =
          parent.get_match_for_query(&self.code, rules_store.get_query(&m.matcher()), false)
        {
          // Generate the scope query for the specific context by substituting the
          // the tags with code snippets appropriately in the `generator` query.
          return m.generator().substitute_tags(&captures_by_tag);
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
    let context = || self.get_context(previous_edit);
    for rule in rules {
      for ancestor in &context() {
        if let Some((range, replacement, captures_by_tag)) =
          self.get_any_match_for_rule(rule, rules_store, *ancestor, false)
        {
          return Some((range, replacement, rule.clone(), captures_by_tag));
        }
      }
    }
    None
  }

  /// Returns the node, its parent, grand parent and great grand parent
  fn get_context(&self, previous_edit: InputEdit) -> Vec<Node> {
    let changed_node =
      self.get_node_for_range(previous_edit.start_byte, previous_edit.new_end_byte);
    // Add parent of the changed node to the context
    let mut context = vec![changed_node];
    if let Some(parent) = changed_node.parent() {
      context.push(parent);
      // Add grand parent of the changed node to the context
      if let Some(grand_parent) = parent.parent() {
        context.push(grand_parent);
        // Add great grand parent of the changed node to the context
        if let Some(great_grand_parent) = grand_parent.parent() {
          context.push(great_grand_parent);
        }
      }
    }
    context
  }

  pub(crate) fn new(
    parser: &mut Parser, code: String, substitutions: &HashMap<String, String>, path: &PathBuf,
  ) -> Self {
    let ast = parser.parse(&code, None).expect("Could not parse code");
    Self {
      ast,
      code,
      substitutions: substitutions.clone(),
      path: path.to_path_buf(),
    }
  }

  /// Gets the first match for the rule in `self`
  fn get_any_match_for_rule(
    &self, rule: &Rule, rule_store: &mut RuleStore, node: Node, recursive: bool,
  ) -> Option<(Range, String, HashMap<String, String>)> {
    // Get all matches for the query in the given scope `node`.
    let all_query_matches = node.get_all_matches_for_query(
      self.code.clone(),
      rule_store.get_query(&rule.get_query()),
      recursive,
      Some(rule.replace_node()),
    );

    // Return the first match that satisfies constraint of the rule
    for (range, tag_substitutions) in all_query_matches {
      let matched_node = self.get_node_for_range(range.start_byte, range.end_byte);
      if self.satisfies_constraint(matched_node, rule, &tag_substitutions, rule_store) {
        let replacement = rule.replace().substitute_tags(&tag_substitutions);
        return Some((range, replacement, tag_substitutions));
      }
    }
    None
  }

  /// Checks if the node satisfies the constraints.
  /// Constraint has two parts (i) `constraint.matcher` (ii) `constraint.query`.
  /// This function traverses the ancestors of the given `node` until `constraint.matcher` matches i.e. finds scope for constraint.
  /// Within this scope it checks if the `constraint.query` DOES NOT MATCH any sub-tree.
  fn satisfies_constraint(
    &self, node: Node, rule: &Rule, capture_by_tags: &HashMap<String, String>,
    rule_store: &mut RuleStore,
  ) -> bool {
    let mut current_node = node;
    // Get the scope of the predicate
    for constraint in rule.constraints() {
      // Loop till you find a parent for the current node
      while let Some(parent) = current_node.parent() {
        // Check if the parent matches the `rule.constraint.matcher`
        // This is the scope in which the constraint query will be applied.
        if let Some((range, _)) = parent.get_match_for_query(
          &self.code,
          &constraint.matcher().create_query(rule_store.language()),
          false,
        ) {
          let scope_node = self.get_node_for_range(range.start_byte, range.end_byte);
          // Apply each query within the `scope_node`
          for query_with_holes in constraint.queries() {
            let query_str = query_with_holes.substitute_tags(capture_by_tags);
            let query = &rule_store.get_query(&query_str);
            // If this query matches anywhere within the scope, return false.
            if scope_node
              .get_match_for_query(&self.code, query, true)
              .is_some()
            {
              return false;
            }
          }
          break;
        }
        current_node = parent;
      }
    }
    true
  }

  /// Replaces the given byte range (`replace_range`) with the `replacement`.
  /// Returns tree-sitter's edit representation along with updated source code.
  /// Note: This method does not update `self`.
  pub fn get_edit(&self, replace_range: Range, replacement: &str) -> (String, InputEdit) {
    let original_source_code = self.code.clone();

    // Create the new source code content by appropriately
    // replacing the range with the replacement string.
    let updated_source_code = [
      &original_source_code[..replace_range.start_byte],
      replacement,
      &original_source_code[replace_range.end_byte..],
    ]
    .concat();

    // Log the edit
    let replaced_code_snippet =
      &original_source_code[replace_range.start_byte..replace_range.end_byte];

    #[rustfmt::skip]
    info!("{} at ({:?}) -\n {}", if replacement.is_empty() { "Delete code" } else {"Update code" }.green(), ((&replace_range.start_point.row, &replace_range.start_point.column), (&replace_range.end_point.row, &replace_range.end_point.column)), if !replacement.is_empty() {format!("{}\n to \n{}",replaced_code_snippet.italic(),replacement.italic())} else {format!("{} ", replaced_code_snippet.italic())});

    let original_bytes = &original_source_code.as_bytes().to_vec();

    let edit =
      Self::get_tree_sitter_edit(replace_range, replacement.as_bytes().len(), original_bytes);

    (updated_source_code, edit)
  }

  // Finds the position (col and row number) for a given offset.
  fn position_for_offset(input: &Vec<u8>, offset: usize) -> Point {
    let mut result = Point { row: 0, column: 0 };
    for c in &input[0..offset] {
      if *c as char == '\n' {
        result.row += 1;
        result.column = 0;
      } else {
        result.column += 1;
      }
    }
    result
  }

  // Creates the InputEdit as per the tree-sitter api documentation.
  fn get_tree_sitter_edit(
    replace_range: Range, len_of_replacement: usize, source_code_bytes: &Vec<u8>,
  ) -> InputEdit {
    let start_byte = replace_range.start_byte;
    let old_end_byte = replace_range.end_byte;
    let new_end_byte = start_byte + len_of_replacement;
    InputEdit {
      start_byte,
      old_end_byte,
      new_end_byte,
      start_position: Self::position_for_offset(source_code_bytes, start_byte),
      old_end_position: Self::position_for_offset(source_code_bytes, old_end_byte),
      new_end_position: Self::position_for_offset(source_code_bytes, new_end_byte),
    }
  }

  #[cfg(test)] // Rust analyzer FP
  pub fn code(&self) -> String {
    String::from(&self.code)
  }
  #[cfg(test)] // Rust analyzer FP
  pub fn path(&self) -> &PathBuf {
    &self.path
  }
}
