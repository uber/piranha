mod config;
mod rule_graph;
#[cfg(test)]
mod test;
mod tree_sitter;
mod utilities;

// TODO: Add logic to main 
// TODO: 

pub mod piranha {
    use crate::config::{PiranhaArguments, Rule};
    use crate::rule_graph::RuleStore;
    use crate::tree_sitter::{
        self as ts_utils, PiranhaRuleMatcher, TSQuery, TagMatches, TreeSitterHelpers,
    };
    use crate::utilities::{get_files_with_extension, read_file};
    use colored::Colorize;
    use std::collections::HashMap;

    use std::path::PathBuf;
    use tree_sitter::{InputEdit, Language, Node, Parser, Range, Tree};

    pub fn get_cleanups_for_code_base_new(args: PiranhaArguments) -> HashMap<PathBuf, String> {
        let mut flag_cleaner = FlagCleaner::new(args);

        flag_cleaner.cleanup();

        flag_cleaner
            .relevant_files
            .iter()
            .map(|(k, x)| (k.clone(), x.code.clone()))
            .collect()
    }

    pub struct FlagCleaner {
        rule_store: RuleStore,
        language: Language,
        files: Vec<PathBuf>,
        relevant_files: HashMap<PathBuf, SourceCodeUnit>,
        input_substitutions: TagMatches,
    }

    impl FlagCleaner {
        pub fn cleanup(&mut self) {
            let mut parser = Parser::new();
            parser
                .set_language(self.language)
                .expect("Could not set language");

            loop {
                let rules = self.rule_store.get_seed_rules();
                println!("Number of seed rules {}", rules.len());
                let mut any_file_updated = false;
                for path in self.files.iter_mut() {
                    let content = read_file(&path);
                    let pattern = self.rule_store.get_grep_heuristics();
                    println!("Searching {:?} in {:?}", pattern, path);
                    if pattern.is_match(&content) {
                        println!("Found!");
                        let scu = self
                            .relevant_files
                            .entry(path.to_path_buf())
                            .or_insert_with(|| {
                                return SourceCodeUnit::new(
                                    &mut parser,
                                    content,
                                    &self.input_substitutions,
                                );
                            });
                        any_file_updated |=
                            scu.apply_rules(&mut self.rule_store, rules.clone(), &mut parser, None);
                    }
                }

                if !any_file_updated {
                    break;
                }
            }
        }

        pub fn new(args: PiranhaArguments) -> Self {
            let language = args.language.get_language();
            let extension = args.language.get_extension();
            let graph_rule_store = RuleStore::new(&args);

            let mut parser = Parser::new();
            parser
                .set_language(language)
                .expect("Could not set language");

            let relevant_files = get_files_with_extension(&args.path_to_code_base, extension);

            let files = relevant_files
                .iter()
                .map(|dir_entry| dir_entry.path())
                .collect();
            Self {
                rule_store: graph_rule_store,
                language,
                files,
                relevant_files: HashMap::new(),
                input_substitutions: args.input_substitutions,
            }
        }
    }

    #[derive(Clone)]
    pub struct SourceCodeUnit {
        pub ast: Tree,
        pub code: String,
        pub substitutions: TagMatches,
    }

    impl SourceCodeUnit {
        // This method performs the input code replacement in the source code
        fn apply_edit(
            &mut self,
            replace_range: Range,
            replacement_str: String,
            parser: &mut Parser,
        ) -> InputEdit {
            let (new_source_code, edit) =
                ts_utils::get_edit(self.code.as_str(), replace_range, &replacement_str);
            self.ast.edit(&edit);
            let new_tree = parser
                .parse(&new_source_code, Some(&self.ast))
                .expect("Could not generate new tree!");
            self.ast = new_tree;
            self.code = new_source_code;
            return edit;
        }

