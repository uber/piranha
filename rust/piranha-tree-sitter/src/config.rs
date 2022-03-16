use crate::tree_sitter as ts_utils;
use crate::tree_sitter::get_language;
use crate::tree_sitter::group_by_tag_str;
use crate::tree_sitter::substitute_tag_with_code;
use crate::utilities::get_extension;
use crate::utilities::get_files_with_extension;
use crate::utilities::read_file;
use colored::Colorize;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tree_sitter::InputEdit;
use tree_sitter::Language;
use tree_sitter::Node;
use tree_sitter::Parser;
use tree_sitter::Query;
use tree_sitter::QueryCursor;
use tree_sitter::QueryMatch;
use tree_sitter::Range;
use tree_sitter::Tree;

#[derive(Deserialize, Debug)]
struct Config {
    pub rules: Vec<Rule>
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
struct Rule {
    pub name: String,
    pub query: String,
    pub replace: String,
    and_then: Option<Vec<Rule>>,
    pub scope: String,
    pub language: String,
}

impl Rule {
    // pub fn and_then(
    //     self,
    //     tag_matches: HashMap<String, String>,
    // ) -> (Vec<Rule>, HashMap<String, Query>) {
    //     let mut and_then_queries = vec![];
    //     let mut rule_query_cache = HashMap::new();
    //     let ts_language = get_language(self.language.as_str());
    //     if self.and_then.is_some() {
    //         for r in self.and_then.unwrap() {
    //             let transformed_rule = &r.fill_holes(&tag_matches);
    //             for q in &transformed_rule.queries {
    //                 let query = Query::new(ts_language, q.as_str())
    //                     .expect(format!("Invalid Query generated, please check {}", q).as_str());
    //                 rule_query_cache.insert(String::from(q), query);
    //                 println!("Added rule to cache");
    //             }
    //             println!("Transformed rule {:?}", transformed_rule);
    //             and_then_queries.push(transformed_rule.clone());
    //         }
    //     }
    //     (and_then_queries, rule_query_cache)
    // }

    // fn fill_holes(self, tag_substutions: &HashMap<String, String>) -> Rule {
    //     println!("Substitutions {:?}", tag_substutions);
    //     let mut new_queries = vec![];
    //     for q in self.queries {
    //         let mut new_q = String::from(q);
    //         for (tag, substitute) in tag_substutions {
    //             let tag_as_template_var = format!("[@{}]", &tag);
    //             new_q = new_q.replace(tag_as_template_var.as_str(), &substitute);
    //         }
    //         println!("New query {}", new_q);
    //         new_queries.push(new_q);
    //     }
    //     let mut new_replace = String::from(self.replace);
    //     for (tag, substitute) in tag_substutions {
    //         let tag_as_template_var = format!("[@{}]", &tag);
    //         new_replace = new_replace.replace(&tag_as_template_var, &substitute);
    //     }
    //     return Rule {
    //         name: self.name,
    //         queries: new_queries,
    //         replace: new_replace,
    //         and_then: self.and_then,
    //         scope: self.scope,
    //         language: self.language,
    //     };
    // }
}

impl Config {
    pub fn read_config(
        language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> Config {
        println!("{}", format!("Loading Configs").purple());
        let path_buf_to_java_toml = match language {
            "Java" => Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("configurations")
                .join("java_rules.toml"),
            _ => panic!(),
        };
        let flag_val = flag_value.eq("true");
        let treated = format!("{}", flag_val);
        let treated_c = format!("{}", !flag_val);

        #[rustfmt::skip]
        println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {flag_name}\n (ii) Value: {treated} \n (iii) flag_namespace : {flag_namespace}").purple());

        let substitutions = HashMap::from([
            (String::from("[stale_flag_name]"), flag_name),
            (String::from("[treated]"), &treated),
            (String::from("[namespace]"), flag_namespace),
            (String::from("[treated_complement]"), &treated_c),
        ]);

        let path_to_config_toml = path_buf_to_java_toml.as_path();

        // Perform the replacement on the entire Config file
        let file_content = fs::read_to_string(&path_to_config_toml);
        if file_content.is_err() {
            panic!(
                "{}",
                format!(
                    "Could not load configuration file - \n {:?}",
                    file_content.err().unwrap()
                )
                .red()
            );
        }
        let mut content = file_content.unwrap();
        for (k, v) in &substitutions {
            content = content.replace(k, v);
        }

        let config: Config = toml::from_str(content.as_str()).unwrap();

        return config;
    }
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
        let config = Config::read_config(input_language, flag_name, flag_namespace, flag_value);
        let mut rule_query_cache = HashMap::new();

        let mut seed_rules = vec![];
        let mut cleanup_rules = vec![];
        for r in &config.rules {
            let query_str = r.query.as_str();
            let query = Query::new(language, query_str);
            if query.is_err(){
                panic!("Cannot process {query_str}");
            }
            rule_query_cache.insert(String::from(query_str), query.unwrap());
            if r.scope.eq("PROJECT"){
                seed_rules.push(r.clone());
            }else{
                cleanup_rules.push(r.clone());
            }
        }

        let rules_store = RulesStore {
            rule_query_cache, seed_rules, cleanup_rules
        };

        // let mut project_level_rules = vec![];
        // let mut cleanup_rules = vec![];
        // for r in &config.rules {
        //     if r.scope.eq("PROJECT") {
        //         project_level_rules.push(r.clone());
        //     } else {
        //         cleanup_rules.push(r.clone());
        //     }
        // }

        let mut files = HashMap::new();
        let relevant_files = get_files_with_extension(path_to_code_base, extension);

        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Could not set language");

        for dir_entry in relevant_files {
            let file_path = dir_entry.path();
            let code = read_file(&file_path);
            files.insert(file_path, SourceCodeUnit::parse(&mut parser, code));
        }

        Self {
            rules_store,
            language,
            files,
        }
    }

