mod config;
mod test;
mod tree_sitter;

pub mod piranha {
    use super::config::ResolvedRule;
    use super::tree_sitter as ts_utils;
    use crate::config::{read_config};
    use crate::tree_sitter::{get_node_captured_by_query, search_node};
    use std::collections::HashMap;
    use std::str;
    use tree_sitter::{Node, Parser, Query, QueryCapture, QueryCursor, Range, Tree};

    pub fn transform(input: &String, input_language: &str) -> (Tree, String) {
        let language = ts_utils::get_language(input_language);
        let (mut xp_api_rules, cleanups) = read_config();
        // let cleanups = get_cleanup_rules(input_language);

        let mut source_code = input.clone();
        let (mut parser, mut tree) = ts_utils::parse_code(language, &source_code);
        loop {
            if let Some((range, replacement, _rule)) =
                scan_for_xp_api_usages(&xp_api_rules, &tree.root_node(), &source_code)
            {
                let (new_tree, new_source_code) = update_capture_sites(
                    &mut parser,
                    &source_code,
                    &tree,
                    (range, replacement),
                    &cleanups,
                );
                tree = new_tree;
                source_code = new_source_code;

                // if rule.chained_rule.is_some() {
                //     let rule1 = rule.chained_rule.as_mut().unwrap();
                //     xp_api_rules.push(rule1);
                // }
            } else {
                return (tree, source_code);
            }
        }
    }



    fn update_capture_sites(
        parser: &mut Parser,
        source_code: &String,
        tree: &Tree,
        piranha_replacement: (Range, String),
        cleanup_rules: &Vec<ResolvedRule>,
    ) -> (Tree, String) {
        let mut capture = Option::Some(piranha_replacement.clone());
        let mut curr_source_code = source_code.clone();
        let mut curr_tree = tree.clone();

        loop {
            match capture {
                None => return (curr_tree, curr_source_code),

                Some((replace_range, replacement)) => {
                    println!(
                        "Updating site {:?}",
                        str::from_utf8(
                            &curr_source_code.as_bytes()
                                [replace_range.start_byte..replace_range.end_byte]
                        )
                    );

                    let (new_source_code, edit) =
                        ts_utils::get_edit(&curr_source_code, replace_range, &replacement);

                    curr_tree.edit(&edit);
                    // Update the remaining captures

                    let new_tree = parser
                        .parse(&new_source_code, Some(&curr_tree))
                        .expect("Could not generate new tree!");

                    let change_range = (&edit.start_byte, &edit.new_end_byte);
                        // &curr_tree .changed_ranges(&new_tree) .next() .expect("Could not get change range!");

                    curr_tree = new_tree;
                    curr_source_code = new_source_code;

                    let node = &curr_tree.root_node();
                    let changed_node = search_node(change_range.0.clone(), change_range.1.clone(), node);
                        // .expect("Node not found!");
                        // .root_node()
                        // .descendant_for_byte_range(change_range.0.clone(), change_range.1.clone())
                        // .expect("No descendant found for range");

                    if changed_node.start_byte() != change_range.0.clone() && changed_node.end_byte() != change_range.1.clone(){
                        println!("NODE IS NOT EXACT!!!")
                    }

                    println!("Cleaning up {:?}, {:?}, {:?}, {:?} {}",
                             change_range.0.clone(), change_range.1.clone(),
                             changed_node.start_byte(), changed_node.end_byte(),
                             str::from_utf8(
                                 &curr_source_code.as_bytes()
                                     [change_range.0.clone()..change_range.1.clone()]
                             ).unwrap());
                    println!("Cleaning up {:?}", changed_node.utf8_text(&curr_source_code.as_bytes()));

                    capture =
                        match get_relevant_parent(&changed_node, &curr_source_code, cleanup_rules) {
                            Some(relevant_parent) => {
                                println!("Found a parent to be cleaned up!");
                                Some((relevant_parent.0.range(), relevant_parent.1))
                            }
                            None => {
                                println!("No parent found to be cleaned up!");
                                None
                            }
                        }
                }
            }
        }
    }




