use crate::tree_sitter as ts_utils;
use crate::tree_sitter::get_language;
use crate::tree_sitter::get_node_captured_by_query;
use crate::tree_sitter::group_by_tag;
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
use tree_sitter::Parser;
use tree_sitter::Query;
use tree_sitter::QueryCapture;
use tree_sitter::QueryCursor;
use tree_sitter::Range;
use tree_sitter::Tree;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
    pub flag_name: String,
    pub flag_value: String,
    pub flag_namespace: String,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub queries: Vec<String>,
    pub replace: String,
    and_then: Option<Vec<Rule>>,
    pub scope: String,
    pub language: String,
}

impl Rule {
    pub fn and_then(
        self,
        tag_matches: HashMap<String, String>,
    ) -> (Vec<Rule>, HashMap<String, Query>) {
        let mut and_then_queries = vec![];
        let mut rule_query_cache = HashMap::new();
        let ts_language = get_language(self.language.as_str());
        if self.and_then.is_some() {
            for r in self.and_then.unwrap() {
                let transformed_rule = &r.substitute(&tag_matches);
                for q in &transformed_rule.queries {
                    let query = Query::new(ts_language, q.as_str())
                        .expect(format!("Invalid Query generated, please check {}", q).as_str());
                    rule_query_cache.insert(String::from(q), query);
                    println!("Added rule to cache");
                }
                println!("Transformed rule {:?}", transformed_rule);
                and_then_queries.push(transformed_rule.clone());
            }
        }
        (and_then_queries, rule_query_cache)
    }

    fn substitute(self, tag_substutions: &HashMap<String, String>) -> Rule {
        println!("Substitutions {:?}", tag_substutions);
        let mut new_queries = vec![];
        for q in self.queries {
            let mut new_q = String::from(q);
            for (tag, substitute) in tag_substutions {
                let tag_as_template_var = format!("[@{}]", &tag);
                new_q = new_q.replace(tag_as_template_var.as_str(), &substitute);
            }
            println!("New query {}", new_q);
            new_queries.push(new_q);
        }
        let mut new_replace = String::from(self.replace);
        for (tag, substitute) in tag_substutions {
            let tag_as_template_var = format!("[@{}]", &tag);
            new_replace = new_replace.replace(&tag_as_template_var, &substitute);
        }
        return Rule {
            name: self.name,
            queries: new_queries,
            replace: new_replace,
            and_then: self.and_then,
            scope: self.scope,
            language: self.language,
        };
    }
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

struct FlagCleaner {
    path_to_code_base: String,
    rule_query_cache: HashMap<String, Query>,
    seed_rules: Vec<Rule>,
    cleanup_rules: Vec<Rule>,
    language: Language,
    files: HashMap<PathBuf, SourceCodeUnit>,
}

impl FlagCleaner {
    fn new(
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

        for r in &config.rules {
            r.queries
                .iter()
                .filter_map(|query_str| {
                    Query::new(language, query_str).ok().map(|x| (query_str, x))
                })
                .for_each(|(query_str, query)| {
                    rule_query_cache.insert(String::from(query_str), query);
                });
        }

        let mut project_level_rules = vec![];
        let mut cleanup_rules = vec![];
        for r in &config.rules {
            if r.scope.eq("PROJECT") {
                project_level_rules.push(r.clone());
            } else {
                cleanup_rules.push(r.clone());
            }
        }

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
            path_to_code_base: String::from(path_to_code_base),
            rule_query_cache,
            seed_rules: project_level_rules,
            cleanup_rules,
            language,
            files
        }
    }
}

struct SourceCodeUnit {
    ast: Tree,
    code: String,
}

impl SourceCodeUnit {
    fn new(ast: Tree, code: String) -> Self {
        Self { ast, code }
    }

    fn parse(parser: &mut Parser, code: String) -> Self {
        let ast = parser
                .parse(&code, None)
                .expect("Could not parse code");
        Self{ast, code}
    }

    

    // This method performs the input code replacement in the source code
    fn apply_edit(self, code_replacement: (Range, String), parser: &mut Parser) -> Self {
        let replace_range = code_replacement.0;
        let replacement = code_replacement.1;
        // let curr_source_code = source_code.clone();
        // let curr_tree = tree.clone();
        let (new_source_code, edit) =
            ts_utils::get_edit(self.code.as_str(), replace_range, &replacement);
        let new_tree = parser
            .parse(&new_source_code, Some(&self.ast))
            .expect("Could not generate new tree!");
        return Self::new(new_tree, new_source_code);
    }

    // Will update all occurences of a rule in the code.
    fn apply_rule() { todo!()}


    //TODO: DO IT THE WAY IT IS ALREADY IMPLEMENTED - USING THE TRAVERSER
    fn match_rule(
        self,
        rule: Rule,
        mut cursor: QueryCursor,
        rule_query_cache: HashMap<String, Query>,
    ) {
        let root = self.ast.root_node();
        let source_code_bytes = self.code.as_bytes();
        let mut captures_by_tag = HashMap::new();
        let mut capture_range = vec![];
        for query_str in rule.queries {
            let query = rule_query_cache.get(&query_str).unwrap();
            let query_matches = cursor.matches(&query, root, source_code_bytes);
            if let Some(qm) = query_matches.into_iter().next() {
                let captures: &[QueryCapture] = qm.captures;
                if let Ok(replace_node) = get_node_captured_by_query(captures) {
                    let code_snippets_by_tag: HashMap<String, String> =
                        group_by_tag(captures, &query, source_code_bytes)
                            .iter()
                            .map(|(k, v)| (String::from(k), v.join("\n")))
                            .collect();
                    captures_by_tag.extend(code_snippets_by_tag);
                    capture_range.push(replace_node.range());
                    for tag in query.capture_names() {
                        if !captures_by_tag.contains_key(tag) {
                            captures_by_tag.insert(String::from(tag), String::new());
                        }
                    }
                }
            }
        }


    }
}
