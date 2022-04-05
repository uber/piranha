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
    query: String,
    pub replace: String,
    pub groups: Option<Vec<String>>,
    pub holes: Option<Vec<String>>,
    pub constraint: Option<Constraint>,
}

pub struct Pred(String);

impl Pred {
    pub fn new(s: String) -> Self {
        if ["All", "None", "Any"].contains(&s.as_str()) {
            return Pred(s);
        }
        panic!("Predicate value should be `All`, `None` or `Any`");
    }

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
    matcher: String,
    predicate: String, // All, any , none
    queries: Vec<String>,
}

impl Constraint {
    pub fn get_matcher(&self) -> TSQuery {
        TSQuery::from(self.matcher.clone())
    }
    pub fn get_queries(&self) -> Vec<TSQuery> {
        self.queries
            .iter()
            .map(|x| TSQuery::from(x.clone()))
            .collect()
    }

    pub fn get_predicate(&self) -> Pred {
        Pred::new(self.predicate.clone())
    }
}

impl Rule {
    pub fn update(&self, query: String, replace: String) -> Self {
        Rule {
            name: String::from(&self.name),
            query,
            replace,
            holes: self.holes.clone(),
            groups: self.groups.clone(),
            constraint: self.constraint.clone(),
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

    pub fn get_query(&self) -> TSQuery {
        TSQuery::from(self.query.clone())
    }
}
