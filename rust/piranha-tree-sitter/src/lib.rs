mod configs;
mod test;
mod tree_sitter;

pub mod piranha {
    use super::configs::Rule;
    use super::tree_sitter as ts_utils;
    use crate::configs::{java_rules::cleanup_rules, java_rules::piranha_rules};
    use crate::tree_sitter::get_node_for_range;
    use std::collections::HashMap;
    use std::str;
    use tree_sitter::{
        Language, Node, Parser, Query, QueryCapture, QueryCursor, QueryMatch, Range, Tree,
    };

    pub fn transform(input: &String, input_language: &str) -> (Tree, String) {
        let language = ts_utils::get_language(input_language).expect("Language not supported");
        let piranha_rules = piranha_rules();
        let mut source_code = input.clone();
        let (mut parser, mut tree) = ts_utils::parse_code(language, &source_code);
        loop {
            let moved_tree = tree;
            let moved_source_code = source_code;
            let mut all_captures_for_queries: Vec<(Range, String)>;

            all_captures_for_queries = get_replacements_for_tree(
                &piranha_rules,
                &moved_tree,
                &moved_source_code,
                language,
            );

            if all_captures_for_queries.is_empty() {
                return (moved_tree, moved_source_code);
            }

            let first_capture = &all_captures_for_queries.pop();

            let (new_tree, new_source_code) = update_capture_sites(
                &mut parser,
                language,
                &moved_source_code,
                &moved_tree,
                &first_capture,
            );

            tree = new_tree;
            source_code = new_source_code;
        }
    }

    fn update_capture_sites(
        parser: &mut Parser,
        language: Language,
        source_code: &String,
        tree: &Tree,
        piranha_site: &Option<(Range, String)>,
    ) -> (Tree, String) {
        let cleanup_rules = cleanup_rules();
        let mut capture = piranha_site.clone();
        let mut curr_source_code = source_code.clone();
        let mut curr_tree = tree.clone();
        while capture.is_some() {
            let (replace_range, replacement) = capture.unwrap();
            println!(
                "Updating site {:?}",
                str::from_utf8(
                    &curr_source_code.as_bytes()[replace_range.start_byte..replace_range.end_byte]
                )
            );

            // get the edit to be performed
            let (new_source_code, edit) =
                ts_utils::get_edit(&curr_source_code, replace_range, &replacement);

            curr_tree.edit(&edit);
            // Update the remaining captures
            let new_tree = parser.parse(&new_source_code, Some(&curr_tree)).unwrap();

            let change_range = &curr_tree.changed_ranges(&new_tree).collect::<Vec<Range>>();
            let cr = change_range.get(0).unwrap();

            curr_tree = new_tree;
            curr_source_code = new_source_code;

            let changed_node = get_node_for_range(cr, &curr_tree).unwrap();

            println!("FINDING PARENT!");
            let relevant_parent =
                get_relevant_parent(language, &cleanup_rules, changed_node, &curr_source_code);
            if relevant_parent.is_some() {
                println!("FOUND PARENT!");
                let rp = relevant_parent.unwrap();
                capture = Some((rp.0.range(), rp.1));
            } else {
                println!("PARENT NOT FOUND!");
                capture = None;
            }
        }

        return (curr_tree, curr_source_code);
    }

    fn get_relevant_parent<'a, 'b>(
        language: Language,
        cleanup_rules: &Vec<Rule>,
        changed_node: Node<'a>,
        new_source_code: &String,
    ) -> Option<(Node<'a>, String)> {
        println!(
            "Changed node 1- {:?}",
            changed_node.utf8_text(new_source_code.as_bytes())
        );
        let replacement =
            get_replacement_for_node(cleanup_rules, &changed_node, &new_source_code, language);
        if replacement.is_some() {
            println!("Found some!");
            // let node: z = p1.unwrap();
            return Option::Some((changed_node, replacement.unwrap()));
        }
        let p1 = changed_node.parent();
        if p1.is_some() {
            println!(
                "Parent 1- {:?}",
                p1.unwrap().utf8_text(new_source_code.as_bytes())
            );
            let replacement =
                get_replacement_for_node(cleanup_rules, &p1.unwrap(), &new_source_code, language);
            if replacement.is_some() {
                println!("Found some!");
                return Option::Some((p1.unwrap(), replacement.unwrap()));
            } else {
                let p2 = changed_node.parent().and_then(|p| p.parent());
                println!(
                    "Parent 2- {:?}",
                    p2.unwrap().utf8_text(new_source_code.as_bytes())
                );
                let replacement = get_replacement_for_node(
                    cleanup_rules,
                    &p2.unwrap(),
                    &new_source_code,
                    language,
                );
                if replacement.is_some() {
                    println!("Found some!");
                    return Option::Some((p2.unwrap().clone(), replacement.unwrap()));
                }
            }
        }

        return None;
    }

