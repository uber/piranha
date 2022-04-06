use colored::Colorize;
use core::panic;
use serde_derive::Deserialize;
use std::{collections::HashMap, hash::Hash};

use crate::tree_sitter::{TSQuery, TagMatches, TreeSitterHelpers};

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
    matcher: String,
    matcher_gen: String,
}

impl ScopeMatcher {
    pub fn get_matcher(&self) -> TSQuery {
        TSQuery::from(self.matcher.clone())
    }

    pub fn get_matcher_gen(&self) -> TSQuery {
        TSQuery::from(self.matcher_gen.clone())
    }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    query: TSQuery,
    pub replace: String,
    pub groups: Option<Vec<String>>,
    pub holes: Option<Vec<String>>,
    pub constraint: Option<Constraint>,
    pub grep_heuristics: Option<Vec<String>>
}
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Pred(String);

impl Pred {

    pub fn is_all(&self) -> bool {
        "All".eq(self.0.as_str())
    }

    pub fn is_none(&self) -> bool {
        "None".eq(self.0.as_str())
    }

    pub fn _is_any(&self) -> bool {
        "Any".eq(self.0.as_str())
    }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
    pub matcher: TSQuery,
    pub predicate: Pred, // All, any , none
    pub queries: Vec<TSQuery>,
}

impl Rule {
    pub fn update(&self, query: TSQuery, replace: String) -> Self {
        Rule {
            name: String::from(&self.name),
            query,
            replace,
            holes: self.holes.clone(),
            groups: self.groups.clone(),
            constraint: self.constraint.clone(),
            grep_heuristics : self.grep_heuristics.clone()
        }
    }

    pub fn is_feature_flag_cleanup(&self) -> bool {
        self.groups
            .as_ref()
            .map(|tags| tags.iter().any(|t| t.eq("Feature-flag API cleanup")))
            .unwrap_or(false)
    }

    pub fn instantiate(&self, substitutions: &TagMatches) -> Rule {
        if let Some(holes) = &self.holes {
            let relevant_substitutions = TagMatches::new(
                holes
                    .iter()
                    .filter_map(|hole| substitutions.get(hole).map(|subs| (hole, subs)))
                    .map(|(a, b)| (a.clone(), b.clone()))
                    .collect(),
            );

            if relevant_substitutions.len() == holes.len() {
                return self.update(
                    self.query.substitute_tags(&relevant_substitutions),
                    self.replace.substitute_tags(&relevant_substitutions),
                );
            } else {
                #[rustfmt::skip]
                panic!("Could not instantiate a rule - {:?}. Some Holes {:?} not found in table {:?}", self, self.holes, substitutions);
            }
        }
        return self.clone();
    }

    pub fn add_grep_heuristics_for_seed_rules(&mut self, substitutions: &TagMatches){   
        let mut gh = vec![];
        for h in self.holes.as_ref().unwrap() {
            if let Some(x) = substitutions.get(h){
                // We do not want to search for strings that occur only in replace.
                if self.query.contains(x)  {
                    gh.push(x.clone());
                }
            }
        }
        self.grep_heuristics = Some(gh.clone());
    }

    pub fn get_query(&self) -> TSQuery {
       self.query.clone()
    }
}
