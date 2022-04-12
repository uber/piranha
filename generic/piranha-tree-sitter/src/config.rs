use crate::tree_sitter::{TSQuery, TagMatches, TreeSitterHelpers};
use clap::Parser;
use colored::Colorize;
use serde_derive::Deserialize;
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Parser, Debug)]
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
    pub flag_value: bool,
    #[clap(short, long)]
    pub path_to_configuration: String,
}

static FEATURE_FLAG_API_GROUP: &str = "Feature-flag API cleanup";
pub struct PiranhaArguments {
    pub path_to_code_base: String,
    pub language: String,
    // These are the input arguments provided to Piranha, mapped to the appropriate tag names.
    // These are used as the seed substitutions to instantiate the feature flag mappings.
    pub input_substitutions: TagMatches,
    pub path_to_configurations: String,
}

impl PiranhaArguments {
    pub fn new(args: Args) -> Self {
        
        let input_substitutions = HashMap::from([
            ("stale_flag_name", &args.flag_name),
            ("treated", &format!("{}", args.flag_value)),
            ("namespace", &args.flag_namespace),
            ("treated_complement", &format!("{}", !args.flag_value)),
        ])
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        #[rustfmt::skip]
        println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {}\n (ii) Value: {} \n (iii) flag_namespace : {}", &args.flag_name.clone(), &format!("{}", args.flag_value), &args.flag_namespace.clone()).purple());
        
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
    pub scopes: Vec<ScopeGenerator>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopeGenerator {
    pub name: String,
    pub rules: Vec<ScopeQueryGenerator>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScopeQueryGenerator {
    pub matcher: TSQuery,
    pub generator: TSQuery,
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
    /// Create a new query from `self` with the input `query` and `replace`
    pub fn update(&self, query: TSQuery, replace: String) -> Self {
        Rule {
            name: self.name.to_string(),
            query,
            replace_node: self.replace_node.to_string(),
            replace,
            holes: self.holes.clone(),
            groups: self.groups.clone(),
            constraint: self.constraint.clone(),
            grep_heuristics: self.grep_heuristics.clone(),
        }
    }

    pub fn is_feature_flag_cleanup(&self) -> bool {
        self.groups.as_ref().map_or(false, |tags| {
            tags.iter().any(|t| t.eq(FEATURE_FLAG_API_GROUP))
        })
    }

    /// Instantiate `self` with substitutions or panic.
    pub fn instantiate(&self, substitutions: &TagMatches) -> Rule {
        if let Ok(r) = self.try_instantiate(substitutions){
            return r;
        }
        panic!("{}", format!("Could not instantiate the rule {:?} with substitutions {:?}", self, substitutions).red());
    }

    /// Tries to instantiate the `self` with the tag matches. 
    /// Note this could fail in case when tag matches dont contain mappings for all the holes.
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

    /// Records the string that should be grepped in order to find files that 
    /// potentially could match this global rule.
    pub fn add_grep_heuristics_for_global_rules(&mut self, substitutions: &TagMatches) {
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

    /// Adds a new group label to the rule. 
    pub fn add_to_feature_flag_api_group(&mut self) {
        let group_name: String = FEATURE_FLAG_API_GROUP.to_string();
        if self.groups.is_none() {
            self.groups = Some(vec![group_name]);
        } else {
            self.groups.as_mut().unwrap().push(group_name);
        }
    }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub scope: String,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Edges {
    pub edges: Vec<Edge>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rules {
    pub rules: Vec<Rule>,
}
