/* 
Copyright (c) 2019 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use crate::config::{PiranhaArguments, Rule};
use crate::rule_graph::{RuleStore, CLASS, GLOBAL, METHOD, PARENT};
use crate::tree_sitter::{PiranhaRuleMatcher, TSQuery, TagMatches, TreeSitterHelpers};
use crate::utilities::read_file;
use colored::Colorize;
use itertools::Itertools;
use jwalk::WalkDir;
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf};
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Range, Tree};
//TODO: File level comments .. what it does ... what it handles so on 

//TODO: Comments for cstruct and its fields. 
pub struct FlagCleaner {
    rule_store: RuleStore,
    // FIXME: Remove
    language: Language,
    path_to_codebase: String,
    extension: String,
    pub relevant_files: HashMap<PathBuf, SourceCodeUnit>,
    input_substitutions: TagMatches,
}

impl FlagCleaner {
    // Performs cleanup related to stale flags
    pub fn cleanup(&mut self) {
        let mut parser = Parser::new();
        parser
            .set_language(self.language)
            .expect("Could not set the language for the parser.");

        loop {
            let curr_rules = self.rule_store.get_global_rules();

            println!("Number of global rules {}", curr_rules.len());

            let files_containing_pattern = self.get_relevant_files();
            
            #[rustfmt::skip]
            println!("{}", format!("Will parse and analyze {} files.", files_containing_pattern.len()).green());

            for (path, content) in files_containing_pattern {
                self.relevant_files
                    .entry(path.to_path_buf())
                    .or_insert_with(|| {
                        SourceCodeUnit::new(&mut parser, content, &self.input_substitutions, &path)
                    })
                    .apply_rules(&mut self.rule_store, &curr_rules, &mut parser, None);

                if self.rule_store.global_rules.len() > curr_rules.len() {
                    println!("Found a new global rule. Will start scanning all the files again.");
                    break;
                }
            }

            if self.rule_store.global_rules.len() == curr_rules.len() {
                break;
            }
        }
    }
    /// Gets all the files from the code base that (i) have the language appropriate file extension, and (ii) contains the grep pattern.
    /// Note that `WalkDir` traverses the directory with parallelism.
    pub fn get_relevant_files(&self) -> HashMap<PathBuf, String> {
        let pattern = self.get_grep_heuristics();
        println!(
            "{}",
            format!("Searching for pattern {}", pattern.as_str()).green()
        );
        WalkDir::new(&self.path_to_codebase)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|de| {
                de.path()
                    .extension()
                    .map(|e| e.to_str().unwrap().eq(self.extension.as_str()))
                    .unwrap_or(false)
            })
            .map(|f| (f.path().to_path_buf(), read_file(&f.path().to_path_buf())))
            .filter(|x| pattern.is_match(x.1.as_str()))
            .collect()
    }

    pub fn new(args: PiranhaArguments) -> Self {
        let language = args.language.get_language();
        let extension = args.language.get_extension();
        let graph_rule_store = RuleStore::new(&args);
        Self {
            rule_store: graph_rule_store,
            language,
            path_to_codebase: args.path_to_code_base,
            extension: extension.to_string(),
            relevant_files: HashMap::new(),
            input_substitutions: args.input_substitutions,
        }
    }

    /// To create the current set of global rules, certain substitutions were applied.
    /// This method creates a regex pattern matching these substituted values.
    ///
    /// At the directory level, we would always look to perform global rules. However this is expensive because
    /// it requires parsing each file. To overcome this, we apply this simple
    /// heuristic to find the (upper bound) files that would match one of our current global rules.
    /// This heurisitic reduces the number of files to parse.
    ///
    pub fn get_grep_heuristics(&self) -> Regex {
        let reg_x = self
            .rule_store
            .get_global_rules()
            .iter()
            .flat_map(|r| r.grep_heuristics.as_ref().unwrap().iter())
            //Remove duplicates
            .sorted()
            .dedup()
            //FIXME: Dirty trick to remove tru and false. Ideally, grep heuristic could be a field in itself for a rule.
            // Since not all "holes" could be used as grep heuristic. 
            .filter(|x|!x.as_str().eq("true") && !x.as_str().eq("false"))
            .join("|");
        Regex::new(reg_x.as_str()).unwrap()
    }
}

#[derive(Clone)]
pub struct SourceCodeUnit {
    // The tree representing the file
    pub ast: Tree,
    // The content of a file
    pub code: String,
    // The tag substitution cache.
    pub substitutions: TagMatches,
    // The path to the source code.
    pub path: PathBuf,
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
        let (new_source_code, edit) = self.get_edit(replace_range, &replacement_str);
        self.ast.edit(&edit);
        let new_tree = parser
            .parse(&new_source_code, Some(&self.ast))
            .expect("Could not generate new tree!");
        self.ast = new_tree;
        self.code = new_source_code;
        return edit;
    }

    /// Will apply the `rule` to all of its occurences in the source code unit.
    fn apply_rule(
        &mut self,
        rule: Rule,
        rules_store: &mut RuleStore,
        parser: &mut Parser,
        scope_query: &Option<TSQuery>,
    ) {
        loop {
            if !self.apply_rule_to_first_match(rule.clone(), rules_store, parser, scope_query) {
                break;
            }
        }
    }

    fn apply_rule_to_first_match(
        &mut self,
        rule: Rule,
        rules_store: &mut RuleStore,
        parser: &mut Parser,
        scope_query: &Option<TSQuery>,
    ) -> bool {
        // Get scope node
        let mut root = self.ast.root_node();
        if let Some(ts_scope_query) = scope_query {
            if let Some((range, _)) = &self.ast.root_node().get_match_for_query(
                &self.code,
                rules_store.get_query(ts_scope_query),
                true,
            ) {
                root = self.get_descendant(range.start_byte, range.end_byte);
            }
        }

        let mut any_match = false;

        // Recurssively scan the node, and match the rule.
        if let Some((range, rpl, captures_by_tag)) =
            self.get_any_match_for_rule(&rule, rules_store, root, true)
        {
            any_match = true;
            let edit = self.apply_edit(range, rpl, parser);
            self.substitutions.extend(captures_by_tag);

            let mut curr_edit = edit.clone();
            let mut curr_rule = rule.clone();
            let mut file_level_next_rules = vec![];

            // perform the parent edits, while queueing the Method and Class level edits.
            loop {
                let next_rules = rules_store.get_next(&curr_rule, &self.substitutions);
                println!("Next rules {}", format!("{:?}", next_rules.values().flatten().map(|x|x.name.to_string()).collect_vec()).bright_purple());

                // Add Method and Class scoped rules to the
                for (scope_s, rules) in &next_rules {
                    if [METHOD, CLASS].contains(&scope_s.as_str()) && !rules.is_empty() {
                        for rule in rules {
                            file_level_next_rules.push((
                                self.get_scope_query(scope_s, curr_edit, rules_store),
                                rule.instantiate(&self.substitutions),
                            ));
                        }
                    }
                }
                // Add Global rules as seed rules
                for r in &next_rules[GLOBAL] {
                    rules_store.add_global_rule(r, &self.substitutions);
                }

                // Process the parent
                if let Some((c_range, replacement_str, matched_rule, new_capture_by_tag)) =
                    self.match_rules_to_context(curr_edit, rules_store, &next_rules[PARENT])
                {
                    println!("{}", format!("Matched parent for cleanup").green());
                    curr_edit = self.apply_edit(c_range, replacement_str, parser);
                    curr_rule = matched_rule;
                    self.substitutions.extend(new_capture_by_tag);
                } else {
                    // No more parents found for cleanup
                    break;
                }
            }
            // Apply the method and class level rules
            file_level_next_rules.reverse();
            for (sq, rle) in file_level_next_rules {
                self.apply_rule(rle, rules_store, parser, &Some(sq));
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
        scope_query: Option<TSQuery>,
    ) {
        for rule in rules.clone() {
            self.apply_rule(rule.clone(), rules_store, parser, &scope_query)
        }
    }

    /// Get the smallest node within `self` that spans the given range.
    fn get_descendant(&self, start_byte: usize, end_byte: usize) -> Node {
        self.ast
            .root_node()
            .descendant_for_byte_range(start_byte, end_byte)
            .unwrap()
    }

    /// Generate a tree-sitter based query representing the scope of the previous edit.
    /// Currently we generate scopes based on the configs provided in `<lang>_scopes.toml`.
    fn get_scope_query(
        &self,
        scope_level: &str,
        previous_edit: InputEdit,
        rules_store: &mut RuleStore,
    ) -> TSQuery {
        let mut changed_node =
            self.get_descendant(previous_edit.start_byte, previous_edit.new_end_byte);

        // Get the scope matchers for `s_scope`
        let scope_matchers = rules_store
            .scopes
            .iter()
            .find(|level| level.name.eq(scope_level))
            .map(|scope| scope.rules.clone())
            .unwrap_or_else(Vec::new);

        // Match the `scope_matcher.matcher` to the parent
        while let Some(parent) = changed_node.parent() {
            for m in &scope_matchers {
                if let Some((_, captures_by_tag)) =
                    parent.get_match_for_query(&self.code, rules_store.get_query(&m.matcher), false)
                {
                    // Generate the scope query for the specific context
                    return m.generator.substitute_tags(&captures_by_tag);
                } else {
                    changed_node = parent;
                }
            }
        }
        panic!("Could not create scope query for {:?}", scope_level);
    }

    // Apply all the `rules` to the node, it's parent and its grand parent.
    // Short-circuit on the first match.
    fn match_rules_to_context(
        &mut self,
        previous_edit: InputEdit,
        rules_store: &mut RuleStore,
        rules: &Vec<Rule>,
    ) -> Option<(Range, String, Rule, TagMatches)> {
        if rules.is_empty() {
            return None;
        }
        let context = self.get_context(previous_edit);
        for rule in rules {
            for ancestor in &context {
                if let Some((range, replacement, captures_by_tag)) =
                    self.get_any_match_for_rule(&rule, rules_store, ancestor.clone(), false)
                {
                    return Some((range, replacement, rule.clone(), captures_by_tag));
                }
            }
        }
        return None;
    }

    /// Returns the node, its parent and its grand parent.
    fn get_context(&self, previous_edit: InputEdit) -> Vec<Node> {
        let changed_node =
            self.get_descendant(previous_edit.start_byte, previous_edit.new_end_byte);
        let mut context = vec![changed_node];
        if let Some(parent) = changed_node.parent() {
            context.push(parent);
            if let Some(grand_parent) = parent.parent() {
                context.push(grand_parent);
            }
        }
        context
    }

    fn new(parser: &mut Parser, code: String, substitutions: &TagMatches, path: &PathBuf) -> Self {
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
        recurssive: bool,
    ) -> Option<(Range, String, TagMatches)> {
        // Get all matches for the query
        let all_query_matches = node.get_all_matches_for_query(
            self.code.clone(),
            rule_store.get_query(&rule.get_query()),
            recurssive,
            Some(rule.replace_node.clone()),
        );

        // Return the first match that satisfies constraint of the rule
        for (range, tag_substitutions) in all_query_matches {
            let n = self.get_descendant(range.start_byte, range.end_byte);
            if self.satisfies_constraint(n, &rule, &tag_substitutions, rule_store) {
                let replacement = rule.replace.substitute_tags(&tag_substitutions);
                return Some((range.clone(), replacement, tag_substitutions));
            }
        }
        None
    }

    /// Checks if the node satisfies the constraints
    fn satisfies_constraint(
        &self,
        node: Node,
        rule: &Rule,
        capture_by_tags: &TagMatches,
        rule_store: &mut RuleStore,
    ) -> bool {
        if let Some(constraint) = &rule.constraint {
            let mut curr_node = node;
            // Get the scope of the predicate
            while let Some(parent) = curr_node.parent() {
                if let Some((range, _)) = parent.get_match_for_query(
                    &self.code,
                    &constraint.matcher.create_query(rule_store.language),
                    false,
                ) {
                    let scope = self.get_descendant(range.start_byte, range.end_byte);
                    // Apply the predicate in the scope
                    let mut all_queries_match = true;
                    for q in &constraint.queries {
                        let query_str = q.substitute_tags(&capture_by_tags);
                        let query = &rule_store.get_query(&query_str);
                        all_queries_match = all_queries_match
                            && scope.get_match_for_query(&self.code, query, true).is_some();
                    }
                    return (all_queries_match && constraint.predicate.is_all())
                        || (!all_queries_match && constraint.predicate.is_none());
                }
                curr_node = parent;
            }
        }
        true
    }

    /// Replaces the given byte range (`replace_range`) with the `replacement`.
    /// Returns tree-sitter's edit representation along with updated source code.
    /// Note: This method does not update `self`.
    pub fn get_edit(&self, replace_range: Range, replacement: &str) -> (String, InputEdit) {
        let source_code = self.code.clone();
        let new_source_code = [
            &source_code[..replace_range.start_byte],
            replacement,
            &source_code[replace_range.end_byte..],
        ]
        .concat();

        let replace_code = &source_code[replace_range.start_byte..replace_range.end_byte];

        #[rustfmt::skip]
        println!("{} at ({:?}) -\n {}", if replacement.is_empty() { "Delete code" } else {"Update code" }.green(),
            ((&replace_range.start_point.row, &replace_range.start_point.column),
                (&replace_range.end_point.row, &replace_range.end_point.column)),
            if !replacement.is_empty() {format!("{}\n to \n{}",replace_code.italic(),replacement.italic())
            } else {format!("{} ", replace_code.italic())}
        );

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
}