    // This replaces comby rewrite.
    fn get_replacement(
        captures: &[QueryCapture],
        rule: &Rule,
        query: &Query,
        source_code_bytes: &[u8],
    ) -> String {
        let mut output = String::from(&rule.rewrite_template);

        // todo group captures by id. One id can have more than one captures.

        // Create a map here Map<Index, [captures]>
        // replace var name with join([captures]

        let mut var_capture = HashMap::new();
        for capture in captures {
            let name = query
                .capture_names()
                .get(capture.index as usize)
                .expect("Capture name not found!");
            let code_snippet = capture.node.utf8_text(source_code_bytes).unwrap();
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
                let replacement = var_capture.get(capture_name).map(|v| v.join("\n"));
                if replacement.is_some() {
                    output = output
                        .replace(&format!("@{}", capture_name), replacement.unwrap().as_str());
                } else {
                    output = output.replace(&format!("@{}", capture_name), "")
                }
            }
        }
        return output;
    }

    fn get_replacement_for_node<'a>(
        rules: &Vec<Rule>,
        node: &'a Node,
        source_code: &String,
        language: Language,
    ) -> Option<String> {
        println!("{}", &source_code);
        let source_code_bytes = source_code.as_bytes();

        for rule in rules {
            let query_str = rule.query.as_str();
            let query = Query::new(language, query_str).unwrap();
            let mut cursor = QueryCursor::new();
            println!("Query str {:?}", query_str);
            let matches = cursor
                .matches(&query, node.clone(), |_n| {
                    &source_code_bytes[_n.byte_range()]
                })
                .collect::<Vec<QueryMatch>>();
            if matches.len() > 0 {
                for query_match in matches {
                    let captures: &[QueryCapture] = &query_match.captures;
                    for c in captures {
                        println!("{:?}, {:?}", c.node.utf8_text(source_code_bytes), c.index);
                    }
                    let replace_node =
                        ts_utils::get_replace_node(&query, &rule, captures, source_code_bytes);
                    println!("Replace node exp {}", replace_node.unwrap().to_sexp());
                    println!("Query node exp {}", node.to_sexp());
                    if replace_node.is_some() && are_s_exp_same(node, &replace_node.unwrap()) {
                        println!("Found replace node");
                        return Some(get_replacement(captures, &rule, &query, source_code_bytes));
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

        let replace_sexp = replace_node.to_sexp();
        let query_node_sexp = node.to_sexp();
        let s1 = parenthesize(&replace_sexp);
        let s2 = &parenthesize(&query_node_sexp);
        println!("s1 and s2 {} {}", s1, s2);
        replace_sexp.eq(&query_node_sexp)
            || s1.eq(&query_node_sexp)
            || replace_sexp.eq(s2)
    }

    fn get_replacements_for_tree<'a>(
        rules: &Vec<Rule>,
        tree: &'a Tree,
        source_code: &String,
        language: Language,
    ) -> Vec<(Range, String)> {
        let mut perfect_match_captures: Vec<(Range, String)> = vec![];
        let source_code_bytes = source_code.as_bytes();
        for rule in rules {
            let mut cursor = QueryCursor::new();
            let query = Query::new(language, rule.query.as_str()).unwrap();
            let matches = cursor.matches(&query, tree.root_node(), |_n| {
                &source_code_bytes[_n.byte_range()]
            });
            for query_match in matches {
                let captures: &[QueryCapture] = &query_match.captures;
                let replace_node =
                    ts_utils::get_replace_node(&query, &rule, captures, source_code_bytes).expect(
                        format!(
                            "Please check the rules. The rule {} does not contain the variable @{}",
                            rule.query, rule.complete_capture_var
                        )
                        .as_str(),
                    );
                let rewritten_source = get_replacement(captures, &rule, &query, source_code_bytes);

                perfect_match_captures.push((replace_node.range(), rewritten_source));
            }
        }
        return perfect_match_captures;
    }
}
