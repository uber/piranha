/*
Copyright (c) 2019 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

//! This module contains all the structs and implementations required for - (i) handling Piranha's runtime arguments,
//! (ii) reading language specific configurations, and (iii) API specific configurations.
//! This module defines all basic building block structs used by Piranha.

use crate::{
    tree_sitter::TreeSitterHelpers,
    utilities::{read_file, MapOfVec},
};

use colored::Colorize;
use serde_derive::Deserialize;
use std::{collections::HashMap, hash::Hash, path::Path};
use tree_sitter::{Query};

use self::command_line_arguments::PiranhaArguments;

static FEATURE_FLAG_API_GROUP: &str = "Feature-flag API cleanup";
pub static GLOBAL: &str = "Global";
pub static METHOD: &str = "Method";
pub static CLASS: &str = "Class";
pub static PARENT: &str = "Parent";

pub mod command_line_arguments {
    //! This module contains structs and implementations for parsing and managing command line arguments passed to Piranha. 
    use std::collections::HashMap;
    use colored::Colorize;
    use clap::Parser;
    use tree_sitter::Language;

    use crate::tree_sitter::TreeSitterHelpers;    
    
    /// Used for parsing command-line arguments passed to Piranha .
    #[derive(Clone, Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    pub struct Args {
        /// Path to source code folder.
        #[clap(short = 'p', long)]
        pub path_to_codebase: String,
        #[clap(short = 'l', long)]
        pub language: String,
        /// Name of the stale flag
        #[clap(short = 'f', long)]
        pub flag_name: String,
        #[clap(short = 'n', long)]
        pub flag_namespace: String,
        /// is the flag treated?
        #[clap(short = 't', long)]
        pub flag_value: bool,
        /// Folder containing the required configuration files
        #[clap(short = 'c', long)]
        pub path_to_configuration: String,
    }

    #[derive(Clone)]
    /// Captures command-line arguments for Piranha.
    pub struct PiranhaArguments {
        /// Path to source code folder.
        pub path_to_code_base: String,
        // Input arguments provided to Piranha, mapped to tag names -
        // @stale_flag_name, @namespace, @treated, @treated_complement
        // These substitutions instantiate the initial set of feature flag rules.
        pub input_substitutions: HashMap<String, String>,
        /// Folder containing the API specific rules
        pub path_to_configurations: String,
        // Language name passed in the command line arguments
        pub language_name: String,
        /// Tree-sitter language model
        pub language: Language,
        // File extension for language 
        pub extension: String,
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
                input_substitutions,
                path_to_configurations: args.path_to_configuration.to_string(),
                language_name:  args.language.to_string(),
                language: args.language.to_string().get_language(),
                extension: args.language.get_extension().to_string(),
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    /// Name of the rule. (It is unique)
    pub name: String,
    /// Tree-sitter query as string
    query: String,
    /// The tag corresponding to the node to be replaced
    pub replace_node: String,
    /// Replacement pattern
    pub replace: String,
    /// Group(s) to which the rule belongs
    pub groups: Option<Vec<String>>,
    /// Holes that need to be filled, in order to instantiate a rule
    pub holes: Option<Vec<String>>,
    /// Additional constraints for matching the rule
    pub constraint: Option<Constraint>,
    /// Heuristics for identifying potential files containing occurrence of the rule.
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
    /// Scope in which the constraint query has to be applied
    pub matcher: String,
    /// All, None or Any
    pub predicate: Pred, // All, any , none
    /// The Tree-sitter queries that need to be applied in the matcher scope
    pub queries: Vec<String>,
}

impl Rule {
    /// Create a new query from `self` with the input `query` and `replace`
    pub fn update(&self, query: String, replace: String) -> Self {
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

    // Dummy rules are helper rules that make it easier to define the rule graph.
    pub fn is_dummy_rule(&self) -> bool {
        return self.query.is_empty() && self.replace.is_empty();
    }

    /// Instantiate `self` with substitutions or panic.
    pub fn instantiate(&self, substitutions: &HashMap<String, String>) -> Rule {
        if let Ok(r) = self.try_instantiate(substitutions) {
            return r;
        }
        panic!(
            "{}",
            format!(
                "Could not instantiate the rule {:?} with substitutions {:?}",
                self, substitutions
            )
            .red()
        );
    }

    /// Groups the rules based on the field `rule.groups`
    /// Note: a rule can belong to more than one group.
    pub fn get_grouped_rules(
        rules: &Vec<Rule>,
    ) -> (HashMap<String, Rule>, HashMap<String, Vec<String>>) {
        let mut rules_by_name = HashMap::new();
        let mut rules_by_group = HashMap::new();
        for rule in rules {
            rules_by_name.insert(rule.name.to_string(), rule.clone());
            if let Some(groups) = &rule.groups {
                for tag in groups {
                    rules_by_group.collect(tag.to_string(), rule.name.to_string());
                }
            }
        }
        (rules_by_name, rules_by_group)
    }

    /// Tries to instantiate the rule (`self`) based on the substitutions.
    /// Note this could fail if the `substitutions` does'nt contain mappings for each hole.
    pub fn try_instantiate(&self, substitutions: &HashMap<String, String>) -> Result<Rule, String> {
        if let Some(holes) = &self.holes {
            let relevant_substitutions: HashMap<String, String> = holes
                .iter()
                .filter_map(|hole| substitutions.get(hole).map(|subs| (hole, subs)))
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect();

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
    pub fn add_grep_heuristics_for_global_rules(
        &mut self,
        substitutions: &HashMap<String, String>,
    ) {
        let mut gh = vec![];
        for h in self.holes.as_ref().unwrap() {
            if let Some(x) = substitutions.get(h) {
                gh.push(x.clone());
            }
        }
        self.grep_heuristics = Some(gh.clone());
    }

    pub fn get_query(&self) -> String {
        self.query.clone()
    }

    /// Adds the rule to a new group - "Feature-flag API cleanup"
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
    pub to: Vec<String>,
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

/// This maintains the state for Piranha.
pub struct RuleStore {
    // A graph that captures the flow amongst the rules
    pub rule_graph: ParameterizedRuleGraph,
    // Caches the compiled tree-sitter queries.
    rule_query_cache: HashMap<String, Query>,
    // All the input rules stored by name
    pub rules_by_name: HashMap<String, Rule>,
    // Current global rules to be applied.
    pub global_rules: Vec<Rule>,
    // Scope generators.
    pub scopes: Vec<ScopeGenerator>,
    // Command line arguments passed to piranha
    pub piranha_args: PiranhaArguments
}

impl RuleStore {
    pub fn new(args: &PiranhaArguments) -> RuleStore {
        let (rule_graph, rules_by_name, scopes) = read_rule_graph_from_config(&args);
        let mut rule_store = RuleStore {
            rule_graph,
            rule_query_cache: HashMap::new(),
            rules_by_name,
            global_rules: vec![],
            scopes,
            piranha_args: args.clone()
        };

        for (_, rule) in rule_store.rules_by_name.clone() {
            rule_store.add_global_rule(&rule, &args.input_substitutions);
        }
        return rule_store;
    }

    pub fn get_global_rules(&self) -> Vec<Rule> {
        self.global_rules.clone()
    }

    /// Add a new global rule, along with grep heuristics.
    pub fn add_global_rule(&mut self, rule: &Rule, tag_captures: &HashMap<String, String>) {
        if !rule.is_feature_flag_cleanup() {
            return;
        }
        if let Ok(mut r) = rule.try_instantiate(&tag_captures) {
            r.add_grep_heuristics_for_global_rules(&tag_captures);
            println!(
                "{}",
                format!("Added Global Rule : {:?} - {}", r.name, r.get_query()).bright_blue()
            );
            self.global_rules.push(r);
        }
        // let mut new_seed_rule = r.instantiate(&tag_captures);
        // new_seed_rule.add_grep_heuristics_for_global_rules(tag_captures);
    }

    /// Get the compiled query for the `query_str` from the cache
    /// else compile it, add it to the cache and return it.
    pub fn get_query(&mut self, query_str: &String) -> &Query {
        self.rule_query_cache
            .entry(query_str.clone())
            .or_insert_with(|| query_str.create_query(self.piranha_args.language))
    }

    /// Get the next rules to be applied grouped by the scope in which they should be performed.
    pub fn get_next(
        &self,
        rule: &Rule,
        tag_matches: &HashMap<String, String>,
    ) -> HashMap<String, Vec<Rule>> {
        let rule_name = &rule.name;
        let mut next_rules: HashMap<String, Vec<Rule>> = HashMap::new();

        for (scope, to_rule) in self.rule_graph.get_neighbors(rule_name) {
            let next_rule = &self.rules_by_name[&to_rule];
            if next_rule.is_dummy_rule() {
                for (next_to_next_rules_scope, next_to_next_rules) in
                    self.get_next(next_rule, tag_matches)
                {
                    for nnr in next_to_next_rules {
                        next_rules.collect(
                            String::from(&next_to_next_rules_scope),
                            nnr.instantiate(&tag_matches),
                        )
                    }
                }
            } else {
                next_rules.collect(String::from(&scope), next_rule.instantiate(&tag_matches));
            }
        }
        for scope in [PARENT, METHOD, CLASS, GLOBAL] {
            next_rules.entry(scope.to_string()).or_default();
        }
        next_rules
    }
}

/// Captures the relationship between the rules as a graph (adjacency list)
pub struct ParameterizedRuleGraph(HashMap<String, Vec<(String, String)>>);

impl ParameterizedRuleGraph {
    // Constructs a graph of rules based on the input `edges` that represent the relationship between two rules or groups of rules.
    fn new(edges: Vec<Edge>, all_rules: Vec<Rule>) -> Self {
        let (rules_by_name, rules_by_group) = Rule::get_grouped_rules(&all_rules);

        // A closure that gets the rules corresponding to the given rule name or group name.
        let get_rules_for_tag_or_name = |val: &String| {
            println!("{}", val);
            rules_by_name
                .get(val)
                .map(|v| vec![v.name.clone()])
                .unwrap_or_else(|| rules_by_group[val].clone())
        };

        let mut graph = HashMap::new();
        // Add the edge(s) to the graph. Multiple edges will be added
        // when either edge endpoint is a group name.
        for edge in edges {
            for from_rule in get_rules_for_tag_or_name(&edge.from) {
                for to_edge in &edge.to {
                    for t in get_rules_for_tag_or_name(&to_edge) {
                        graph.collect(from_rule.clone(), (String::from(&edge.scope), t.clone()));
                    }
                }
            }
        }
        ParameterizedRuleGraph(graph)
    }

    /// Get all the outgoing edges for `rule_name`
    pub fn get_neighbors(&self, rule_name: &String) -> Vec<(String, String)> {
        self.0
            .get(rule_name)
            .map(|x| x.clone())
            .unwrap_or_else(|| vec![])
    }
}

/// Read the language specific cleanup rules.
pub fn get_cleanup_rules(language: &String) -> (Rules, Edges, Vec<ScopeGenerator>) {
    match language.as_str() {
        "Java" => (
            toml::from_str::<Rules>(include_str!("cleanup_rules/java/rules.toml")).unwrap(),
            toml::from_str::<Edges>(include_str!("cleanup_rules/java/edges.toml")).unwrap(),
            toml::from_str::<ScopeConfig>(include_str!("cleanup_rules/java/scope_config.toml"))
                .map(|x| x.scopes)
                .unwrap(),
        ),
        _ => panic!(),
    }
}

/// Reads the input configurations and creates a rule graph.
pub fn read_rule_graph_from_config(
    args: &PiranhaArguments,
) -> (
    ParameterizedRuleGraph,
    HashMap<String, Rule>,
    Vec<ScopeGenerator>,
) {
    let path_to_config = Path::new(args.path_to_configurations.as_str());

    // Read the language specific cleanup rules and edges
    let (language_rules, language_edges, scopes) = get_cleanup_rules(&args.language_name);

    // Read the API specific cleanup rules and edges
    let (mut input_rules, input_edges) = (
        toml::from_str::<Rules>(read_file(&path_to_config.join("rules.toml")).as_str()).unwrap(),
        toml::from_str::<Edges>(read_file(&path_to_config.join("edges.toml")).as_str()).unwrap(),
    );

    // Label the input-rules as `Feature-flag API cleanup`
    for r in input_rules.rules.iter_mut() {
        r.add_to_feature_flag_api_group();
    }

    let all_rules = [language_rules.rules, input_rules.rules].concat();
    let all_edges = [language_edges.edges, input_edges.edges].concat();

    let (rules_by_name, _) = Rule::get_grouped_rules(&all_rules);

    let graph = ParameterizedRuleGraph::new(all_edges, all_rules);

    (graph, rules_by_name, scopes)
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
    pub matcher: String,
    pub generator: String,
}
