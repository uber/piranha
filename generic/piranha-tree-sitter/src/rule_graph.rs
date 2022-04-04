use crate::{
    config::{PiranhaArguments, Rule, Scope, ScopeConfig},
    tree_sitter::TreeSitterHelpers,
    utilities::{read_file, MapOfVec},
};
use colored::Colorize;
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
    pub rule_graph: HashMap<String, Vec<(String, String)>>,
    rule_query_cache: HashMap<String, Query>,
    pub language: Language,
    pub rules_by_name: HashMap<String, Rule>,
    pub seed_rules: Vec<Rule>,
    pub scopes: Vec<Scope>,
}

impl RuleStore {
    pub fn new(args: &PiranhaArguments) -> RuleStore {
        let (rule_graph, rules_by_name, scopes) = create_rule_graph(&args);

        let seed_rules:Vec<Rule> = rules_by_name.iter()
            .filter(|(_,rule)| rule.is_feature_flag_cleanup())
            .map(|(_, r)| r.instantiate(&args.input_substiution))
            .collect();

        RuleStore {
            rule_graph,
            rule_query_cache: HashMap::new(),
            language: args.language.get_language(),
            rules_by_name,
            seed_rules,
            scopes,
        }
    }

    pub fn get_seed_rules(&self) -> Vec<Rule> {
        self.seed_rules.clone()
    }

    pub fn add_seed_rule(&mut self, r: Rule, tag_captures_previous_edit: &HashMap<String, String>) {
        let new_seed_rule = r.instantiate(&tag_captures_previous_edit);
        println!("{}", format!("Added Seed Rule : {:?}", new_seed_rule).red());
        self.seed_rules.push(new_seed_rule);
    }

    pub fn get_query(&mut self, query_str: &String) -> &Query {
        self.rule_query_cache
            .entry(query_str.clone())
            .or_insert_with(|| query_str.create_query(self.language))
    }

    fn get_next_rules(
        &self,
        rule: Rule,
        tag_matches: &HashMap<String, String>,
    ) -> HashMap<String, Vec<Rule>> {
        let rule_name = &rule.name;
        let mut next_rules: HashMap<String, Vec<Rule>> = HashMap::new();
        if let Some(from_rule) = self.rule_graph.get(rule_name) {
            for (scope, to_rule) in from_rule {
                next_rules.collect_as_counter(
                    String::from(scope),
                    self.rules_by_name[to_rule].instantiate(&tag_matches),
                );
            }
        }
        next_rules
    }

    pub fn get_next(
        &self,
        rule: Rule,
        tag_matches: &HashMap<String, String>,
    ) -> (Vec<Rule>, Vec<Rule>, Vec<Rule>, Vec<Rule>) {
        let next_rules = self.get_next_rules(rule, tag_matches);

        let get = |s: &str| {
            next_rules
                .get(s)
                .map(|x| x.clone())
                .unwrap_or_else(|| vec![])
        };

        (get("Parent"), get("Method"), get("Class"), get("Global"))
    }
}

type ParameterizedRuleGraph = HashMap<String, Vec<(String, String)>>;

pub fn create_rule_graph(
    args: &PiranhaArguments,
) -> (ParameterizedRuleGraph, HashMap<String, Rule>, Vec<Scope>) {
    let path_to_config = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("configurations");

    // Read the configuration files.
    let (path_to_all_rules_toml, path_to_edges, path_to_scope_config) = match args.language.as_str()
    {
        "Java" => (
            path_to_config.join("all_rules.toml"),
            path_to_config.join("edges.toml"),
            path_to_config.join("java_scope_config.toml"),
        ),
        _ => panic!(),
    };

    let all_rules_content = read_file(&path_to_all_rules_toml);
    let edges_content = read_file(&path_to_edges);
    let scope_config_content = read_file(&path_to_scope_config);

    // Group rules by tag
    // Collect groups by name
    let all_rules: Rules = toml::from_str(all_rules_content.as_str()).unwrap();

    let mut rules_by_name = HashMap::new();
    let mut rules_by_tag = HashMap::new();
    for rule in all_rules.rules {
        rules_by_name.insert(rule.name.clone(), rule.clone());
        if let Some(tags) = &rule.tag {
            for tag in tags {
                rules_by_tag.collect_as_counter(tag.clone(), rule.name.clone());
            }
        }
    }

    // Construct Graph

    let get_rules_for_tag_or_name = |val: &String| {
        rules_by_name
            .get(val)
            .map(|v| vec![v.name.clone()])
            .unwrap_or_else(|| rules_by_tag[val].clone())
    };

    let mut graph: ParameterizedRuleGraph = HashMap::new();
    let edges: Edges = toml::from_str(edges_content.as_str()).unwrap();
    for edge in edges.edges {
        for f in get_rules_for_tag_or_name(&edge.from) {
            for t in get_rules_for_tag_or_name(&edge.to) {
                graph.collect_as_counter(f.clone(), (String::from(&edge.scope), t.clone()));
            }
        }
    }

    let scopes = toml::from_str(&scope_config_content)
        .map(|x: ScopeConfig| x.scopes)
        .unwrap();
    (graph, rules_by_name, scopes)
}
