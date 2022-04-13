use crate::{
    config::{PiranhaArguments, Rule, ScopeGenerator, ScopeConfig, Rules, Edges},
    tree_sitter::{TSQuery, TagMatches, TreeSitterHelpers},
    utilities::{read_file, MapOfVec},
};
use colored::Colorize;
use std::{collections::HashMap, path::Path};
use tree_sitter::{Language, Query};

pub static GLOBAL: &str = "Global";
pub static METHOD: &str = "Method";
pub static CLASS: &str = "Class";
pub static PARENT: &str = "Parent";

/// This maintains the state for Piranha. 
pub struct RuleStore {
    pub rule_graph: ParameterizedRuleGraph,
    // Caches the compiled tree-sitter queries.
    rule_query_cache: HashMap<TSQuery, Query>,
    pub language: Language,
    // All the input rules stored by name
    pub rules_by_name: HashMap<String, Rule>,
    // Current global rules to be applied.
    pub global_rules: Vec<Rule>,
    // Scope generators.
    pub scopes: Vec<ScopeGenerator>,
}

impl RuleStore {

    pub fn new(args: &PiranhaArguments) -> RuleStore {
        let (rule_graph, rules_by_name, scopes) = read_rule_graph_from_config(&args);

        let mut rule_store = RuleStore {
            rule_graph,
            rule_query_cache: HashMap::new(),
            language: args.language.get_language(),
            rules_by_name,
            global_rules: vec![],
            scopes,
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
    pub fn add_global_rule(&mut self, rule: &Rule, tag_captures: &TagMatches) {
        if !rule.is_feature_flag_cleanup(){
            return;
        }
        if let Ok(mut r) = rule.try_instantiate(&tag_captures) {
            r.add_grep_heuristics_for_global_rules(&tag_captures);
            // seed_rules.push(r.clone());
            println!("{}", format!("Added Global Rule : {:?} - {}", r.name, r.get_query().pretty()).bright_blue());
            self.global_rules.push(r);
        }
        // let mut new_seed_rule = r.instantiate(&tag_captures);
        // new_seed_rule.add_grep_heuristics_for_global_rules(tag_captures);
    }

    /// Get the compiled query for the `query_str` from the cache 
    /// else compile it, add it to the cache and return it.
    pub fn get_query(&mut self, query_str: &TSQuery) -> &Query {
        self.rule_query_cache
            .entry(query_str.clone())
            .or_insert_with(|| query_str.create_query(self.language))
    }

    /// Get the next rules to be applied grouped by the scope in which they should be performed. 
    pub fn get_next(&self, rule: Rule, tag_matches: &TagMatches) -> HashMap<String, Vec<Rule>> {
        let rule_name = &rule.name;
        let mut next_rules: HashMap<String, Vec<Rule>> = HashMap::new();
        for (scope, to_rule) in self.rule_graph.get_nbrs(rule_name) {
            next_rules.collect(
                String::from(scope),
                self.rules_by_name[&to_rule].instantiate(&tag_matches),
            );
        }
        for scope in [PARENT, METHOD, CLASS, GLOBAL] {
            next_rules.entry(scope.to_string()).or_default();
        }
        next_rules
    }
}

/// a new_type for the rule graph.
pub struct ParameterizedRuleGraph(HashMap<String, Vec<(String, String)>>);

impl ParameterizedRuleGraph {

    // Constructs a graph of rules based on the input `edges`
    fn new(edges: Edges, all_rules: Rules) -> Self {
        let mut rules_by_name = HashMap::new();
        let mut rules_by_group = HashMap::new();

        // Collect all the rules based on the group.
        for rule in all_rules.rules {
            rules_by_name.insert(rule.name.clone(), rule.clone());
            if let Some(groups) = &rule.groups {
                for tag in groups {
                    rules_by_group.collect(tag.clone(), rule.name.clone());
                }
            }
        }

        // Get the rules corresponding to the given rule name or group name.
        let get_rules_for_tag_or_name = |val: &String| {
            rules_by_name
                .get(val)
                .map(|v| vec![v.name.clone()])
                .unwrap_or_else(|| rules_by_group[val].clone())
        };

        let mut graph = HashMap::new();
        // Add the edge(s) to the graph. Multiple edges will be added 
        // when the either edge endpoint is a group name.
        for edge in edges.edges {
            for f in get_rules_for_tag_or_name(&edge.from) {
                for t in get_rules_for_tag_or_name(&edge.to) {
                    graph.collect(f.clone(), (String::from(&edge.scope), t.clone()));
                }
            }
        }
        ParameterizedRuleGraph(graph)
    }

    /// Get all the outgoing edges from `rule_name`
    pub fn get_nbrs(&self, rule_name: &String) -> Vec<(String, String)> {
        self.0
            .get(rule_name)
            .map(|x| x.clone())
            .unwrap_or_else(|| vec![])
    }
}

/// Reads the input configurations and creates a rule graph.
pub fn read_rule_graph_from_config(
    args: &PiranhaArguments,
) -> (ParameterizedRuleGraph, HashMap<String, Rule>, Vec<ScopeGenerator>) {
    let path_to_config = Path::new(args.path_to_configurations.as_str());

    // Read the rules 
    let (language_rules, language_edges, scopes) = match args.language.as_str() {
        "Java" => (
            toml::from_str::<Rules>(include_str!("config/java/java_rules.toml")).unwrap(),
            toml::from_str::<Edges>(include_str!("config/java/java_edges.toml")).unwrap(),
            toml::from_str::<ScopeConfig>(include_str!("config/java/java_scope_config.toml"),)
            .map(|x| x.scopes)
            .unwrap(),
        ),
        _ => panic!(),
    };

    // Read the edges 
    let (mut input_rules, input_edges) = (
        toml::from_str::<Rules>(read_file(&path_to_config.join("input_rules.toml")).as_str())
            .unwrap(),
        toml::from_str::<Edges>(read_file(&path_to_config.join("input_edges.toml")).as_str())
            .unwrap(),
    );

    // Label the input-rules as `Feature-flag API cleanup`
    for r in input_rules.rules.iter_mut() {
        r.add_to_feature_flag_api_group();
    }

    let all_rules = Rules {
        rules: [language_rules.clone().rules, input_rules.clone().rules].concat(),
    };
    let edges = Edges {
        edges: [language_edges.edges.clone(), input_edges.edges.clone()].concat(),
    };

    let rules_by_name = all_rules
        .rules
        .iter()
        .map(|r| (r.name.clone(), r.clone()))
        .collect();

    let graph = ParameterizedRuleGraph::new(edges, all_rules);

    (graph, rules_by_name, scopes)
}
