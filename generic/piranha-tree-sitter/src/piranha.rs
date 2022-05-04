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

//! This module contains the core logic for Piranha.
use crate::config::{
    command_line_arguments::PiranhaArguments, Rule, RuleStore, CLASS, GLOBAL, METHOD, PARENT,
};
use crate::tree_sitter::{PiranhaRuleMatcher, TreeSitterHelpers};
use crate::utilities::read_file;
use colored::Colorize;
use itertools::Itertools;
use jwalk::WalkDir;
use log::info;
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf};
use tree_sitter::{InputEdit, Node, Parser, Point, Range, Tree};

// Maintains the state of Piranha and the updated content of files in the source code.
pub struct FlagCleaner {
    // Maintains Piranha's state
    rule_store: RuleStore,
    // Path to source code folder
    path_to_codebase: String,
    // Files updated by Piranha.
    relevant_files: HashMap<PathBuf, SourceCodeUnit>,
}

impl FlagCleaner {
    
    /// Getter for `relevant_files`
    pub fn get_updated_files(&self) -> Vec<SourceCodeUnit>  {
        self.relevant_files.values().cloned().collect_vec()
    }

    /// Performs cleanup related to stale flags
    pub fn perform_cleanup(&mut self) {
        // Setup the parser for the specific language
        let mut parser = Parser::new();
        parser
            .set_language(self.rule_store.language())
            .expect("Could not set the language for the parser.");

        // Keep looping until new `global` rules are added.
        loop {
            let current_rules = self.rule_store.global_rules();

            info!("{}", format!("Number of global rules {}", current_rules.len()));
            // Iterate over each file containing the usage of the feature flag API
            for (path, content) in self.get_files_containing_feature_flag_api_usage() {
                self.relevant_files
                    // Get the content of the file for `path` from the cache `relevant_files`
                    .entry(path.to_path_buf())
                    // Populate the cache (`relevant_files`) with the content, in case of cache miss (lazily)
                    .or_insert_with(|| {
                        // Create new source code unit
                        SourceCodeUnit::new(
                            &mut parser,
                            content,
                            &self.rule_store.input_substitutions(),
                            &path,
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
    pub fn get_files_containing_feature_flag_api_usage(&self) -> HashMap<PathBuf, String> {
        let pattern = self.get_grep_heuristics();
        info!(
            "{}",
            format!("Searching for pattern {}", pattern.as_str()).green()
        );
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
            .map(|f| (f.path().to_path_buf(), read_file(&f.path().to_path_buf()).unwrap()))
            // Filter the files containing the desired regex pattern
            .filter(|x| pattern.is_match(x.1.as_str()))
            .collect();
        #[rustfmt::skip]
        info!("{}", format!("Will parse and analyze {} files.", files.len()).green());
        return files;
    }

    /// Instantiate PiranhaArguments
    pub fn new(args: PiranhaArguments) -> Self {
        let graph_rule_store = RuleStore::new(&args);
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
    pub fn get_grep_heuristics(&self) -> Regex {
        let reg_x = self
            .rule_store
            .global_rules()
            .iter()
            .flat_map(|r| r.grep_heuristics().unwrap().iter())
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
        fs::write(&self.path.to_path_buf(), self.code.as_str()).expect("Unable to Write file");
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
        &mut self,
        replace_range: Range,
        replacement_str: String,
        parser: &mut Parser,
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
        return edit;
    }

    /// Will apply the `rule` to all of its occurrences in the source code unit.
    fn apply_rule(
        &mut self,
        rule: Rule,
        rules_store: &mut RuleStore,
        parser: &mut Parser,
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
        &mut self,
        rule: Rule,
        rules_store: &mut RuleStore,
        parser: &mut Parser,
        scope_query: &Option<String>,
    ) -> bool {
        // Get scope node
        let mut scope_node = self.ast.root_node();
        if let Some(query_str) = scope_query {
            // Apply the scope query in the source code and get the appropriate node 
            let tree_sitter_scope_query = rules_store.get_query(query_str);
            if let Some((range, _)) = &self.ast.root_node().get_match_for_query(
                &self.code,
                tree_sitter_scope_query,
                true,
            ) {
                scope_node = self.get_node_for_range(range.start_byte, range.end_byte);
            }
        }

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

            let mut current_edit = edit.clone();
            let mut current_rule = rule.clone();
            let mut file_level_next_rules = vec![];

            // perform the parent edits, while queueing the Method and Class level edits.
            let file_level_scope_names = [METHOD, CLASS];
            loop {
                // Get all the (next) rules that could be after applying the current rule (`rule`)
                let next_rules_by_scope = rules_store.get_next(&current_rule, &self.substitutions);
                for (scope_level, rules) in &next_rules_by_scope {
                    // Scope level will be "Class" or "Method"
                    if file_level_scope_names.contains(&scope_level.as_str()) {
                        for rule in rules {
                            // Generate the scope query for the previously applied edit 
                            // This query will precisely capture the enclosing method / class.
                            let scope_query =
                                self.get_scope_query(scope_level, current_edit, rules_store);
                            // Add Method and Class scoped rules to the queue
                            file_level_next_rules.push((
                                scope_query.to_string(),
                                rule.instantiate(&self.substitutions),
                            ));
                        }
                    }
                }
                // Add Global rules as seed rules
                for r in &next_rules_by_scope[GLOBAL] {
                    rules_store.add_global_rule(r, &self.substitutions);
                }

                // Process the parent
                // Find the rules to be applied in the "Parent" scope that match any parent (context) of the changed node in the previous edit
                if let Some((c_range, replacement_str, matched_rule, code_snippets_by_tag_parent_rule)) = self
                    .match_rules_to_context(current_edit, rules_store, &next_rules_by_scope[PARENT])
                {
                    info!("{}", format!("Matched parent for cleanup").green());
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
            // Apply the method and class level rules
            file_level_next_rules.reverse();
            for (sq, rle) in file_level_next_rules {
                self.apply_rule(rle, rules_store, parser, &Some(sq.to_string()));
            }
        }
        return any_match;
    }

    /// Apply all `rules` sequentially.
    fn apply_rules(
        &mut self,
        rules_store: &mut RuleStore,
        rules: &Vec<Rule>,
        parser: &mut Parser,
        scope_query: Option<String>,
    ) {
        for rule in rules.clone() {
            self.apply_rule(rule.clone(), rules_store, parser, &scope_query)
        }
    }

    /// Get the smallest node within `self` that spans the given range.
    fn get_node_for_range(&self, start_byte: usize, end_byte: usize) -> Node {
        self.ast
            .root_node()
            .descendant_for_byte_range(start_byte, end_byte)
            .unwrap()
    }

    /// Generate a tree-sitter based query representing the scope of the previous edit.
    /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
    fn get_scope_query(
        &self,
        scope_level: &str,
        previous_edit: InputEdit,
        rules_store: &mut RuleStore,
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
        &mut self,
        previous_edit: InputEdit,
        rules_store: &mut RuleStore,
        rules: &Vec<Rule>,
    ) -> Option<(Range, String, Rule, HashMap<String, String>)> {
        // Context contains -  the changed node in the previous edit, its's parent, grand parent and great grand parent
        let context = || self.get_context(previous_edit);
        for rule in rules {
            for ancestor in &context() {
                if let Some((range, replacement, captures_by_tag)) =
                    self.get_any_match_for_rule(&rule, rules_store, ancestor.clone(), false)
                {
                    return Some((range, replacement, rule.clone(), captures_by_tag));
                }
            }
        }
        return None;
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

    fn new(
        parser: &mut Parser,
        code: String,
        substitutions: &HashMap<String, String>,
        path: &PathBuf,
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
        &self,
        rule: &Rule,
        rule_store: &mut RuleStore,
        node: Node,
        recursive: bool,
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
            if self.satisfies_constraint(matched_node, &rule, &tag_substitutions, rule_store) {
                let replacement = rule.replace().substitute_tags(&tag_substitutions);
                return Some((range.clone(), replacement, tag_substitutions));
            }
        }
        None
    }

    /// Checks if the node satisfies the constraints.
    /// Constraint has two parts (i) `constraint.matcher` (ii) `constraint.query`.
    /// This function traverses the ancestors of the given `node` until `constraint.matcher` matches i.e. finds scope for constraint.
    /// Within this scope it checks if the `constraint.query` DOES NOT MATCH any subtree.
    fn satisfies_constraint(
        &self,
        node: Node,
        rule: &Rule,
        capture_by_tags: &HashMap<String, String>,
        rule_store: &mut RuleStore,
    ) -> bool {
        if let Some(constraints) = rule.constraints() {
            let mut current_node = node;
            // Get the scope of the predicate
            for constraint in constraints {
                // Loop till you find a parent for the current node
                while let Some(parent) = current_node.parent() {
                    // Check if the parent matches the `rule.constraint.matcher`
                    // This is the scope in which the constraint query will be applied.
                    if let Some((range, _)) = parent.get_match_for_query(
                        &self.code,
                        &constraint
                            .matcher()
                            .create_query(rule_store.language()),
                        false,
                    ) {
                        let scope_node = self.get_node_for_range(range.start_byte, range.end_byte);
                        // Apply each query within the `scope_node`
                        for q in constraint.queries() {
                            let query_str = q.substitute_tags(&capture_by_tags);
                            let query = &rule_store.get_query(&query_str);
                             // If this query matches anywhere within the scope, return false.
                            if scope_node.get_match_for_query(&self.code, query, true).is_some() {
                                return false;
                            }
                        }
                        break;
                    }
                    current_node = parent;
                }
            }
        }
        true
    }

    /// Replaces the given byte range (`replace_range`) with the `replacement`.
    /// Returns tree-sitter's edit representation along with updated source code.
    /// Note: This method does not update `self`.
    pub fn get_edit(&self, replace_range: Range, replacement: &str) -> (String, InputEdit) {
        let source_code = self.code.clone();
        // Create the new source code content by appropriately
        // replacing the range with the replacement string.
        let new_source_code = [
            &source_code[..replace_range.start_byte],
            replacement,
            &source_code[replace_range.end_byte..],
        ]
        .concat();

        
        // Log the edit
        let replaced_code_snippet = &source_code[replace_range.start_byte..replace_range.end_byte];
        #[rustfmt::skip]
        info!("{} at ({:?}) -\n {}", if replacement.is_empty() { "Delete code" } else {"Update code" }.green(),
            ((&replace_range.start_point.row, &replace_range.start_point.column),
                (&replace_range.end_point.row, &replace_range.end_point.column)),
            if !replacement.is_empty() {format!("{}\n to \n{}",replaced_code_snippet.italic(),replacement.italic())
            } else {format!("{} ", replaced_code_snippet.italic())}
        );

        // Inner function to find the position (Point) for a given offset.
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

        let len_new_source_code_bytes = replacement.as_bytes().len();
        let byte_vec = &source_code.as_bytes().to_vec();
        // Create the InputEdit as per the tree-sitter api documentation.
        let edit = InputEdit {
            start_byte: replace_range.start_byte,
            old_end_byte: replace_range.end_byte,
            new_end_byte: replace_range.start_byte + len_new_source_code_bytes,
            start_position: position_for_offset(byte_vec, replace_range.start_byte),
            old_end_position: position_for_offset(byte_vec, replace_range.end_byte),
            new_end_position: position_for_offset(
                byte_vec,
                replace_range.start_byte + len_new_source_code_bytes,
            ),
        };

        (new_source_code, edit)
    }

    /// Get a reference to the source code unit's code.
    #[must_use]
    pub fn code(&self) -> String {
        String::from(&self.code)
    }


    /// Get a reference to the source code unit's path.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