    pub fn cleanup(&mut self){
        let mut parser = Parser::new();
        parser
            .set_language(self.language)
            .expect("Could not set language");
        for (_, scu) in self.files.iter_mut() {
            scu.apply_seed_rules(&mut self.rules_store, &mut parser);
        }   
    }
}

pub struct SourceCodeUnit {
    pub ast: Tree,
    pub code: String,
}

impl SourceCodeUnit {
    // This method performs the input code replacement in the source code
    fn apply_edit(&mut self, code_replacement: (Range, String), parser: &mut Parser) -> InputEdit {
        let replace_range = code_replacement.0;
        let replacement = code_replacement.1;
        // let curr_source_code = source_code.clone();
        // let curr_tree = tree.clone();
        let (new_source_code, edit) =
            ts_utils::get_edit(self.code.as_str(), replace_range, &replacement);
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
    ) {
        loop {
            let cr = rule.clone();
            //TODO: Will return andThen rules too.
            let replacement = self.scan_and_match_rule(cr, rules_store);
            if replacement.is_none() {
                break;
            } else {
                let edit = self.apply_edit(replacement.unwrap(), parser);
                self.cleanup_previous_edit_site(edit, rules_store, parser);
            }
        }
    }

    fn apply_seed_rules(
        &mut self,
        rules_store: &mut RulesStore,
        parser: &mut Parser,
    ) {
        let rules = rules_store.seed_rules.clone();
        for rule in rules {
            self.apply_rule(rule.clone(), rules_store, parser);
        }
    }

    fn cleanup_previous_edit_site(
        &mut self,
        edit: InputEdit,
        rules_store: &mut RulesStore,
        parser: &mut Parser,
        
    ) {
        let mut previous_edit = edit.clone();
        loop {
            let replacement = self.match_cleanup_site(previous_edit, rules_store);
            if replacement.is_none() {
                break;
            } else {
                previous_edit = self.apply_edit(replacement.unwrap(), parser);
            }
        }
    }

