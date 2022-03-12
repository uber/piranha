mod config;
mod test;
mod tree_sitter;
mod utilities;

pub mod piranha {
    use super::tree_sitter as ts_utils;
    use crate::config::{Config, Rule};
    use crate::tree_sitter::get_node_captured_by_query;
    use crate::utilities::{get_extension, get_files_with_extension, read_file};

    use colored::Colorize;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tree_sitter::{Language, Node, Parser, Query, QueryCapture, QueryCursor, Range, Tree};
    use tree_sitter_traversal::{traverse, Order};

    // TODO: Add a string level ennry point

    pub fn get_cleanups_for_code_base(
        path_to_code_base: &str,
        input_language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> HashMap<PathBuf, String> {
        let language = ts_utils::get_language(input_language);
        let extension = get_extension(input_language);
        let config = Config::read_config(input_language, flag_name, flag_namespace, flag_value);
        let mut rule_query_cache = HashMap::new();
        //TODO: Move to associated method
        for r in &config.rules {
            r.queries
                .iter()
                .filter_map(|query_str| {
                    Query::new(language, query_str).ok().map(|x| (query_str, x))
                })
                .for_each(|(query_str, query)| {
                    // println!("{}", format!("{}", query_str).bright_red());
                    rule_query_cache.insert(String::from(query_str), query);
                });
        }
        println!("Rule query cache size {}", rule_query_cache.len());
        cleanup_code_base(
            path_to_code_base,
            extension,
            &config.rules,
            language,
            &mut rule_query_cache,
        )
    }

    pub fn cleanup_code_base(
        path_to_code_base: &str,
        extension: &str,
        all_rules: &Vec<Rule>,
        language: Language,
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> HashMap<PathBuf, String> {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Could not set language");

        let mut cache: HashMap<PathBuf, String> = HashMap::new();
        
        let mut project_level_rules = vec![];
        for r in all_rules {
            if r.scope.eq("PROJECT"){
                project_level_rules.push(r.clone());
            }
        }

        loop {
            println!("Project level rules {}", project_level_rules.len());
            let (all_code_replacements_in_codebase, new_and_then_rules) = get_rewrites_for_directory(
                &project_level_rules,
                path_to_code_base,
                extension,
                &cache,
                &mut parser,
                rule_query_cache,
            );

            #[rustfmt::skip]
            println!("{}",format!("Number of files that matched rules {}", all_code_replacements_in_codebase.len()).purple());

            if all_code_replacements_in_codebase.is_empty() {
                break;
            }

            println!("Rule Query Cache len {}", &rule_query_cache.len());
            if !new_and_then_rules.is_empty(){
                println!("{}", format!("Adding new and then rules - ").yellow());
                for n in new_and_then_rules{
                     println!("{}", format!("{}",&n.name).yellow());
                     println!("Query {}", format!("{}",&n.queries.join("\n")).yellow());
                     println!("Replace {}", format!("{}",&n.replace).yellow());
                     project_level_rules.push(n);
                }
            }

            for (path, (code, ast, replacements_by_file_by_rule)) in
                all_code_replacements_in_codebase
            {   

                #[rustfmt::skip]
                println!("{}",format!("{} rules matched in {:?} .\n{} replacements to be performed. ",replacements_by_file_by_rule.len(),path,replacements_by_file_by_rule.values().flatten().count()).purple().bold());

                let new_source_code = apply_rules_to_file(
                    &replacements_by_file_by_rule,
                    &mut parser,
                    code,
                    ast,
                    all_rules,
                    rule_query_cache,
                );
                cache.insert(path, new_source_code);
            }
        }
        cache
    }

    fn apply_rules_to_file(
        replacements_by_file_by_rule: &HashMap<Rule, Vec<(Range, String)>>,
        parser: &mut Parser,
        code: String,
        ast: Tree,
        all_rules: &Vec<Rule>,
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> String {
        let mut tree = ast.clone();
        let mut source_code = code.clone();
        let mut rules_for_file = vec![];
        let mut code_replacements_for_file = vec![];

        for (k, v) in replacements_by_file_by_rule {
            rules_for_file.push(k.clone());
            code_replacements_for_file.extend(v.clone());
        }

        loop {
            if code_replacements_for_file.is_empty() {
                println!("{}", format!("Finished updating the file!").yellow());
                return source_code;
            }

            loop {
                let head = code_replacements_for_file.pop();

                if head.is_none() {
                    break;
                }

                let (range, replacement) = head.unwrap();
                let (new_tree, new_source_code, synced_nodes) =
                    apply_rule_and_then_cleanup_with_sync(
                        parser,
                        &source_code,
                        &tree,
                        (range, replacement),
                        &all_rules,
                        &code_replacements_for_file,
                        rule_query_cache,
                    );
                tree = new_tree;
                source_code = new_source_code;
                code_replacements_for_file = synced_nodes;
            }

            let (replacements_by_rule, new_and_then_rules) = get_rewrites_for_node(
                &rules_for_file,
                &tree.root_node(),
                &source_code,
                rule_query_cache,
            );
            code_replacements_for_file = vec![];
            for (_k, v) in replacements_by_rule {
                code_replacements_for_file.extend(v.clone());
            }
        }
    }

    fn apply_rule_and_then_cleanup_with_sync<'a>(
        parser: &mut Parser,
        source_code: &String,
        tree: &Tree,
        code_replacement: (Range, String),
        all_rules: &Vec<Rule>,
        sync_replacements: &Vec<(Range, String)>,
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> (Tree, String, Vec<(Range, String)>) {
        let mut sync_nodes = search_nodes(tree, sync_replacements);
        let mut capture = Option::Some(code_replacement.clone());
        let mut curr_source_code = source_code.clone();
        let mut curr_tree = tree.clone();

        let cleanup_rules = &all_rules
            .into_iter()
            .filter(|r| r.scope.eq("PARENT"))
            .map(|c| c.clone())
            .collect();
        loop {
            match capture {
                None => {
                    let mut temp = vec![];
                    for sn in sync_nodes.iter().enumerate() {
                        temp.push((sn.1 .0.range(), String::from(&sn.1 .1)));
                    }
                    return (curr_tree, curr_source_code, temp);
                }

                Some((replace_range, replacement)) => {
                    let (new_source_code, edit) =
                        ts_utils::get_edit(&curr_source_code, replace_range, &replacement);

                    curr_tree.edit(&edit);

                    sync_nodes = sync_nodes_and_remove_overlapping_edits(&sync_nodes, edit);

                    let new_tree = parser
                        .parse(&new_source_code, Some(&curr_tree))
                        .expect("Could not generate new tree!");

                    curr_tree = new_tree;
                    curr_source_code = new_source_code;

                    // let change_range = (&edit.start_byte, &edit.new_end_byte);
                    let changed_node = &curr_tree
                        .root_node()
                        .descendant_for_byte_range(edit.start_byte, edit.new_end_byte)
                        .unwrap();

                    capture = match_parent_rules(
                        &changed_node,
                        &curr_source_code,
                        cleanup_rules,
                        rule_query_cache,
                    );

                    #[rustfmt::skip]
                    println!("{}",(if capture.is_some() { "Cleaning up parent." } else { "No parent for deeper clean up. Finished updating and cleaning up the site!" }).bold().yellow()
                    );
                }
            }
        }
    }

    // TODO: Implement one traversal solution
    fn search_nodes<'a, 'b>(
        tree: &'a Tree,
        sync_replacements: &Vec<(Range, String)>,
    ) -> Vec<(Node<'a>, String)> {
        let root_node = tree.root_node().clone();
        sync_replacements
            .iter()
            .map(|s| {
                let z = root_node
                    .descendant_for_byte_range(s.0.start_byte, s.0.end_byte)
                    .unwrap();
                (z, String::from(&s.1))
            })
            .collect::<Vec<(Node, String)>>()
    }

    fn sync_nodes_and_remove_overlapping_edits<'a, 'b>(
        sync_nodes: &Vec<(Node<'a>, String)>,
        edit: tree_sitter::InputEdit,
    ) -> Vec<(Node<'a>, String)> {
        let mut new_sync_nodes = vec![];
        for sn in sync_nodes {
            if &edit.start_byte <= &sn.0.range().start_byte
                && &edit.old_end_byte >= &sn.0.range().end_byte
            {
                continue;
            }
            let mut n1 = sn.0.clone();
            n1.edit(&edit);
            new_sync_nodes.push((n1, String::from(&sn.1)));
        }
        new_sync_nodes
    }

    fn match_parent_rules<'a>(
        updated_node: &Node<'a>,
        source_code: &String,
        rules: &Vec<Rule>,
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> Option<(Range, String)> {
        let cc = updated_node.clone();
        let parent: Node = cc.parent().clone().unwrap();
        let grand_parent = parent.parent().clone().unwrap();
        let context = vec![cc, parent, grand_parent];

        // Apply each rule to the three ancestors. Short-circuit on finding the replacement.
        for rule in rules {
            for ancestor in &context {
                if let Some((replacement, and_then_rules)) =
                    apply_rule(&rule, ancestor, source_code.as_bytes(), rule_query_cache)
                {   
                    return Some((ancestor.range(), replacement));
                }
            }
        }
        return None;
    }

    fn substitute_tag_with_code(
        tag_substitutes: &HashMap<String, String>,
        rewrite_template: &String,
    ) -> String {
        let mut output = String::from(rewrite_template);
        for (tag, substitute) in tag_substitutes {
            output = output.replace(&format!("@{}", tag), substitute)
        }
        output
    }

    fn group_by_tag<'a>(
        captures: &[QueryCapture],
        query: &'a Query,
        source_code_bytes: &'a [u8],
    ) -> HashMap<String, Vec<String>> {
        let mut tag_capture = HashMap::new();
        // let capture_names = &query.capture_names();
        for capture in captures {
            let name = query
                .capture_names()
                .get(capture.index as usize)
                .expect("Capture name not found!");
            let code_snippet = capture
                .node
                .utf8_text(source_code_bytes)
                .expect("Could not get source code for node");
            tag_capture
                .entry(String::from(name))
                .or_insert_with(Vec::new)
                .push(String::from(code_snippet));
        }
        tag_capture
    }

    fn apply_rule<'a, 'b>(
        rule: &Rule,
        node: &'a Node,
        source_code_bytes: &[u8],
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> Option<(String, Vec<Rule>)> {
        let tag_matches =
            get_tag_matches_for_rule(rule, node, source_code_bytes, rule_query_cache);
        if tag_matches.is_empty() {
            return None;
        }
        let replacement = substitute_tag_with_code(&tag_matches, &rule.replace);
        let cr = rule.clone();
        let (and_then_rules, new_rule_queries) = cr.and_then(tag_matches);
        if !new_rule_queries.is_empty(){
            rule_query_cache.extend(new_rule_queries);
            println!("Added to Rule Query cache");
        }
        return Some((replacement,and_then_rules));
    }

    // If result is empty it implies that the rule did not match the node.
    fn get_tag_matches_for_rule<'a>(
        rule: &'a Rule,
        node: &'a Node,
        source_code_bytes: &'a [u8],
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> HashMap<String, String> {
        let mut cursor = QueryCursor::new();
        let mut captures_by_tag = HashMap::new();
        let mut r_node = 0;
        for query_str in &rule.queries {

            let query = rule_query_cache.get(query_str).unwrap();

            let query_matches = cursor.matches(&query, node.clone(), source_code_bytes);
            if let Some(qm) = query_matches.into_iter().next() {
                let captures: &[QueryCapture] = qm.captures;
                if let Ok(replace_node) = get_node_captured_by_query(captures) {
                    // FIXME: Compare tree not s-expression
                    if node.to_sexp().eq(&replace_node.to_sexp()) {
                        let code_snippets_by_tag: HashMap<String, String> =
                            group_by_tag(captures, &query, source_code_bytes)
                                .iter()
                                .map(|(k, v)| (String::from(k), v.join("\n")))
                                .collect();

                        captures_by_tag.extend(code_snippets_by_tag);
                        r_node = r_node + 1;
                        for tag in query.capture_names() {
                            if !captures_by_tag.contains_key(tag) {
                                captures_by_tag.insert(String::from(tag), String::new());
                            }
                        }
                    }
                    // break;
                }
            }
        }
        // all queries in the rule do not match the query node
        if r_node != rule.queries.len() {
            return HashMap::new();
        }
        return captures_by_tag;
    }

    fn get_replacement_for_node<'a, 'b>(
        rules: &Vec<Rule>,
        node: &'a Node,
        source_code_bytes: &[u8],
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> Option<(String, Rule, Vec<Rule>)> {
        for rule in rules {
            if let Some((replacement, and_then_rules)) =
                apply_rule(rule, &node, &source_code_bytes, rule_query_cache)
            {
                return Some((replacement, rule.clone(), and_then_rules));
            }
        }
        return None;
    }

    fn get_rewrites_for_node<'b>(
        rules: &Vec<Rule>,
        node: &Node,
        source_code: &String,
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> (HashMap<Rule, Vec<(Range, String)>>, Vec<Rule>) {
        let preorder: Vec<Node<'_>> = traverse(node.walk(), Order::Pre).collect::<Vec<_>>();
        let mut code_replacements_in_node = HashMap::new();
        let mut new_and_then_rules = vec![];

        for node in preorder {
            let replacement_info =
                get_replacement_for_node(rules, &node, source_code.as_bytes(), rule_query_cache)
                    .map(|replacement| (node.range(), replacement));

            if replacement_info.is_none() {
                continue;
            }

            let (range, (replacement, rule, and_then_rules)) = replacement_info.unwrap();
            
            new_and_then_rules.extend(and_then_rules);
            

            // let cr = &rule.clone();

            code_replacements_in_node
                .entry(rule.clone())
                .or_insert_with(Vec::new)
                .push((range, replacement));
        }

        (code_replacements_in_node, new_and_then_rules)
    }

    fn get_rewrites_for_directory<'b>(
        rules: &Vec<Rule>,
        path_to_code_base: &str,
        extension: &str,
        cache: &HashMap<PathBuf, String>,
        parser: &mut Parser,
        rule_query_cache: &mut HashMap<String, Query>,
    ) -> (HashMap<PathBuf, (String, Tree, HashMap<Rule, Vec<(Range, String)>>)>, Vec<Rule>) {
        let mut matches_by_file_by_rule = HashMap::new();
        let files = get_files_with_extension(path_to_code_base, extension);
        let mut new_and_then_rules = vec![];
        for dir_entry in files {
            let file_path = dir_entry.path();
            // Get content for file from cache or read from file system
            let source_code = cache
                .get(&file_path)
                .map(|x| x.to_owned())
                .unwrap_or_else(|| read_file(&file_path));

            let tree = parser
                .parse(&source_code, None)
                .expect("Could not parse code");

            let (code_replacements_in_file, and_then_rules) =
                get_rewrites_for_node(rules, &tree.root_node(), &source_code, rule_query_cache);

            if !code_replacements_in_file.is_empty() {
                println!("{}",format!("Rule matched in file {:?}", file_path).purple());
                matches_by_file_by_rule
                    .insert(file_path, (source_code, tree, code_replacements_in_file));
            }
            new_and_then_rules.extend(and_then_rules);
        }
        (matches_by_file_by_rule, new_and_then_rules)
    }
}