        // Will update all occurences of a rule in the code.
        // We will do this without sync for now. Keep things simple.
        fn apply_rule(
            &mut self,
            rule: Rule,
            rules_store: &mut RuleStore,
            parser: &mut Parser,
            scope_query: &Option<TSQuery>,
        ) -> bool {
            let mut is_rule_applied = false;
            loop {
                if self.apply_rule_to_first_match(rule.clone(), rules_store, parser, scope_query) {
                    is_rule_applied = true;
                } else {
                    break;
                }
            }
            is_rule_applied
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
            if let Some(scope_q) = scope_query {
                if let Some((range, _)) = &self.ast.root_node().get_first_match_for_query(
                    &self.code,
                    rules_store.get_query(scope_q),
                    true,
                ) {
                    root = self.get_descendant(range.start_byte, range.end_byte);
                }
            }

            let mut any_match = false;
            if let Some((range, rpl, captures_by_tag)) =
                self.get_any_match_for_rule(&rule, rules_store, root, true)
            {
                any_match = true;
                let edit = self.apply_edit(range, rpl, parser);
                self.substitutions.extend(captures_by_tag);

                let mut previous_edit = edit.clone();
                let mut curr_rule = rule.clone();
                let mut new_rules_q = vec![];

                // recurssively perform the parent edits, while queueing the Method and Class level edits.
                loop {
                    let next_rules = rules_store.get_next(curr_rule.clone(), &self.substitutions);

                    // Add Method and Class scoped rules to the
                    for (scope_s, rules) in &next_rules {
                        if ["Method", "Class"].contains(&scope_s.as_str()) && !rules.is_empty() {
                            let scope_query_q =
                                self.get_scope_query(scope_s, previous_edit, rules_store);
                            for rule in rules {
                                new_rules_q
                                    .push((scope_query_q.clone(), rule.instantiate(&self.substitutions)));
                            }
                        }
                    }

                    for r in &next_rules["Global"] {
                        rules_store.add_seed_rule(r, &self.substitutions);
                    }

                    // Process the parent
                    if let Some((c_range, replacement_str, matched_rule, new_capture_by_tag)) =
                        self.match_rules_to_context(previous_edit, rules_store, &next_rules["Parent"])
                    {
                        println!("{}", format!("Matched parent for cleanup").green());
                        previous_edit = self.apply_edit(c_range, replacement_str, parser);
                        curr_rule = matched_rule;
                        self.substitutions.extend(new_capture_by_tag);
                    } else {
                        // No more parents found for cleanup
                        break;
                    }
                }
                // Process the method and class level rules.
                // Apply recurssively
                new_rules_q.reverse();
                for (sq, rle) in new_rules_q {
                    self.apply_rule(rle, rules_store, parser, &Some(sq));
                }
            }
            return any_match;
        }

        fn apply_rules(
            &mut self,
            rules_store: &mut RuleStore,
            rules: Vec<Rule>,
            parser: &mut Parser,
            scope_query: Option<TSQuery>,
        ) -> bool {
            let mut is_any_rule_applied = false;
            for rule in rules {
                is_any_rule_applied |=
                    self.apply_rule(rule.clone(), rules_store, parser, &scope_query)
            }
            return is_any_rule_applied;
        }

        fn get_descendant(&self, start_byte: usize, end_byte: usize) -> Node {
            self.ast
                .root_node()
                .descendant_for_byte_range(start_byte, end_byte)
                .unwrap()
        }

