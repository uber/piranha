use colored::Colorize;
use serde_derive::Deserialize;
use tree_sitter::Query;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use crate::tree_sitter::get_language;
use crate::tree_sitter::substitute_tag_with_code;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub query: String,
    pub replace: String,
    and_then: Option<Vec<Rule>>,
    pub scope: String,
    pub language: String,
}

impl Rule {
    pub fn and_then(
        self,
        tag_matches: HashMap<String, String>,
    ) -> HashMap<Rule, Query> {
        // let mut and_then_queries = vec![];
        let mut rule_query_cache = HashMap::new();
        let ts_language = get_language(self.language.as_str());
        if self.and_then.is_some() {
            for r in self.and_then.unwrap() {
                let transformed_rule = Self::fill_holes(&r, &tag_matches);
                // for q in &transformed_rule.queries {
                let q = &transformed_rule.query;
                let query = Query::new(ts_language, q.as_str())
                    .expect(format!("Invalid Query generated, please check {}", q).as_str());
                println!("Added rule to cache");
                rule_query_cache.insert(r, query);
            }
        }
        rule_query_cache
    }

    fn map_key(s: &String) -> String {
        format!("[@{}]", s)
    }

    fn fill_holes(cr: &Rule, tag_substutions: &HashMap<String, String>) -> Rule {
        println!("Substitutions {:?}", tag_substutions);
        let rule = cr.clone();

        let new_query = substitute_tag_with_code(tag_substutions,&rule.query, &Self::map_key);
        println!("New query {}", new_query);
        let new_replace = substitute_tag_with_code(tag_substutions,&rule.replace, &Self::map_key);
        return Rule {
            name: rule.name,
            query: new_query,
            replace: new_replace,
            and_then: rule.and_then,
            scope: rule.scope,
            language: rule.language,
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

