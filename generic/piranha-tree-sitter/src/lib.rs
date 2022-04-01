mod config;
mod rule_graph;
#[cfg(test)]
mod test;
mod tree_sitter;
mod utilities;

pub mod piranha {

    use crate::config::Constraint;
    use crate::config::PiranhaArguments;
    use crate::config::Rule;
    use crate::tree_sitter::node_matches_range;
    use colored::Colorize;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tree_sitter::{Tree, Language, Node, Parser, QueryCursor, Range, InputEdit, Query};
    use crate::tree_sitter::group_captures_by_tag;
    use crate::tree_sitter::{self as ts_utils, TreeSitterQuery};
    use crate::utilities::get_files_with_extension;
    use crate::utilities::read_file;
    use crate::utilities::{MapOfVec};

    use crate::rule_graph::{map_identity, GraphRuleStore};

    

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
                println!("Number of seed rules {}", rules.len());
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

        pub fn new(args: PiranhaArguments) -> Self {
            let language = args.language.get_language();
            let extension = args.language.get_extension();

            let graph_rule_store = GraphRuleStore::new(&args);

            let mut files = HashMap::new();
            let relevant_files = get_files_with_extension(&args.path_to_code_base, extension);

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
            let mut any_match = false;
            loop {
                if self.apply_rule_to_any_match(rule.clone(), rules_store, parser, scope_query) {
                    any_match = true;
                } else {
                    break;
                }
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
                if let Some((range, _)) = get_any_match_for_query_str(
                    self.ast.root_node(),
                    rules_store.get_query(scope_q),
                    self.code.as_bytes(),
                    true,
                ) {
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
                    let (parent_rules, method_level_rules, class_level_rules, _global_rules) =
                        rules_store.get_next(curr_rule.clone(), &self.substitutions);
                    println!("Method {} Class {}", method_level_rules.len(), class_level_rules.len());

                    let mut add_to_queue = |s: &str, rules: Vec<Rule>| {
                        for r in rules {
                            if let Some(scope_query) =
                                self.get_scope_query(String::from(s), previous_edit, rules_store)
                            {
                                if let Some(transformed_rule) =
                                    r.instantiate(&self.substitutions)
                                {
                                    scope_rule_queue.push((scope_query, transformed_rule));
                                } else {
                                    panic!(
                                        "Could not transform rule {:?} {:?}",
                                        r, &self.substitutions
                                    );
                                }
                            } else {
                                panic!("Could not create scope query {:?}", s);
                            }
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

                scope_rule_queue.reverse();
                for (sq, rle) in scope_rule_queue {
                    println!("Applying file level changes");
                    println!("Scope query {:?}", sq);
                    println!("Rule {:?}", rle);
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
                    if let Some((_, captures_by_tag)) = get_any_match_for_query_str(
                        parent,
                        rules_store.get_query(&m.matcher),
                        self.code.as_bytes(),
                        false,
                    ) {
                        let transformed_query = m
                            .matcher_gen
                            .substitute_rule_holes(&captures_by_tag);
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
            rules: &Vec<Rule>,
        ) -> Option<(Range, String, Rule, HashMap<String, String>)> {
            let context = self.get_context(previous_edit);

            for rule in rules {
                for ancestor in &context {
                    let cr = rule.clone();
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
            let all_relevant_query_matches = _get_all_relevant_matches_no_constraint(
                node,
                rule_store.get_query(&rule.query),
                source_code_bytes,
                recurssive,
            );

            if all_relevant_query_matches.is_empty() {
                return None;
            }

            for (range, captures_by_tag) in all_relevant_query_matches {
                let n = node
                    .descendant_for_byte_range(range.start_byte, range.end_byte)
                    .unwrap();
                
                if self.satisfies_constraint(
                    n,
                    rule_store.language,
                    rule.constraint.clone(),
                    source_code_bytes,
                    &captures_by_tag,
                ) {

                    let replacement = rule.replace.substitute_rule_holes(&captures_by_tag);
                    return Some((range.clone(), replacement, captures_by_tag));
                }
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
                    let query_str = q.substitute_rule_holes(&capture_by_tags);
                    println!("Inside satifies constraint {:?} {:?}", capture_by_tags, query_str);
                    if let Ok(query) = Query::new(language, &query_str) {
                        query_cache.insert(query_str, query);
                    }
                }
                let matcher_query = Query::new(language, &c.matcher).unwrap();

                // Apply matcher

                let mut curr_node = node;
                while let Some(parent) = curr_node.parent() {
                    if let Some((range, _)) = get_any_match_for_query_str(
                        parent,
                        &matcher_query,
                        source_code_bytes,
                        false,
                    ) {
                        let matcher = self.get_descendant(range.start_byte, range.end_byte);
                        let mut c_node = node;
                        while let Some(c_p) = c_node.parent() {
                            let mut all_queries_match = true;
                            for (_, query) in &query_cache {
                                all_queries_match = all_queries_match
                                    && get_any_match_for_query_str(
                                        matcher,
                                        query,
                                        source_code_bytes,
                                        true,
                                    )
                                    .is_some();
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

    fn get_any_match_for_query_str(
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
            if let Some(matched_node) = qm.captures.first() {
                let captured_tags = group_captures_by_tag(qm.captures, query, source_code_bytes);
                matched_node_query_match.collect_as_counter(matched_node.node.range(), captured_tags);
            }else{
                break;
            }
        }

        let mut output = vec![];
        for (range, v) in matched_node_query_match {
            if v.len() == pattern_count {
                if recurssive || node_matches_range(node, range){
                    let mut captures_by_tag = HashMap::new();
                    for i in v {
                        captures_by_tag.extend(i.clone());
                    }
                    output.push((range, captures_by_tag));
                }
            }
        }
        output.sort_by(|a, b| a.0.start_byte.cmp(&b.0.start_byte));
        return output;
    }
}
