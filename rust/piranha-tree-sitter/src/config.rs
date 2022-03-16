use colored::Colorize;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

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