    fn get_relevant_parent_helper<'a, 'b>(
        cleanup_rules: &Vec<ResolvedRule>,
        changed_node: &Node<'a>,
        new_source_code: &String,
        count: i8,
    ) -> Option<(Node<'a>, String)> {
        match count {
            0 => None,
            _ => {
                if let Some((replacement, _r)) = get_replacement_for_node(
                    cleanup_rules,
                    changed_node,
                    &new_source_code.as_bytes(),
                ) {
                    return Option::Some((changed_node.clone(), replacement));
                } else {
                    match changed_node.parent() {
                        Some(parent) => get_relevant_parent_helper(
                            cleanup_rules,
                            &parent,
                            new_source_code,
                            count - 1,
                        ),
                        None => None,
                    }
                }
            }
        }
    }

    fn get_relevant_parent<'a>(
        changed_node: &Node<'a>,
        new_source_code: &String,
        rules: &Vec<ResolvedRule>,
    ) -> Option<(Node<'a>, String)> {
        // let cleanup_rules = rules;
        get_relevant_parent_helper(rules, changed_node, new_source_code, 3)
    }

    // This is like comby rewrite.
    fn get_replacement(
        captures: &[QueryCapture],
        rule: &ResolvedRule,
        source_code_bytes: &[u8],
    ) -> String {
        let query: &Query = &rule.query;
        let mut output = String::from(&rule.rewrite_template);

        let mut var_capture = HashMap::new();
        for capture in captures {
            let name = query
                .capture_names()
                .get(capture.index as usize)
                .expect("Capture name not found!");
            let code_snippet = capture
                .node
                .utf8_text(source_code_bytes)
                .expect("Could not get source code for node");
            var_capture
                .entry(name)
                .or_insert_with(Vec::new)
                .push(code_snippet);
        }

        for capture_name in query.capture_names() {
            if rule
                .rewrite_template
                .contains(&format!("@{}", capture_name))
            {
                output = match var_capture.get(capture_name).map(|v| v.join("\n")) {
                    Some(replacement) => {
                        output.replace(&format!("@{}", capture_name), replacement.as_str())
                    }
                    None => output.replace(&format!("@{}", capture_name), ""),
                };
            }
        }
        return output;
    }

    // Returns the replacement for the first rule that matches node.
    fn get_replacement_for_node<'a, 'b>(
        rules: &'b Vec<ResolvedRule>,
        node: &'a Node,
        source_code_bytes: &[u8],
    ) -> Option<(String, &'b ResolvedRule)> {
        let source_code_bytes = source_code_bytes;
        let mut cursor = QueryCursor::new();
        for rule in rules {
            for query_match in cursor.matches(&rule.query, node.clone(), |_n| {
                &source_code_bytes[_n.byte_range()]
            }) {
                let captures: &[QueryCapture] = &query_match.captures;
                if let Ok(replace_node) = get_node_captured_by_query(captures) {
                    if are_s_exp_same(node, &replace_node) {
                        let replacement = get_replacement(captures, rule, source_code_bytes);
                        return Some((replacement, rule));
                    }
                }
            }
        }

        return None;
    }

    fn parenthesize(s: &String) -> String {
        return format!("(parenthesized_expression {})", s);
    }

    fn are_s_exp_same(node: &Node, replace_node: &Node) -> bool {
        println!("Replace node exp {}", replace_node.to_sexp());
        println!("Query node exp {}", node.to_sexp());
        let replace_sexp = replace_node.to_sexp();
        let query_node_sexp = node.to_sexp();
        let s1 = parenthesize(&replace_sexp);
        let s2 = &parenthesize(&query_node_sexp);
        // println!("s1 and s2 {} {}", s1, s2);
        replace_sexp.eq(&query_node_sexp) || s1.eq(&query_node_sexp) || replace_sexp.eq(s2)
    }

    fn scan_for_xp_api_usages<'a, 'b>(
        rules: &'b Vec<ResolvedRule>,
        node: &'a Node,
        source_code: &String,
    ) -> Option<(Range, String, &'b ResolvedRule)> {
        let mut i = 0;
        while i < node.child_count() {
            let c = node.child(i).unwrap();
            if let Some(usage_replacement) = scan_for_xp_api_usages(rules, &c, source_code) {
                return Some(usage_replacement);
            }
            i = i + 1;
        }
        if let Some((replacement, r)) =
            get_replacement_for_node(rules, &node, source_code.as_bytes())
        {
            return Option::Some((node.range(), replacement, r));
        }
        return None;
    }
}
