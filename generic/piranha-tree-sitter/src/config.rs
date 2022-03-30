use std::collections::HashMap;

use serde_derive::Deserialize;

use crate::utilities::substitute_in_str;

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

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub query: String,
    pub replace: String,
    pub tag: Option<Vec<String>>,
    pub holes: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
    pub matcher: String,
    pub predicate: String, // All, any , none
    pub queries: Vec<String>,
}

pub fn map_key(s: &String) -> String {
    format!("[@{}]", s)
}

impl Rule {
    pub fn instantiate(&self, substitutions: &HashMap<String, String>, key_mapper: &dyn Fn(&String) -> String) -> Option<Rule> {
        if self.holes.is_none(){
            Some(self.clone())
        }else if self.holes.clone().unwrap().iter().all(|x|substitutions.contains_key(x)){
            Some(Rule {
                name: String::from(&self.name),
                query: substitute_in_str(&substitutions, &self.query, &key_mapper),
                replace: substitute_in_str(&substitutions, &self.replace, &key_mapper),
                holes: self.holes.clone(),
                tag: self.tag.clone(),
            })
        }else {
            None
        }  
    }
}

// impl Config {
//     pub fn read_config(
//         language: &str,
//         flag_name: &str,
//         flag_namespace: &str,
//         flag_value: &str,
//     ) -> (Config, Config, ScopeConfig) {
//         println!("{}", format!("Loading Configs").purple());
//         let path_to_config = Path::new(env!("CARGO_MANIFEST_DIR"))
//             .join("src")
//             .join("configurations");

//         let path_to_feature_flags_toml = match language {
//             "Java" => path_to_config.join("java_feature_flag_rules.toml"),
//             _ => panic!(),
//         };
//         let path_to_cleanups_toml = match language {
//             "Java" => path_to_config.join("java_cleanup_rules.toml"),
//             _ => panic!(),
//         };

//         let path_to_scope_config = match language {
//             "Java" => path_to_config.join("java_scope_config.toml"),
//             _ => panic!(),
//         };

//         let flag_val = flag_value.eq("true");
//         let treated = format!("{}", flag_val);
//         let treated_c = format!("{}", !flag_val);

//         #[rustfmt::skip]
//         println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {flag_name}\n (ii) Value: {treated} \n (iii) flag_namespace : {flag_namespace}").purple());

//         let substitutions = HashMap::from([
//             (String::from("[stale_flag_name]"), String::from(flag_name)),
//             (String::from("[treated]"), String::from(&treated)),
//             (String::from("[namespace]"), String::from(flag_namespace)),
//             (
//                 String::from("[treated_complement]"),
//                 String::from(&treated_c),
//             ),
//         ]);

//         fn identity(s: &String) -> String {
//             String::from(s)
//         }

//         let feature_flag_rules_content = substitute_in_str(
//             &substitutions,
//             &read_file(&path_to_feature_flags_toml),
//             &identity,
//         );
//         let feature_flag_config: Config =
//             toml::from_str(feature_flag_rules_content.as_str()).unwrap();

//         let cleanup_rules_content = substitute_in_str(
//             &substitutions,
//             &read_file(&path_to_cleanups_toml),
//             &identity,
//         );
//         let cleanup_config: Config = toml::from_str(cleanup_rules_content.as_str()).unwrap();

//         let scope_config_content = read_file(&path_to_scope_config);
//         let scope_config: ScopeConfig = toml::from_str(&scope_config_content).unwrap();

//         return (feature_flag_config, cleanup_config, scope_config);
//     }
// }

// pub struct RulesStore {
//     pub rule_query_cache: HashMap<String, Query>,
//     pub seed_rules: Vec<Rule>,
//     pub cleanup_rules: Vec<Rule>,
//     pub scopes: Vec<Scope>,
//     pub language: Language,
// }

// impl RulesStore {
//     pub fn new(
//         input_language: &str,
//         language: Language,
//         flag_name: &str,
//         flag_namespace: &str,
//         flag_value: &str,
//     ) -> RulesStore {
//         let (ff_config, cleanup_config, scope_config) =
//             Config::read_config(input_language, flag_name, flag_namespace, flag_value);

//         let mut rule_store = Self {
//             rule_query_cache: HashMap::new(),
//             seed_rules: ff_config.rules.clone(),
//             cleanup_rules: cleanup_config.rules.clone(),
//             scopes: scope_config.scopes.clone(),
//             language,
//         };

//         for r in &ff_config.rules {
//             rule_store.cache_query(String::from(r.query.as_str()));
//         }

//         for r in &cleanup_config.rules {
//             rule_store.cache_query(String::from(r.query.as_str()));
//         }

//         for s in &scope_config.scopes {
//             for r in s.rules.iter() {
//                 rule_store.cache_query(String::from(r.matcher.as_str()));
//             }
//         }

//         return rule_store;
//     }

//     pub fn cache_query(&mut self, query_str: String) {
//         let q = Query::new(self.language, &query_str);
//         if q.is_err() {
//             panic!("Could not parse the query : {}", query_str);
//         }
//         let query = q.unwrap();
//         let _ = self.rule_query_cache.insert(query_str, query);
//     }

//     // pub fn get_query(&self, query_str: String) -> &Query {
//     //     if !self.rule_query_cache.contains_key(&query_str){
//     //         self.cache_query(query_str.clone());
//     //     }
//     //     let r =  self.rule_query_cache.get(&query_str);
//     //     return r.unwrap();
//     // }
// }
