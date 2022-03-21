use colored::Colorize;
use serde_derive::Deserialize;
use tree_sitter::Language;
use tree_sitter::Query;
use std::collections::HashMap;
use std::env;
use std::panic;
use std::path::Path;

use crate::utilities::substitute_in_str;
use crate::utilities::read_file;

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
    pub and_then_scope : Option<String>, 
    pub constraint: Option<Vec<Constraint>>
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
    pub predicate_kind : String, // contains, matches/ does not contain
    pub matcher: String, // ts-query 
    pub frequency : String, // none, one, any
    pub query: String 
}


impl Rule {
    pub fn and_then(
        self,
        tag_matches: HashMap<String, String>,
        language: Language
    ) -> HashMap<Rule, Query> {
        let mut rule_query_cache = HashMap::new();
        if self.and_then.is_some() {
            for r in self.and_then.unwrap() {
                let transformed_rule = Self::fill_holes(&r, &tag_matches);
                // let q = &transformed_rule.query;
                let query = transformed_rule.get_query(language);
                println!("Added rule to cache");
                rule_query_cache.insert(transformed_rule, query);
            }
        }
        rule_query_cache
    }

    pub fn get_query(&self, language: Language) -> Query {
        let q = Query::new(language, self.query.as_str());
        if q.is_err() {
            panic!("Could not create query for {:?}", self.query);
        }
        q.unwrap()
    }

    fn map_key(s: &String) -> String {
        format!("[@{}]", s)
    }

    fn fill_holes(cr: &Rule, tag_substutions: &HashMap<String, String>) -> Rule {
        println!("Substitutions {:?}", tag_substutions);
        let rule = cr.clone();

        let new_query = substitute_in_str(tag_substutions,&rule.query, &Self::map_key);
        println!("New query {}", new_query);
        let new_replace = substitute_in_str(tag_substutions,&rule.replace, &Self::map_key);
        println!("{}", format!("Filled hole {new_replace}").bright_blue());
        return Rule {
            name: rule.name,
            query: new_query,
            replace: new_replace,
            and_then: rule.and_then,
            and_then_scope: rule.and_then_scope,
            constraint : rule.constraint
        };
    }
}

impl Config {
    pub fn read_config(
        language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> (Config, Config) {
        println!("{}", format!("Loading Configs").purple());
        let path_to_feature_flags_toml = match language {
            "Java" => Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("configurations")
                .join("java_feature_flag_rules.toml"),
            _ => panic!(),
        };
        let path_to_cleanups_toml = match language {
            "Java" => Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("configurations")
                .join("java_cleanup_rules.toml"),
            _ => panic!(),
        };

        let flag_val = flag_value.eq("true");
        let treated = format!("{}", flag_val);
        let treated_c = format!("{}", !flag_val);

        #[rustfmt::skip]
        println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {flag_name}\n (ii) Value: {treated} \n (iii) flag_namespace : {flag_namespace}").purple());

        let substitutions = HashMap::from([
            (String::from("[stale_flag_name]"), String::from(flag_name)),
            (String::from("[treated]"), String::from(&treated)),
            (String::from("[namespace]"), String::from(flag_namespace)),
            (String::from("[treated_complement]"), String::from(&treated_c)),
        ]);

        fn identity(s:&String) -> String {
            String::from(s)
        }

        let feature_flag_rules_content = substitute_in_str(&substitutions, &read_file(&path_to_feature_flags_toml),&identity );
        let feature_flag_config: Config = toml::from_str(feature_flag_rules_content.as_str()).unwrap();

        let cleanup_rules_content = substitute_in_str(&substitutions, &read_file(&path_to_cleanups_toml),&identity );
        let cleanup_config: Config = toml::from_str(cleanup_rules_content.as_str()).unwrap();

        return (feature_flag_config, cleanup_config);
    }
}

