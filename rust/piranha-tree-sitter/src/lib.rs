mod config;
#[cfg(test)]
mod test;
mod tree_sitter;
mod utilities;

pub mod piranha {

    use crate::config::{Config, Rule};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tree_sitter::InputEdit;
    use tree_sitter::Language;
    use tree_sitter::Node;
    use tree_sitter::Parser;
    use tree_sitter::Query;
    use tree_sitter::QueryCursor;
    use tree_sitter::Range;
    use tree_sitter::Tree;

    use crate::tree_sitter as ts_utils;
    use crate::tree_sitter::group_by_tag_str;
    use crate::utilities::get_extension;
    use crate::utilities::get_files_with_extension;
    use crate::utilities::read_file;
    use crate::utilities::substitute_in_str;

    // TODO: Add a string level entry point
    // TODO: Verify configs (Make sure no same named tags in "and queries")
    // FIXME: Change s-expression based equality to tree based
    // TODO: Add Inline variable cleanup (Basically add Method and File based and then rules)

    pub fn get_cleanups_for_code_base_new(
        path_to_code_base: &str,
        input_language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> HashMap<PathBuf, String> {
        let mut flag_cleaner = FlagCleaner::new(
            path_to_code_base,
            input_language,
            flag_name,
            flag_namespace,
            flag_value,
        );
        flag_cleaner.cleanup();
        flag_cleaner
            .files
            .iter()
            .map(|(k, x)| (k.clone(), x.code.clone()))
            .collect()
    }

    struct RulesStore {
        pub rule_query_cache: HashMap<String, Query>,
        pub seed_rules: Vec<Rule>,
        pub cleanup_rules: Vec<Rule>,
    }

    pub struct FlagCleaner {
        rules_store: RulesStore,
        language: Language,
        pub files: HashMap<PathBuf, SourceCodeUnit>,
    }

    impl FlagCleaner {
        pub fn new(
            path_to_code_base: &str,
            input_language: &str,
            flag_name: &str,
            flag_namespace: &str,
            flag_value: &str,
        ) -> Self {
            let language = ts_utils::get_language(input_language);
            let extension = get_extension(input_language);
            let (ff_config, cleanup_config) =
                Config::read_config(input_language, flag_name, flag_namespace, flag_value);
            let mut rule_query_cache = HashMap::new();

            let seed_rules = ff_config.rules;
            let cleanup_rules = cleanup_config.rules;

            for r in &seed_rules {
                rule_query_cache.insert(String::from(r.query.as_str()), r.get_query(language));
            }
            for r in &cleanup_rules {
                rule_query_cache.insert(String::from(r.query.as_str()), r.get_query(language));
            }

            let rules_store = RulesStore {
                rule_query_cache,
                seed_rules,
                cleanup_rules,
            };

            let mut files = HashMap::new();
            let relevant_files = get_files_with_extension(path_to_code_base, extension);

            let mut parser = Parser::new();
            parser
                .set_language(language)
                .expect("Could not set language");

            for dir_entry in relevant_files {
                let file_path = dir_entry.path();
                let code = read_file(&file_path);
                files.insert(
                    file_path,
                    SourceCodeUnit::parse(&mut parser, code, language),
                );
            }

            Self {
                rules_store,
                language,
                files,
            }
        }

        pub fn cleanup(&mut self) {
            let mut parser = Parser::new();
            parser
                .set_language(self.language)
                .expect("Could not set language");
            loop {
                let mut any_file_updated = false;
                for (_, scu) in self.files.iter_mut() {
                    if scu.apply_seed_rules(&mut self.rules_store, &mut parser) {
                        any_file_updated = true;
                    }
                }
                if !any_file_updated {
                    break;
                }
            }
        }
    }

    pub struct SourceCodeUnit {
        pub ast: Tree,
        pub code: String,
        language: Language,
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
            rules_store: &mut RulesStore,
            parser: &mut Parser,
        ) -> bool {
            let mut any_match = false;
            loop {
                //TODO: Will return andThen rules too.
                let replacement = self.scan_and_match_rule(rule.clone(), rules_store);
                if replacement.is_none() {
                    return any_match;
                } else {
                    any_match = true;
                    let (range, rpl) = replacement.unwrap();
                    let edit = self.apply_edit(range, rpl, parser);
                    self.apply_cleanup_rules(edit, rules_store, parser);
                }
            }
        }

        fn apply_seed_rules(&mut self, rules_store: &mut RulesStore, parser: &mut Parser) -> bool {
            let rules = rules_store.seed_rules.clone();
            self.apply_rules(rules_store, rules, parser)
        }

        fn apply_rules(
            &mut self,
            rules_store: &mut RulesStore,
            rules: Vec<Rule>,
            parser: &mut Parser,
        ) -> bool {
            let mut any_matches = false;
            for rule in rules {
                if self.apply_rule(rule.clone(), rules_store, parser) {
                    any_matches = true;
                }
            }
            return any_matches;
        }

        fn apply_cleanup_rules(
            &mut self,
            edit: InputEdit,
            rules_store: &mut RulesStore,
            parser: &mut Parser,
        ) {
            let mut previous_edit = edit.clone();
            loop {
                if let Some((range, replacement_str, new_rules)) =
                    self.match_cleanup_site(previous_edit, rules_store)
                {
                    previous_edit = self.apply_edit(range, replacement_str, parser);
                    if !new_rules.is_empty() {
                        self.apply_rules(rules_store, new_rules, parser);
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        fn match_cleanup_site(
            &mut self,
            previous_edit: InputEdit,
            rules_store: &mut RulesStore,
        ) -> Option<(Range, String, Vec<Rule>)> {
            let context = self.get_context(previous_edit);

            let cleanup_rules = rules_store.cleanup_rules.clone();

            for rule in &cleanup_rules {
                for ancestor in &context {
                    if let Some((range, replacement, new_rules)) = self.get_any_match_for_rule(
                        rule.clone(),
                        rules_store,
                        ancestor.clone(),
                        self.code.as_bytes(),
                        false,
                    ) {
                        return Some((range, replacement, new_rules));
                    }
                }
            }
            return None;
        }

        fn get_context(&self, previous_edit: InputEdit) -> Vec<Node> {
            let changed_node = self
                .ast
                .root_node()
                .descendant_for_byte_range(previous_edit.start_byte, previous_edit.new_end_byte)
                .unwrap();
            let mut context = vec![changed_node];
            let parent = changed_node.parent().clone();
            if parent.is_some() {
                let pu = parent.unwrap();
                context.push(pu);
                let grand_parent = pu.parent().clone();
                if grand_parent.is_some() {
                    context.push(grand_parent.unwrap());
                }
            }
            context
        }

        fn parse(parser: &mut Parser, code: String, language: Language) -> Self {
            let ast = parser.parse(&code, None).expect("Could not parse code");
            Self {
                ast,
                code,
                language,
            }
        }

        fn scan_and_match_rule(
            &self,
            rule: Rule,
            rule_store: &mut RulesStore, // rule_query_cache: &mut HashMap<String, Query>,
        ) -> Option<(Range, String)> {
            let root = self.ast.root_node();
            let source_code_bytes = self.code.as_bytes();
            if let Some((rng, rpl, _new_rules)) = self.get_any_match_for_rule(rule, rule_store, root, source_code_bytes, true){
                return Some((rng, rpl));
            }
            None
        }

        fn get_any_match_for_rule(
            &self,
            rule: Rule,
            rule_store: &mut RulesStore,
            node: Node,
            source_code_bytes: &[u8],
            recurssive: bool,
        ) -> Option<(Range, String, Vec<Rule>)> {
            let query_str = rule.query.as_str();
            if !rule_store.rule_query_cache.contains_key(query_str) {
                panic!("{}", query_str);
            }

            let all_relevant_query_matches = get_all_relevant_matches(
                node,
                query_str,
                source_code_bytes,
                &rule_store,
                recurssive,
            );

            if all_relevant_query_matches.is_empty() {
                return None;
            }

            //TODO: Add logic for ancestor predicate

            let relevant_match = all_relevant_query_matches.first().unwrap().clone();

            let mut captures_by_tag = HashMap::new();
            for i in relevant_match.1 {
                captures_by_tag.extend(i.clone());
            }

            let replacement = substitute_in_str(&captures_by_tag, &rule.replace, &map_key_as_tag);

            let and_then_rules = rule.and_then(captures_by_tag, self.language);
            let mut new_rules = vec![];

            for (r, q) in and_then_rules {
                rule_store
                    .rule_query_cache
                    .insert(String::from(&r.query), q);
                rule_store.seed_rules.push(r.clone());
                new_rules.push(r);
            }

            return Some((relevant_match.0.clone(), replacement, new_rules));
        }
    }

    fn map_key_as_tag(s: &String) -> String {
        format!("@{}", s)
    }

    fn get_all_relevant_matches(
        node: Node,
        query_str: &str,
        source_code_bytes: &[u8],
        rule_store: &RulesStore,
        recurssive: bool,
    ) -> Vec<(Range, Vec<HashMap<String, String>>)> {
        // TODO: extract parameter `cursor`
        let query = rule_store.rule_query_cache.get(query_str).unwrap();
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, node, source_code_bytes);
        let pattern_count = query.pattern_count();

        let mut matched_node_query_match = HashMap::new();

        for qm in query_matches {
            let captures = qm.captures;
            if captures.is_empty() {
                break;
            }

            let mut captured_tags = group_by_tag_str(captures, query, source_code_bytes);
            for cn in query.capture_names() {
                captured_tags
                    .entry(String::from(cn))
                    .or_insert_with(String::new);
            }

            // The first capture is the outermost tag
            let matched_node_range = captures.first().map(|z| z.node.range()).unwrap();
            matched_node_query_match
                .entry(matched_node_range)
                .or_insert_with(Vec::new)
                .push(captured_tags);
        }

        let mut output = vec![];
        for (k, v) in matched_node_query_match {
            if v.len() == pattern_count
                && (recurssive
                    || k.start_byte == node.start_byte() && k.end_byte == node.end_byte())
            {
                output.push((k, v));
            }
        }
        return output;
    }
}
