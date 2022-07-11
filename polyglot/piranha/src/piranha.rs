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

use std::{collections::HashMap, path::PathBuf};

use colored::Colorize;
use itertools::Itertools;
use jwalk::WalkDir;
use log::info;
use regex::Regex;
use tree_sitter::Parser;

use crate::{
  models::{
    piranha_arguments::PiranhaArguments, rule_store::RuleStore, source_code_unit::SourceCodeUnit,
  },
  utilities::read_file,
};

use crate::models::scopes::ScopeGenerator;
use crate::utilities::tree_sitter_utilities::PiranhaHelpers;
use crate::{
  models::{
    rule::Rule,
    rule_store::{GLOBAL, PARENT},
  },
  utilities::tree_sitter_utilities::get_node_for_range,
};

use std::collections::VecDeque;
use tree_sitter::{InputEdit, Node};

pub(crate) fn execute_piranha(args: &PiranhaArguments) -> Vec<SourceCodeUnit> {
  let mut flag_cleaner = FlagCleaner::new(args);
  flag_cleaner.perform_cleanup();
  flag_cleaner.get_updated_files()
}

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
      // Apply edit_1
      let applied_ts_edit = self.apply_edit(&edit_1, parser);

      // Add all the (code_snippet, tag) mapping to the substitution table.
      self.add_to_substitutions(edit_1.matches());

      let mut current_edit = applied_ts_edit;
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
        if let Some(edit) = Rule::get_edit_for_context(
          &self.clone(),
          current_edit.start_byte,
          current_edit.new_end_byte,
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
          let scope_query = ScopeGenerator::get_scope_query(
            self.clone(),
            scope_level,
            current_edit.start_byte,
            current_edit.new_end_byte,
            rules_store,
          );
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
  fn apply_rules(
    &mut self, rules_store: &mut RuleStore, rules: &[Rule], parser: &mut Parser,
    scope_query: Option<String>,
  ) {
    for rule in rules {
      self.apply_rule(rule.to_owned(), rules_store, parser, &scope_query)
    }
  }
}

// Maintains the state of Piranha and the updated content of files in the source code.
struct FlagCleaner {
  // Maintains Piranha's state
  rule_store: RuleStore,
  // Path to source code folder
  path_to_codebase: String,
  // Files updated by Piranha.
  relevant_files: HashMap<PathBuf, SourceCodeUnit>,
}

impl FlagCleaner {
  fn get_updated_files(&self) -> Vec<SourceCodeUnit> {
    self.relevant_files.values().cloned().collect_vec()
  }

  /// Performs cleanup related to stale flags
  fn perform_cleanup(&mut self) {
    // Setup the parser for the specific language
    let mut parser = Parser::new();
    parser
      .set_language(self.rule_store.language())
      .expect("Could not set the language for the parser.");

    // Keep looping until new `global` rules are added.
    loop {
      let current_rules = self.rule_store.global_rules();

      info!("{}", format!("# Global rules {}", current_rules.len()));
      // Iterate over each file containing the usage of the feature flag API
      for (path, content) in self.get_files_containing_feature_flag_api_usage() {
        self
          .relevant_files
          // Get the content of the file for `path` from the cache `relevant_files`
          .entry(path.to_path_buf())
          // Populate the cache (`relevant_files`) with the content, in case of cache miss (lazily)
          .or_insert_with(|| {
            // Create new source code unit
            SourceCodeUnit::new(
              &mut parser,
              content,
              &self.rule_store.input_substitutions(),
              path.as_path(),
            )
          })
          // Apply the rules to this file
          .apply_rules(&mut self.rule_store, &current_rules, &mut parser, None);

        // Break when a new `global` rule is added
        if self.rule_store.global_rules().len() > current_rules.len() {
          info!("Found a new global rule. Will start scanning all the files again.");
          break;
        }
      }
      // If no new `global_rules` were added, break.
      if self.rule_store.global_rules().len() == current_rules.len() {
        break;
      }
    }
  }

  /// Gets all the files from the code base that (i) have the language appropriate file extension, and (ii) contains the grep pattern.
  /// Note that `WalkDir` traverses the directory with parallelism.
  fn get_files_containing_feature_flag_api_usage(&self) -> HashMap<PathBuf, String> {
    let pattern = self.get_grep_heuristics();
    info!("{}", format!("Searching pattern {}", pattern).green());
    let files: HashMap<PathBuf, String> = WalkDir::new(&self.path_to_codebase)
      // Walk over the entire code base
      .into_iter()
      // Ignore errors
      .filter_map(|e| e.ok())
      // Filter files with the desired extension
      .filter(|de| {
        de.path()
          .extension()
          .and_then(|e| {
            e.to_str()
              .filter(|x| x.eq(&self.rule_store.language_name()))
          })
          .is_some()
      })
      // Read the file
      .map(|f| (f.path(), read_file(&f.path()).unwrap()))
      // Filter the files containing the desired regex pattern
      .filter(|x| pattern.is_match(x.1.as_str()))
      .collect();
    #[rustfmt::skip]
    println!("{}", format!("Will parse and analyze {} files.", files.len()).green());
    files
  }

  /// Instantiate Flag-cleaner
  fn new(args: &PiranhaArguments) -> Self {
    let graph_rule_store = RuleStore::new(args);
    Self {
      rule_store: graph_rule_store,
      path_to_codebase: String::from(args.path_to_code_base()),
      relevant_files: HashMap::new(),
    }
  }

  /// To create the current set of global rules, certain substitutions were applied.
  /// This method creates a regex pattern matching these substituted values.
  ///
  /// At the directory level, we would always look to perform global rules. However this is expensive because
  /// it requires parsing each file. To overcome this, we apply this simple
  /// heuristic to find the (upper bound) files that would match one of our current global rules.
  /// This heuristic reduces the number of files to parse.
  ///
  fn get_grep_heuristics(&self) -> Regex {
    let reg_x = self
      .rule_store
      .global_rules()
      .iter()
      .flat_map(|r| r.grep_heuristics())
      .sorted()
      //Remove duplicates
      .dedup()
      //FIXME: Dirty trick to remove true and false. Ideally, grep heuristic could be a field in itself for a rule.
      // Since not all "holes" could be used as grep heuristic.
      .filter(|x| !x.as_str().eq("true") && !x.as_str().eq("false"))
      .join("|");
    Regex::new(reg_x.as_str()).unwrap()
  }
}
