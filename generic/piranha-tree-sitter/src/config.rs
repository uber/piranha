use colored::Colorize;
use serde_derive::Deserialize;
use std::{collections::HashMap, hash::Hash};

use crate::{tree_sitter::TreeSitterHelpers};

pub struct PiranhaArguments {
    pub path_to_code_base: String,
    pub language: String,
    pub input_substiution: HashMap<String, String>,
}

impl PiranhaArguments {
    pub fn new(
        path_to_code_base: &str,
        input_language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> Self {
        let flag_val = flag_value.eq("true");
        let (treated, treated_c) = (format!("{}", flag_val), format!("{}", !flag_val));

        let input_substiution = HashMap::from([
            (String::from("stale_flag_name"), String::from(flag_name)),
            (String::from("treated"), String::from(&treated)),
            (String::from("namespace"), String::from(flag_namespace)),
            (String::from("treated_complement"), String::from(&treated_c)),
        ]);

        let treated = format!("{}", flag_value.eq("true"));
        #[rustfmt::skip]
        println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {flag_name}\n (ii) Value: {treated} \n (iii) flag_namespace : {flag_namespace}").purple());
        Self {
            path_to_code_base: path_to_code_base.to_string(),
            language: input_language.to_string(),
            input_substiution,
        }
    }
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

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub query: String,
    pub replace: String,
    pub tag: Option<Vec<String>>,
    pub holes: Option<Vec<String>>,
    pub constraint: Option<Constraint>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
    pub matcher: String,
    pub predicate: String, // All, any , none
    pub queries: Vec<String>,
}

impl Rule {
    pub fn instantiate(&self, substitutions: &HashMap<String, String>) -> Option<Rule> {
        if self.holes.is_none() {
            Some(self.clone())
        } 
        //TODO: Cleanup
        else if self
            .holes
            .clone()
            .unwrap()
            .iter()
            .map(|x| x.replace("@", ""))
            .all(|x| substitutions.contains_key(&x))
        {
            let mut cc = HashMap::new();
            for (k, v) in substitutions {
                if self.holes.clone().unwrap().contains(&["@", k].join("")){
                    cc.insert(k.clone(), v.clone());
                }
            }
            Some(Rule {
                name: String::from(&self.name),
                query: self.query.substitute_rule_holes(&cc), //substitute_in_str(&substitutions, &self.query, &key_mapper),
                replace: self.replace.substitute_rule_holes(&cc),
                holes: self.holes.clone(),
                tag: self.tag.clone(),
                constraint: self.constraint.clone(),
            })
        } else {
            println!("Holes {:?} not found in table", self.holes);
            None
        }
    }
}
