use crate::{
    config::{PiranhaArguments, Rule, Scope, ScopeConfig},
    tree_sitter::{TSQuery, TagMatches, TreeSitterHelpers},
    utilities::{read_file, MapOfVec},
};
use colored::Colorize;
use itertools::Itertools;
use regex::Regex;
use serde_derive::Deserialize;
use std::{collections::HashMap, path::Path};
use tree_sitter::{Language, Query};

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
struct Edge {
    pub from: String,
    pub to: String,
    pub scope: String,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
struct Edges {
    edges: Vec<Edge>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
struct Rules {
    pub rules: Vec<Rule>,
}

pub struct RuleStore {
    pub rule_graph: ParameterizedRuleGraph,
    rule_query_cache: HashMap<TSQuery, Query>,
    pub language: Language,
    pub rules_by_name: HashMap<String, Rule>,
    pub seed_rules: Vec<Rule>,
    pub scopes: Vec<Scope>,
}

impl RuleStore {
    pub fn new(args: &PiranhaArguments) -> RuleStore {
        let (rule_graph, rules_by_name, scopes) = create_rule_graph(&args);

        let mut seed_rules: Vec<Rule> = vec![];
        for (_, rule) in &rules_by_name {
            if rule.is_feature_flag_cleanup() {
                if let Ok(mut r) = rule.try_instantiate(&args.input_substitutions){
                    r.add_grep_heuristics_for_seed_rules(&args.input_substitutions);
                    seed_rules.push(r.clone());
                }
            }
        }

        RuleStore {
            rule_graph,
            rule_query_cache: HashMap::new(),
            language: args.language.get_language(),
            rules_by_name,
            seed_rules,
            scopes,
        }
    }

    

    pub fn get_grep_heuristics(&self) -> Regex {
        let reg_x = self
            .get_seed_rules()
            .iter()
            .flat_map(|r| r.grep_heuristics.as_ref().unwrap().iter())
            //Remove duplicates
            .sorted()
            .dedup()
            .join("|");
        Regex::new(reg_x.as_str()).unwrap()
    }

    pub fn get_seed_rules(&self) -> Vec<Rule> {
        self.seed_rules.clone()
    }

    pub fn add_seed_rule(&mut self, r: &Rule, tag_captures: &TagMatches) {
        let mut new_seed_rule = r.instantiate(&tag_captures);
        new_seed_rule.add_grep_heuristics_for_seed_rules(tag_captures);
        println!("{}", format!("Added Seed Rule : {:?}", new_seed_rule).red());
        self.seed_rules.push(new_seed_rule);
    }

    pub fn get_query(&mut self, query_str: &TSQuery) -> &Query {
        self.rule_query_cache
            .entry(query_str.clone())
            .or_insert_with(|| query_str.create_query(self.language))
    }

    fn get_next_rules(&self, rule: Rule, tag_matches: &TagMatches) -> HashMap<String, Vec<Rule>> {
        let rule_name = &rule.name;
        let mut next_rules: HashMap<String, Vec<Rule>> = HashMap::new();
        // if let Some(nbrs) = self.rule_graph.get_nbrs(rule_name) {
        for (scope, to_rule) in self.rule_graph.get_nbrs(rule_name) {
            next_rules.collect_as_counter(
                String::from(scope),
                self.rules_by_name[&to_rule].instantiate(&tag_matches),
            );
        }
        // }
        next_rules
    }

    pub fn get_next(&self, rule: Rule, tag_matches: &TagMatches) -> HashMap<String, Vec<Rule>> {
        let next_rules = self.get_next_rules(rule, tag_matches);

        let get = |s: &str| {
            (
                String::from(s),
                next_rules
                    .get(s)
                    .map(|x| x.clone())
                    .unwrap_or_else(|| vec![]),
            )
        };

        HashMap::from([get("Parent"), get("Method"), get("Class"), get("Global")])
    }
}

pub struct ParameterizedRuleGraph(HashMap<String, Vec<(String, String)>>);

impl ParameterizedRuleGraph {
    fn new(edges: Edges, all_rules: Rules) -> Self {
        let mut rules_by_name = HashMap::new();
        let mut rules_by_tag = HashMap::new();

        for rule in all_rules.rules {
            rules_by_name.insert(rule.name.clone(), rule.clone());
            if let Some(tags) = &rule.groups {
                for tag in tags {
                    rules_by_tag.collect_as_counter(tag.clone(), rule.name.clone());
                }
            }
        }

        let get_rules_for_tag_or_name = |val: &String| {
            rules_by_name
                .get(val)
                .map(|v| vec![v.name.clone()])
                .unwrap_or_else(|| rules_by_tag[val].clone())
        };

        let mut graph = HashMap::new();
        for edge in edges.edges {
            for f in get_rules_for_tag_or_name(&edge.from) {
                for t in get_rules_for_tag_or_name(&edge.to) {
                    graph.collect_as_counter(f.clone(), (String::from(&edge.scope), t.clone()));
                }
            }
        }
        ParameterizedRuleGraph(graph)
    }

    pub fn get_nbrs(&self, rule_name: &String) -> Vec<(String, String)> {
        self.0
            .get(rule_name)
            .map(|x| x.clone())
            .unwrap_or_else(|| vec![])
    }
}

pub fn create_rule_graph(
    args: &PiranhaArguments,
) -> (ParameterizedRuleGraph, HashMap<String, Rule>, Vec<Scope>) {
    let path_to_config = Path::new(args.path_to_configurations.as_str());
        
    // Read the configuration files.
    let (language_rules, language_edges, scopes) = match args.language.as_str() {
        "Java" => (
            toml::from_str::<Rules>(read_file(&path_to_config.join("java_rules.toml")).as_str()).unwrap(),
            toml::from_str::<Edges>(read_file(&path_to_config.join("java_edges.toml")).as_str()).unwrap(),
            toml::from_str::<ScopeConfig>(read_file(&path_to_config.join("java_scope_config.toml")).as_str()).map(|x| x.scopes).unwrap(),
        ),
        _ => panic!(),
    };

    let(mut input_rules, input_edges) = (
        toml::from_str::<Rules>(read_file(&path_to_config.join("input_rules.toml")).as_str()).unwrap(),
        toml::from_str::<Edges>(read_file(&path_to_config.join("input_edges.toml")).as_str()).unwrap(),
    );

    for r in input_rules.rules.iter_mut(){
        r.add_group(String::from("Feature-flag API cleanup"));
    }

    let all_rules = Rules {rules: [language_rules.clone().rules, input_rules.clone().rules].concat()};
    let edges = Edges { edges: [language_edges.edges.clone(), input_edges.edges.clone()].concat() };
    // all_rules.extend(in)
    
    let rules_by_name = all_rules
        .rules
        .iter()
        .map(|r| (r.name.clone(), r.clone()))
        .collect();
    let graph = ParameterizedRuleGraph::new(edges, all_rules);
    // let scopes = toml::from_str(&scope_config_content)
    //     .map(|x: ScopeConfig| x.scopes)
    //     .unwrap();
    (graph, rules_by_name, scopes)
}