        fn get_scope_query(
            &self,
            s_scope: &str,
            previous_edit: InputEdit,
            rules_store: &mut RuleStore,
        ) -> TSQuery {
            let mut changed_node =
                self.get_descendant(previous_edit.start_byte, previous_edit.new_end_byte);
            let mut scope_matchers = vec![];
            for s in rules_store.scopes.iter() {
                if s.name.eq(s_scope) {
                    scope_matchers = s.rules.clone();
                    break;
                }
            }
            if scope_matchers.is_empty() {
                panic!("Could not find scope matcher for {:?}", s_scope);
            }
            while let Some(parent) = changed_node.parent() {
                for m in &scope_matchers {
                    if let Some((_, captures_by_tag)) = parent.get_first_match_for_query(
                        &self.code,
                        rules_store.get_query(&m.get_matcher()),
                        false,
                    ) {
                        let transformed_query =
                            m.get_matcher_gen().substitute_tags(&captures_by_tag);
                        let _ = rules_store.get_query(&transformed_query);
                        return transformed_query;
                    } else {
                        changed_node = parent;
                    }
                }
            }
            panic!("Could not create scope query for {:?}", s_scope);
        }

        fn match_rules_to_context(
            &mut self,
            previous_edit: InputEdit,
            rules_store: &mut RuleStore,
            rules: &Vec<Rule>,
        ) -> Option<(Range, String, Rule, TagMatches)> {
            if rules.is_empty(){
                return None;
            }
            let context = self.get_context(previous_edit);
            for rule in rules {
                for ancestor in &context {
                    let cr = rule.clone();
                    if let Some((range, replacement, captures_by_tag)) =
                        self.get_any_match_for_rule(&cr, rules_store, ancestor.clone(), false)
                    {
                        return Some((range, replacement, cr, captures_by_tag));
                    }
                }
            }
            return None;
        }

        fn get_context(&self, previous_edit: InputEdit) -> Vec<Node> {
            let changed_node =
                self.get_descendant(previous_edit.start_byte, previous_edit.new_end_byte);
            let mut context = vec![changed_node];
            let parent = changed_node.parent().clone();
            if let Some(pu) = parent {
                context.push(pu);
                if let Some(grand_parent) = pu.parent() {
                    context.push(grand_parent);
                }
            }
            context
        }

        fn new(parser: &mut Parser, code: String, substitutions: &TagMatches) -> Self {
            let ast = parser.parse(&code, None).expect("Could not parse code");
            Self {
                ast,
                code,
                substitutions: substitutions.clone(),
            }
        }

        fn get_any_match_for_rule(
            &self,
            rule: &Rule,
            rule_store: &mut RuleStore,
            node: Node,
            recurssive: bool,
        ) -> Option<(Range, String, TagMatches)> {
            let all_relevant_query_matches = node.match_query(
                String::from(&self.code),
                rule_store.get_query(&rule.get_query()),
                recurssive,
            );

            for (range, tag_substitutions) in all_relevant_query_matches {
                let n = self.get_descendant(range.start_byte, range.end_byte);
                if self.satisfies_constraint(n, &rule, &tag_substitutions, rule_store) {
                    let replacement = rule.replace.substitute_tags(&tag_substitutions);
                    return Some((range.clone(), replacement, tag_substitutions));
                }
            }
            None
        }

        fn satisfies_constraint(
            &self,
            node: Node,
            rule: &Rule,
            capture_by_tags: &TagMatches,
            rule_store: &mut RuleStore,
        ) -> bool {
            if let Some(constraint) = &rule.constraint {
                let mut curr_node = node;
                while let Some(parent) = curr_node.parent() {
                    if let Some((range, _)) = parent.get_first_match_for_query(
                        &self.code,
                        &constraint.matcher.create_query(rule_store.language),
                        false,
                    ) {
                        let matcher = self.get_descendant(range.start_byte, range.end_byte);
                        let mut all_queries_match = true;
                        for q in &constraint.queries {
                            let query_str = q.substitute_tags(&capture_by_tags);
                            let query = &rule_store.get_query(&query_str);
                            all_queries_match = all_queries_match
                                && matcher
                                    .get_first_match_for_query(&self.code, query, true)
                                    .is_some();
                        }
                        return (all_queries_match && constraint.predicate.is_all())
                            || (!all_queries_match && constraint.predicate.is_none());
                    }
                    curr_node = parent;
                }
            }
            true
        }
    }
}
