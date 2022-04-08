use colored::Colorize;
use serde_derive::Deserialize;
use std::{collections::HashMap, hash::Hash};
use clap::Parser;
use crate::tree_sitter::{TSQuery, TagMatches, TreeSitterHelpers};


#[derive(Clone,Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub path_to_codebase: String,
    #[clap(short, long)]
    pub language: String,
    #[clap(short, long)]
    pub flag_name: String,
    #[clap(short, long)]
    pub flag_namespace: String,
    #[clap(short, long)]
    pub flag_value: String,
    #[clap(short, long)]
    pub path_to_configuration: String
}

pub struct PiranhaArguments {
    pub path_to_code_base: String,
    pub language: String,
    pub input_substitutions: TagMatches,
    pub path_to_configurations: String,
}

impl PiranhaArguments {
    pub fn new(args: Args) -> Self {
        let flag_val = args.flag_value.eq("true");
        let (treated, treated_c) = (format!("{}", flag_val), format!("{}", !flag_val));

        let input_substitutions = HashMap::from([
            (String::from("stale_flag_name"), String::from(&args.flag_name)),
            (String::from("treated"), String::from(&treated)),
            (String::from("namespace"), String::from(&args.flag_namespace)),
            (String::from("treated_complement"), String::from(&treated_c)),
        ]);

        let treated = format!("{}", args.flag_value.eq("true"));
        #[rustfmt::skip]
        println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {}\n (ii) Value: {} \n (iii) flag_namespace : {}", &args.flag_name.clone(), treated, &args.flag_namespace.clone()).purple());
        Self {
            path_to_code_base: args.path_to_codebase.to_string(),
            language: args.language.to_string(),
            input_substitutions: TagMatches::new(input_substitutions),
            path_to_configurations: args.path_to_configuration.to_string(),
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
    pub replace_node: String,
    pub replace: String,
    pub groups: Option<Vec<String>>,
    pub holes: Option<Vec<String>>,
    pub constraint: Option<Constraint>,
    pub grep_heuristics: Option<Vec<String>>,
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
            replace_node: self.replace_node.clone(),
            replace,
            holes: self.holes.clone(),
            groups: self.groups.clone(),
            constraint: self.constraint.clone(),
            grep_heuristics: self.grep_heuristics.clone(),
        }
    }

    pub fn is_feature_flag_cleanup(&self) -> bool {
        self.groups
            .as_ref()
            .map(|tags| tags.iter().any(|t| t.eq("Feature-flag API cleanup")))
            .unwrap_or(false)
    }

    pub fn instantiate(&self, substitutions: &TagMatches) -> Rule {
        self.try_instantiate(substitutions).unwrap()
    }

    pub fn try_instantiate(&self, substitutions: &TagMatches) -> Result<Rule, String> {
        if let Some(holes) = &self.holes {
            let relevant_substitutions = TagMatches::new(
                holes
                    .iter()
                    .filter_map(|hole| substitutions.get(hole).map(|subs| (hole, subs)))
                    .map(|(a, b)| (a.clone(), b.clone()))
                    .collect(),
            );

            if relevant_substitutions.len() == holes.len() {
                return Ok(self.update(
                    self.query.substitute_tags(&relevant_substitutions),
                    self.replace.substitute_tags(&relevant_substitutions),
                ));
            } else {
                #[rustfmt::skip]
                return Err(format!("Could not instantiate a rule - {:?}. Some Holes {:?} not found in table {:?}", self, self.holes, substitutions));
            }
        }
        return Ok(self.clone());
    }

    pub fn add_grep_heuristics_for_seed_rules(&mut self, substitutions: &TagMatches) {
        let mut gh = vec![];
        for h in self.holes.as_ref().unwrap() {
            if let Some(x) = substitutions.get(h) {
                // We do not want to search for strings that occur only in replace.
                if self.query.contains(x) {
                    gh.push(x.clone());
                }
            }
        }
        self.grep_heuristics = Some(gh.clone());
    }

    pub fn get_query(&self) -> TSQuery {
        self.query.clone()
    }

    pub fn add_group(&mut self, group_name: String) {
        if self.groups.is_none() {
            self.groups = Some(vec![group_name]);
        } else {
            self.groups.as_mut().unwrap().push(group_name);
        }
    }
}
