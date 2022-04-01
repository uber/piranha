use std::{collections::HashMap};

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
    pub constraint: Option<Constraint>
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
    pub matcher: String,
    pub predicate: String, // All, any , none
    pub queries: Vec<String>,
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
                constraint: self.constraint.clone()
            })
        }else {
            println!("Holes {:?} not found in table", self.holes);
            None
        }  
    }
}