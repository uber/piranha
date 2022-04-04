use colored::Colorize;
use serde_derive::Deserialize;
use core::panic;
use std::{collections::HashMap, hash::Hash};

use crate::tree_sitter::{TreeSitterHelpers, TagMatches};


pub struct PiranhaArguments {
    pub path_to_code_base: String,
    pub language: String,
    pub input_substitutions: TagMatches,
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

        let input_substitutions = HashMap::from([
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
            input_substitutions: TagMatches::new(input_substitutions),
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

    pub fn update(&self, query: String, replace: String) -> Self{
        Rule {
            name: String::from(&self.name),
            query,
            replace,
            holes: self.holes.clone(),
            tag: self.tag.clone(),
            constraint: self.constraint.clone(),
        }
    }

    pub fn is_feature_flag_cleanup(&self) -> bool {
        self.tag.as_ref()
                        .map(|tags| tags.iter().any(|t| t.eq( "Feature-flag API cleanup" )))
                        .unwrap_or(false)
    }

    pub fn instantiate(&self, substitutions: &TagMatches) -> Rule {
        if let Some(holes) = &self.holes {

            let relevant_substitutions = TagMatches::new(holes
                .iter()
                .filter_map(|hole| substitutions.get(hole).map(|subs| (hole, subs)))
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect());

            if relevant_substitutions.len() == holes.len() {
                return self.update(
                    self.query.substitute_rule_holes(&relevant_substitutions), 
                    self.replace.substitute_rule_holes(&relevant_substitutions));
            } else {
                #[rustfmt::skip]
                println!("Some Holes {:?} not found in table {:?}",  self.holes, substitutions);
                panic!("Could not instantiate a rule - {:?}. Some Holes {:?} not found in table {:?}", self, self.holes, substitutions);
            }
        }
        return self.clone();
    }
}