    fn match_cleanup_site(
        &mut self,
        previous_edit: InputEdit,
        rules_store: &mut RulesStore,
    ) -> Option<(Range, String)> {
        let changed_node = self
            .ast
            .root_node()
            .descendant_for_byte_range(previous_edit.start_byte, previous_edit.new_end_byte)
            .unwrap();
        println!("Previously edited site {:?} {:?}", previous_edit.start_byte, previous_edit.new_end_byte);
        println!("Cleaning up site {:?} {:?} {:?}", changed_node.utf8_text(self.code.as_bytes()), changed_node.range().start_byte, changed_node.range().end_byte);
        let parent: Node = changed_node.parent().clone().unwrap();
        let grand_parent = parent.parent().clone().unwrap();
        let context = vec![changed_node, parent, grand_parent];

        let cleanup_rules = rules_store.cleanup_rules.clone();

        for rule in &cleanup_rules {
            for ancestor in &context {
                if let Some((range, replacement)) = match_rule(
                    rule.clone(),
                    rules_store,
                    ancestor.clone(),
                    self.code.as_bytes(),
                    false,
                ) {
                    return Some((range, replacement));
                }
            }
        }
        return None;
    }

    fn new(ast: Tree, code: String) -> Self {
        Self { ast, code }
    }

    fn parse(parser: &mut Parser, code: String) -> Self {
        let ast = parser.parse(&code, None).expect("Could not parse code");
        Self { ast, code }
    }

    fn scan_and_match_rule(
        &self,
        rule: Rule,
        rule_store: &mut RulesStore, // rule_query_cache: &mut HashMap<String, Query>,
    ) -> Option<(Range, String)> {
        let root = self.ast.root_node();
        let source_code_bytes = self.code.as_bytes();
        return match_rule(rule, rule_store, root, source_code_bytes, true);
    }
}

fn match_rule(
    rule: Rule,
    rule_store: &mut RulesStore,
    node: Node,
    source_code_bytes: &[u8],
    recurssive: bool,
) -> Option<(Range, String)> {
    let query_str = rule.query.as_str();
    if !rule_store.rule_query_cache.contains_key(query_str){
        panic!("{}",query_str);
    }
    // if !recurssive {
    //     print!("{:?} {:?}", node.start_position(), node.end_position());
    // }
    let query = rule_store.rule_query_cache.get(query_str).unwrap();
    let pattern_count = query.pattern_count();
    
    // TODO: extract parameter `cursor`
    let mut cursor = QueryCursor::new();
    let query_matches = cursor.matches(&query, node, source_code_bytes);
        // .into_iter()
        // .collect::<Vec<QueryMatch>>();
    let mut matched_node_query_match = HashMap::new();
    // println!("Query matches {:?}", query_matches.len());
    for qm in query_matches {
        // The assumption is that the first capture is the node that the pattern matches to.
        let captures = qm.captures;
        // for c in captures{
            // println!("{:?} \n {:?} \n\n", query.capture_names().get(c.index as usize), c.node.utf8_text(source_code_bytes))
        // }
        if captures.is_empty() {
            break;
        }
        let matched_node_range = captures.first().map(|z| z.node.range()).unwrap();
        println!("First Capture {:?}", captures.first());
        let mut captured_tags = group_by_tag_str(captures, query, source_code_bytes);
        for cn in query.capture_names(){
            captured_tags.entry(String::from(cn)).or_insert_with(String::new);
        }
        matched_node_query_match
            .entry(matched_node_range)
            .or_insert_with(Vec::new)
            .push(captured_tags);
    }

    let relevant_query_matches = matched_node_query_match
        .iter()
        .filter(|(_, v)| v.len() == pattern_count)
        .filter(|(k, _)| {
            recurssive || k.start_byte == node.start_byte() && k.end_byte == node.end_byte()
        })
        .next();
    
    let mut captures_by_tag = HashMap::new();
    if relevant_query_matches.is_some() {
        let relevant_match = relevant_query_matches.unwrap();
        for i in relevant_match.1 {
            captures_by_tag.extend(i.clone());
        }
        let replacement = substitute_tag_with_code(&captures_by_tag, &rule.replace);
        println!("{replacement}");
        return Some((relevant_match.0.clone(), replacement));
    }
    return None;
}
