mod config;
mod rule_graph;
#[cfg(test)]
mod test;
mod tree_sitter;
mod utilities;

pub mod piranha {

    use crate::config::{PiranhaArguments, Rule};
    use crate::rule_graph::RuleStore;
    use crate::tree_sitter::{
        self as ts_utils, TreeSitterHelpers,PiranhaRuleMatcher
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
            .files
            .iter()
            .map(|(k, x)| (k.clone(), x.code.clone()))
            .collect()
    }

    pub struct FlagCleaner {
        rule_store: RuleStore,
        language: Language,
        pub files: HashMap<PathBuf, SourceCodeUnit>,
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
                for (_, scu) in self.files.iter_mut() {
                    any_file_updated |=
                        scu.apply_rules(&mut self.rule_store, rules.clone(), &mut parser, None);
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

            // let mut files = HashMap::new();
            let relevant_files = get_files_with_extension(&args.path_to_code_base, extension);

            let files = relevant_files
                .iter()
                .map(|dir_entry| (dir_entry.path(), read_file(&dir_entry.path())))
                .map(|(file_path, code)| {
                    (
                        file_path,
                        SourceCodeUnit::new(&mut parser, code, args.input_substiution.clone()),
                    )
                })
                .collect();

            Self {
                rule_store: graph_rule_store,
                language,
                files,
            }
        }
    }

    pub struct SourceCodeUnit {
        pub ast: Tree,
        pub code: String,
        pub substitutions: HashMap<String, String>,
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
            scope_query: &Option<String>,
        ) -> bool {
            let mut is_rule_applied = false;
            loop {
                if self.apply_rule_to_any_match(rule.clone(), rules_store, parser, scope_query) {
                    is_rule_applied = true;
                } else {
                    break;
                }
            }
            is_rule_applied
        }

        fn apply_rule_to_any_match(
            &mut self,
            rule: Rule,
            rules_store: &mut RuleStore,
            parser: &mut Parser,
            scope_query: &Option<String>,
        ) -> bool {
            // Get scope node
            let mut root = self.ast.root_node();
            if let Some(scope_q) = scope_query {
                if let Some((range, _)) = &self.ast.root_node().get_any_match_query_str(
                    &self.code,
                    rules_store.get_query(scope_q),
                    true,
                ) {
                    root = self.get_descendant(range.start_byte, range.end_byte);
                }
            }

            let mut any_match = false;
            if let Some((range, rpl, captures_by_tag)) =
                self.scan_and_match_rule(root, rule.clone(), rules_store)
            {
                any_match = true;
                let edit = self.apply_edit(range, rpl, parser);
                self.substitutions.extend(captures_by_tag);

                let mut previous_edit = edit.clone();
                let mut curr_rule = rule.clone();
                let mut new_rules_q = vec![];

                loop {
                    let (parent_rules, method_level_rules, class_level_rules, _global_rules) =
                        rules_store.get_next(curr_rule.clone(), &self.substitutions);

                    #[rustfmt::skip]
                    println!("Method {} Class {}",method_level_rules.len(),class_level_rules.len());

                    let mut add_to_queue = |s: &str, rules: Vec<Rule>| {
                        for rule in rules {
                            let scope_query = self.get_scope_query(s, previous_edit, rules_store);
                            new_rules_q.push((scope_query, rule.instantiate(&self.substitutions)));
                        }
                    };

                    let _ = &add_to_queue("Method", method_level_rules);
                    let _ = &add_to_queue("Class", class_level_rules);
                    for r in _global_rules {
                        rules_store.add_seed_rule(r, &self.substitutions);
                    }

                    if parent_rules.is_empty() {
                        break;
                    }

                    if let Some((c_range, replacement_str, matched_rule, new_capture_by_tag)) =
                        self.match_rules_to_context(previous_edit, rules_store, &parent_rules)
                    {
                        println!("{}", format!("Matched parent for cleanup").red());
                        previous_edit = self.apply_edit(c_range, replacement_str, parser);
                        curr_rule = matched_rule;
                        self.substitutions.extend(new_capture_by_tag);
                    } else {
                        break;
                    }
                }
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
            scope_query: Option<String>,
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
        ) -> String {
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
                    if let Some((_, captures_by_tag)) = parent.get_any_match_query_str(
                        &self.code,
                        rules_store.get_query(&m.matcher),
                        false,
                    ) {
                        let transformed_query =
                            m.matcher_gen.substitute_rule_holes(&captures_by_tag);
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
        ) -> Option<(Range, String, Rule, HashMap<String, String>)> {
            let context = self.get_context(previous_edit);

            for rule in rules {
                for ancestor in &context {
                    let cr = rule.clone();
                    if let Some((range, replacement, captures_by_tag)) = self
                        .get_any_match_for_rule(cr.clone(), rules_store, ancestor.clone(), false)
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

        fn new(parser: &mut Parser, code: String, substitutions: HashMap<String, String>) -> Self {
            let ast = parser.parse(&code, None).expect("Could not parse code");
            Self {
                ast,
                code,
                substitutions,
            }
        }

        fn scan_and_match_rule(
            &self,
            root: Node,
            rule: Rule,
            rule_store: &mut RuleStore,
        ) -> Option<(Range, String, HashMap<String, String>)> {
            if let Some((rng, rpl, captures_by_tag)) =
                self.get_any_match_for_rule(rule, rule_store, root, true)
            {
                return Some((rng, rpl, captures_by_tag));
            }
            None
        }

        fn get_any_match_for_rule(
            &self,
            rule: Rule,
            rule_store: &mut RuleStore,
            node: Node,
            recurssive: bool,
        ) -> Option<(Range, String, HashMap<String, String>)> {
            let all_relevant_query_matches =
            node.match_query_str(String::from(&self.code), rule_store.get_query(&rule.query), recurssive);

            if all_relevant_query_matches.is_empty() {
                return None;
            }

            for (range, captures_by_tag) in all_relevant_query_matches {
                let n = self.get_descendant(range.start_byte, range.end_byte);

                if self.satisfies_constraint(n, &rule, &captures_by_tag, rule_store) {
                    let replacement = rule.replace.substitute_rule_holes(&captures_by_tag);
                    return Some((range.clone(), replacement, captures_by_tag));
                }
            }
            None
        }

        fn satisfies_constraint(
            &self,
            node: Node,
            rule: &Rule,
            capture_by_tags: &HashMap<String, String>,
            rule_store: &mut RuleStore,
        ) -> bool {
            if let Some(constraint) = &rule.constraint {
                let mut curr_node = node;
                while let Some(parent) = curr_node.parent() {
                    if let Some((range, _)) = parent.get_any_match_query_str(
                        &self.code,
                        &constraint.matcher.create_query(rule_store.language),
                        false,
                    ) {
                        let matcher = self.get_descendant(range.start_byte, range.end_byte);
                        let mut c_node = node;
                        while let Some(c_p) = c_node.parent() {
                            let mut all_queries_match = true;
                            for q in &constraint.queries {
                                let query_str = q.substitute_rule_holes(&capture_by_tags);
                                let query = &rule_store.get_query(&query_str);
                                all_queries_match = all_queries_match
                                    && matcher.get_any_match_query_str(&self.code, query, true)
                                        .is_some();
                            }
                            if all_queries_match {
                                return constraint.predicate.eq("All");
                            }
                            c_node = c_p;
                        }
                        return !constraint.predicate.eq("All");
                    }
                    curr_node = parent;
                }
            }
            true
        }
    }
}
