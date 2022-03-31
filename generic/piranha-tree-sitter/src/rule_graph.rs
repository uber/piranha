use crate::{
    config::{Rule, Scope, ScopeConfig},
    utilities::{read_file, substitute_in_str, MapOfVec},
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

pub struct GraphRuleStore {
    pub p_rule_graph: HashMap<String, Vec<(String, String)>>,
    rule_query_cache: HashMap<String, Query>,
    pub language: Language,
    pub p_rules_by_name: HashMap<String, Rule>,
    pub seed_rules: Vec<Rule>,
    pub seed_substitutions: HashMap<String, String>,
    pub scopes: Vec<Scope>,
}

pub fn map_key(s: &String) -> String {
    format!("[@{}]", s)
}

pub fn map_identity(x: &String) -> String {
    String::from(x)
}

impl GraphRuleStore {
    pub fn new(
        input_language: &str,
        language: Language,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str,
    ) -> GraphRuleStore {
        let (p_rule_graph, p_rules_by_name, scopes) =
            create_rule_graph(input_language, flag_name, flag_namespace, flag_value);

        let rule_query_cache = HashMap::new();
        let flag_val = flag_value.eq("true");
        let (treated, treated_c) = (format!("{}", flag_val), format!("{}", !flag_val));

        let seed_substitutions = HashMap::from([
            (String::from("[stale_flag_name]"), String::from(flag_name)),
            (String::from("[treated]"), String::from(&treated)),
            (String::from("[namespace]"), String::from(flag_namespace)),
            (
                String::from("[treated_complement]"),
                String::from(&treated_c),
            ),
        ]);

        // Seed rules = All parameterized rules with `> 0` holes and can be instantiated with above
        // substitutions
        let mut seed_rules = vec![];
        for (_, rule) in &p_rules_by_name {
            if let Some(_) = &rule.holes {
                if let Some(r) = rule.instantiate(&seed_substitutions, &map_identity) {
                    print!("{:?}", r);
                    seed_rules.push(r);
                }
            }
        }
        println!("{}", format!("{}", seed_rules.len()).red());

        GraphRuleStore {
            p_rule_graph,
            rule_query_cache,
            language,
            p_rules_by_name,
            seed_rules,
            seed_substitutions,
            scopes,
        }
    }

    pub fn get_seed_rules(&self) -> Vec<Rule> {
        self.seed_rules.clone()
    }

    pub fn add_seed_rule(&mut self, r: Rule, tag_captures_previous_edit: &HashMap<String, String>) {
        // let new_seed_rule = r.instantiate(&tag_captures_previous_edit, &map_identity);
        if let Some(new_seed_rule) = r.instantiate(&tag_captures_previous_edit, &map_identity) {
            println!("{}", format!("Added Seed Rule : {:?}", new_seed_rule).red());
            self.seed_rules.push(new_seed_rule);
        }
    }

    pub fn get_query(&mut self, query_str: &String) -> &Query {
        if !self.rule_query_cache.contains_key(query_str) {
            let q = Query::new(self.language, query_str);
            if q.is_err() {
                panic!("Could not parse the query : {}", query_str);
            }
            let _ = self
                .rule_query_cache
                .insert(String::from(query_str), q.unwrap());
        }
        return self.rule_query_cache.get(query_str).unwrap();
    }

    fn get_next_rules(
        &self,
        rule: Rule,
        tag_matches: &HashMap<String, String>,
    ) -> HashMap<String, Vec<Rule>> {
        let rule_name = &rule.name;
        let mut next_rules: HashMap<String, Vec<Rule>> = HashMap::new();
        
        
        if let Some(from_rule) = self.p_rule_graph.get(rule_name){
            for (scope, to_rule) in from_rule {
                if let Some(transformed_rule) =
                    self.p_rules_by_name[to_rule].instantiate(&tag_matches, &map_key)
                {
                    next_rules.collect(String::from(scope), transformed_rule);
                } else {
                    panic!("Could not transform {:?} \n \n {:?}", from_rule, tag_matches);
                }
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
        
        let get = |s: &str|{
            if next_rules.contains_key(s) {next_rules[s].clone()} else {vec![]}
        };

        (get("Parent"), get("Method"), get("Class"), get("Global"))
    }
}

type ParameterizedRuleGraph = HashMap<String, Vec<(String, String)>>;

pub fn create_rule_graph(
    language: &str,
    flag_name: &str,
    flag_namespace: &str,
    flag_value: &str,
) -> (ParameterizedRuleGraph, HashMap<String, Rule>, Vec<Scope>) {
    let path_to_config = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("configurations");

    // Read the configuration files.
    let (path_to_all_rules_toml, path_to_edges, path_to_scope_config) = match language {
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

    #[rustfmt::skip]
    let treated = format!("{}", flag_value.eq("true"));
    println!("{}",  format!("Piranha arguments are :\n (i) flag_name : {flag_name}\n (ii) Value: {treated} \n (iii) flag_namespace : {flag_namespace}").purple());

    // Group rules by tag
    // Collect groups by name
    let all_rules: Rules = toml::from_str(all_rules_content.as_str()).unwrap();
    
    let mut rules_by_name = HashMap::new();
    let mut rules_by_tag = HashMap::new();
    for rule in all_rules.rules {
        rules_by_name.insert(rule.name.clone(), rule.clone());
        if let Some(tags) = &rule.tag {
            for tag in tags {
                rules_by_tag.collect(tag.clone(), rule.name.clone());
            }
        }
    }

    // Construct Graph
    let mut graph: ParameterizedRuleGraph = HashMap::new();
    let edges: Edges = toml::from_str(edges_content.as_str()).unwrap();
    for edge in edges.edges {
        for f in get_rules_for_tag_or_name(&edge.from, &rules_by_name, &rules_by_tag) {
            for t in get_rules_for_tag_or_name(&edge.to, &rules_by_name, &rules_by_tag) {
                graph.collect(f.clone(), (String::from(&edge.scope), t.clone()));
            }
        }
    }

    let scopes = toml::from_str(&scope_config_content).map(|x:ScopeConfig|x.scopes).unwrap();
    (graph, rules_by_name, scopes)
}

fn get_rules_for_tag_or_name(
    val: &String,
    rules_by_name: &HashMap<String, Rule>,
    rules_by_tag: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    if rules_by_name.contains_key(val) {
        vec![String::from(&rules_by_name[val].name)]
    } else {
        rules_by_tag[val].clone()
    }
}
