use colored::Colorize;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::panic;
use std::path::Path;
use tree_sitter::Language;
use tree_sitter::Query;

use crate::utilities::read_file;
use crate::utilities::substitute_in_str;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopeConfig {
    pub scopes: Vec<Scope>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Scope {
    pub name: String,
    pub rules: Vec<ScopeMatcher>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopeMatcher {
    pub matcher: String,
    pub matcher_gen: String,
}

// impl ScopeMatcher {
//     pub fn get_query(&self, language: Language) -> Query {
//         let q = Query::new(language, self.matcher.as_str());
//         if q.is_err() {
//             panic!("Could not create query for {:?}", self.matcher);
//         }
//         q.unwrap()
//     }
// }

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub query: String,
    pub replace: String,
    and_then: Option<Vec<Rule>>,
    pub and_then_scope: Option<String>,
    pub constraint: Option<Vec<Constraint>>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
    pub predicate_kind: String,
    pub matcher: String,   // ts-query
    pub frequency: String, // none, one, any
    pub query: String,
}

impl Rule {
    pub fn and_then(&self, tag_matches: HashMap<String, String>) -> Vec<Rule> {
        let mut and_then_rules = vec![];
        if self.and_then.is_some() {
            for cr in self.and_then.as_ref().unwrap() {
                let r = cr.clone();
                let transformed_rule = Rule {
                    name: r.name,
                    query: substitute_in_str(&tag_matches, &r.query, &map_key),
                    replace: substitute_in_str(&tag_matches, &r.replace, &map_key),
                    and_then: r.and_then,
                    and_then_scope: r.and_then_scope,
                    constraint: r.constraint,
                };
                // let query = transformed_rule.get_query(language);
                println!("Added rule to cache");
                and_then_rules.push(transformed_rule);
            }
        }
        and_then_rules
    }

    // pub fn get_query(&self, language: Language) -> Query {
    //     let q = Query::new(language, self.query.as_str());
    //     if q.is_err() {
    //         panic!("Could not create query for {:?}", self.query);
    //     }
    //     q.unwrap()
    // }
}

pub fn map_key(s: &String) -> String {
    format!("[@{}]", s)
}

impl Config {
    pub fn read_config(
        language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> (Config, Config, ScopeConfig) {
        
        println!("{}", format!("Loading Configs").purple());
        let path_to_config = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("configurations");

        let path_to_feature_flags_toml = match language {
            "Java" => path_to_config.join("java_feature_flag_rules.toml"),
            _ => panic!(),
        };
        let path_to_cleanups_toml = match language {
            "Java" => path_to_config.join("java_cleanup_rules.toml"),
            _ => panic!(),
        };

        let path_to_scope_config = match language {
            "Java" => path_to_config.join("java_scope_config.toml"),
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
            (
                String::from("[treated_complement]"),
                String::from(&treated_c),
            ),
        ]);

        fn identity(s: &String) -> String {
            String::from(s)
        }

        let feature_flag_rules_content = substitute_in_str(
            &substitutions,
            &read_file(&path_to_feature_flags_toml),
            &identity,
        );
        let feature_flag_config: Config =
            toml::from_str(feature_flag_rules_content.as_str()).unwrap();

        let cleanup_rules_content = substitute_in_str(
            &substitutions,
            &read_file(&path_to_cleanups_toml),
            &identity,
        );
        let cleanup_config: Config = toml::from_str(cleanup_rules_content.as_str()).unwrap();

        let scope_config_content = read_file(&path_to_scope_config);
        let scope_config: ScopeConfig = toml::from_str(&scope_config_content).unwrap();

        return (feature_flag_config, cleanup_config, scope_config);
    }
}

pub struct RulesStore {
    pub rule_query_cache: HashMap<String, Query>,
    pub seed_rules: Vec<Rule>,
    pub cleanup_rules: Vec<Rule>,
    pub scopes: Vec<Scope>,
    language: Language,
}

impl RulesStore {
    pub fn new(
        input_language: &str,
        language: Language,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> RulesStore {
        let (ff_config, cleanup_config, scope_config) =
            Config::read_config(input_language, flag_name, flag_namespace, flag_value);

        let mut rule_store = Self {
            rule_query_cache: HashMap::new(),
            seed_rules: ff_config.rules.clone(),
            cleanup_rules: cleanup_config.rules.clone(),
            scopes: scope_config.scopes.clone(),
            language,
        };

        for r in &ff_config.rules {
            rule_store.cache_query(String::from(r.query.as_str()));
        }

        for r in &cleanup_config.rules {
            rule_store.cache_query(String::from(r.query.as_str()));
        }

        for s in &scope_config.scopes {
            for r in s.rules.iter() {
                rule_store.cache_query(String::from(r.matcher.as_str()));
            }
        }

        return rule_store;
    }

    pub fn cache_query(&mut self, query_str: String) {
        let q = Query::new(self.language, &query_str);
        if q.is_err() {
            panic!("Could not parse the query : {}", query_str);
        }
        let query = q.unwrap();
        let _ = self.rule_query_cache.insert(query_str, query);
    }
}
