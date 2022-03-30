mod config;
mod rule_graph;
#[cfg(test)]
mod test;
mod tree_sitter;
mod utilities;

pub mod piranha {

    use crate::config::Constraint;
    use crate::config::{map_key, Rule};
    use colored::Colorize;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tree_sitter::Language;
    use tree_sitter::Node;
    use tree_sitter::Parser;
    use tree_sitter::QueryCursor;
    use tree_sitter::Range;
    use tree_sitter::Tree;
    use tree_sitter::{InputEdit, Query};

    use crate::tree_sitter as ts_utils;
    use crate::tree_sitter::group_by_tag_str;
    use crate::utilities::get_extension;
    use crate::utilities::get_files_with_extension;
    use crate::utilities::read_file;
    use crate::utilities::substitute_in_str;

    use crate::rule_graph::{EdgeWeight, GraphRuleStore};

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

    pub struct FlagCleaner {
        // rules_store: RulesStore,
        graph_rule_store: GraphRuleStore,
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
                let rules = self.graph_rule_store.get_seed_rules();
                let mut any_file_updated = false;
                for (_, scu) in self.files.iter_mut() {
                    if scu.apply_rules(&mut self.graph_rule_store, rules.clone(), &mut parser, None)
                    {
                        any_file_updated = true;
                    }
                }
                if !any_file_updated {
                    break;
                }
            }
        }

        pub fn new(
            path_to_code_base: &str,
            input_language: &str,
            flag_name: &str,
            flag_namespace: &str,
            flag_value: &str,
        ) -> Self {
            let language = ts_utils::get_language(input_language);
            let extension = get_extension(input_language);

            let graph_rule_store = GraphRuleStore::new(
                input_language,
                language,
                flag_name,
                flag_namespace,
                flag_value,
            );

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
                    SourceCodeUnit::new(
                        &mut parser,
                        code,
                        graph_rule_store.seed_substitutions.clone(),
                    ),
                );
            }
            Self {
                graph_rule_store,
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
            rules_store: &mut GraphRuleStore,
            parser: &mut Parser,
            scope_query: &Option<String>,
        ) -> bool {
            let mut any_match = true;

            while any_match {
                any_match = any_match
                    && self.apply_rule_to_any_match(rule.clone(), rules_store, parser, scope_query);
            }

            any_match
        }

        fn apply_rule_to_any_match(
            &mut self,
            rule: Rule,
            rules_store: &mut GraphRuleStore,
            parser: &mut Parser,
            scope_query: &Option<String>,
        ) -> bool {

            // Get scope node
            let mut root = self.ast.root_node();
            if let Some(scope_q) = scope_query {
                if let Some((range, _)) = get_any_match_for_rule_no_constraint(
                    self.ast.root_node(),
                    rules_store.get_query(scope_q),
                    self.code.as_bytes(),
                    true,
                ){
                root = self.get_descendant(range.start_byte, range.end_byte);
                }
            }


            let mut any_match = false;
            if let Some((range, rpl, captures_by_tag)) =
                self.scan_and_match_rule(root, self.code.as_bytes(), rule.clone(), rules_store)
            {
                any_match = true;
                let edit = self.apply_edit(range, rpl, parser);
                self.substitutions.extend(captures_by_tag);
                
                let mut previous_edit = edit.clone();
                let mut curr_rule = rule.clone();
                let mut scope_rule_queue = vec![];

                loop {

                    let (parent_rules, file_level_rules, _global_rules) = rules_store
                        .get_next(curr_rule.clone(), &self.substitutions);

                    for (ew, r) in file_level_rules {
                        if let Some(scope_query) = self.get_scope_query(ew.scope, previous_edit, rules_store) {
                            scope_rule_queue.push((scope_query, r));
                        }
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

                scope_rule_queue.reverse();
                for (sq, rle) in scope_rule_queue {
                    self.apply_rule(rle, rules_store, parser, &Some(sq));
                }
            }
            return any_match;
        }

        fn apply_rules(
            &mut self,
            rules_store: &mut GraphRuleStore,
            rules: Vec<Rule>,
            parser: &mut Parser,
            scope_query: Option<String>,
        ) -> bool {
            let mut any_matches = false;
            for rule in rules {
                if self.apply_rule(rule.clone(), rules_store, parser, &scope_query) {
                    any_matches = true;
                }
            }
            return any_matches;
        }

        fn get_descendant(&self, start_byte: usize, end_byte: usize) -> Node {
            self.ast
                .root_node()
                .descendant_for_byte_range(start_byte, end_byte)
                .unwrap()
        }

        fn get_scope_query(
            &self,
            s_scope: String,
            previous_edit: InputEdit,
            rules_store: &mut GraphRuleStore,
        ) -> Option<String> {
            // if let Some(s_scope) = o_scope {
            let mut changed_node =
                self.get_descendant(previous_edit.start_byte, previous_edit.new_end_byte);
            let mut scope_matchers = vec![];
            for s in rules_store.scopes.iter() {
                if s.name.eq(s_scope.as_str()) {
                    scope_matchers = s.rules.clone();
                    break;
                }
            }
            if scope_matchers.is_empty() {
                return None;
            }
            while let Some(parent) = changed_node.parent() {
                for m in &scope_matchers {
                    if let Some((_, captures_by_tag)) = get_any_match_for_rule_no_constraint(
                        parent,
                        rules_store.get_query(&m.matcher),
                        self.code.as_bytes(),
                        false,
                    ) {
                        let transformed_query =
                            substitute_in_str(&captures_by_tag, &m.matcher_gen, &map_key);
                        let _ = rules_store.get_query(&transformed_query);
                        return Some(transformed_query);
                    } else {
                        changed_node = parent;
                    }
                }
            }
            None
        }

        fn match_rules_to_context(
            &mut self,
            previous_edit: InputEdit,
            rules_store: &mut GraphRuleStore,
            rules: &Vec<(EdgeWeight, Rule)>,
        ) -> Option<(Range, String, Rule, HashMap<String, String>)> {
            let context = self.get_context(previous_edit);

            for rule in rules {
                for ancestor in &context {
                    let cr = rule.1.clone();
                    if let Some((range, replacement, captures_by_tag)) = self
                        .get_any_match_for_rule(
                            cr.clone(),
                            rules_store,
                            ancestor.clone(),
                            self.code.as_bytes(),
                            false,
                        )
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
            source_code_bytes: &[u8],
            rule: Rule,
            rule_store: &mut GraphRuleStore,
        ) -> Option<(Range, String, HashMap<String, String>)> {
            if let Some((rng, rpl, captures_by_tag)) =
                self.get_any_match_for_rule(rule, rule_store, root, source_code_bytes, true)
            {
                return Some((rng, rpl, captures_by_tag));
            }
            None
        }

        fn get_any_match_for_rule(
            &self,
            rule: Rule,
            rule_store: &mut GraphRuleStore,
            node: Node,
            source_code_bytes: &[u8],
            recurssive: bool,
        ) -> Option<(Range, String, HashMap<String, String>)> {
            // let query_str = rule.query.as_str();
            let all_relevant_query_matches = _get_all_relevant_matches_no_constraint(
                node,
                rule_store.get_query(&rule.query),
                source_code_bytes,
                recurssive,
            );

            if all_relevant_query_matches.is_empty() {
                // println!("Found no match");
                return None;
            }

            //TODO: Add logic for ancestor predicate
            for (range, captures_by_tag) in all_relevant_query_matches {
                // let n = node
                //     .descendant_for_byte_range(range.start_byte, range.end_byte)
                //     .unwrap();
                // if self.satisfies_constraint(
                //     n,
                //     rule_store.language,
                //     rule.constraint.clone(),
                //     source_code_bytes,
                //     &captures_by_tag,
                // ) {
                // println!("Satisfied constraint!");
                let replacement =
                    substitute_in_str(&captures_by_tag, &rule.replace, &map_key_as_tag);
                return Some((range.clone(), replacement, captures_by_tag));
                // }
            }
            None
        }

        fn satisfies_constraint(
            &self,
            node: Node,
            language: Language,
            constraint: Option<Constraint>,
            source_code_bytes: &[u8],
            capture_by_tags: &HashMap<String, String>,
        ) -> bool {
            if let Some(c) = constraint {
                let mut query_cache = HashMap::new();
                for q in &c.queries {
                    let z = substitute_in_str(capture_by_tags, q, &map_key);
                    let query = Query::new(language, &z);
                    query_cache.insert(z, query.unwrap());
                }
                let matcher_query = Query::new(language, &c.matcher).unwrap();

                // Apply matcher

                let mut curr_node = node;
                while let Some(parent) = curr_node.parent() {
                    if let Some((range, _)) = get_any_match_for_rule_no_constraint(
                        parent,
                        &matcher_query,
                        source_code_bytes,
                        false,
                    )
                    {
                        let matcher = self.get_descendant(range.start_byte, range.end_byte);
                        let mut c_node = node;
                        while let Some(c_p) = c_node.parent() {
                            let mut all_queries_match = true;
                            for (_, query) in &query_cache {
                                all_queries_match = all_queries_match
                                    && get_any_match_for_rule_no_constraint(
                                        matcher,
                                        query,
                                        source_code_bytes,
                                        true,
                                    ).is_some();
                            }
                            if all_queries_match {
                                return c.predicate.eq("All");
                            }
                            c_node = c_p;
                        }
                        return !c.predicate.eq("All");
                    }
                    curr_node = parent;
                }
            }
            true
        }
    }

    fn map_key_as_tag(s: &String) -> String {
        format!("@{}", s)
    }

    fn get_any_match_for_rule_no_constraint(
        node: Node,
        query: &Query,
        source_code_bytes: &[u8],
        recurssive: bool,
    ) -> Option<(Range, HashMap<String, String>)> {
        let ms =
            _get_all_relevant_matches_no_constraint(node, query, source_code_bytes, recurssive);
        return ms.first().map(|x| x.clone());
    }

    fn _get_all_relevant_matches_no_constraint(
        node: Node,
        query: &Query,
        source_code_bytes: &[u8],
        recurssive: bool,
    ) -> Vec<(Range, HashMap<String, String>)> {
        // TODO: extract parameter `cursor`
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
            let matched_node = captures.first().map(|x| x.node).unwrap();
            // let matched_node_range = matched_node.range();
            matched_node_query_match
                .entry(matched_node.range())
                .or_insert_with(Vec::new)
                .push(captured_tags);
        }

        let mut output = vec![];
        for (k, v) in matched_node_query_match {
            if v.len() == pattern_count {
                if recurssive
                    || (node.start_byte() == k.start_byte && node.end_byte() == k.end_byte)
                {
                    let mut captures_by_tag = HashMap::new();
                    for i in v {
                        captures_by_tag.extend(i.clone());
                    }
                    output.push((k, captures_by_tag));
                }
            }
        }
        output.sort_by(|a, b| a.0.start_byte.cmp(&b.0.start_byte));
        output.reverse();
        return output;
    }
}
