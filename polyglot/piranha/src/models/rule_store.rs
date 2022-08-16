/*
Copyright (c) 2022 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use std::collections::HashMap;

use colored::Colorize;
use log::info;
use tree_sitter::{Language, Query};

use crate::{
  config::read_config_files,
  models::piranha_arguments::PiranhaArguments,
  models::{
    rule::Rule,
    rule_graph::RuleGraph,
    scopes::{ScopeGenerator, ScopeQueryGenerator},
  },
  utilities::{tree_sitter_utilities::TreeSitterHelpers, MapOfVec},
};

pub(crate) static GLOBAL: &str = "Global";
pub(crate) static PARENT: &str = "Parent";
/// This maintains the state for Piranha.
pub(crate) struct RuleStore {
  // A graph that captures the flow amongst the rules
  rule_graph: RuleGraph,
  // Caches the compiled tree-sitter queries.
  rule_query_cache: HashMap<String, Query>,
  // All the input rules stored by name
  rules_by_name: HashMap<String, Rule>,
  // Current global rules to be applied.
  global_rules: Vec<Rule>,
  // Scope generators.
  scopes: Vec<ScopeGenerator>,
  // Command line arguments passed to piranha
  piranha_args: PiranhaArguments,
}

impl RuleStore {
  pub(crate) fn new(args: &PiranhaArguments) -> RuleStore {
    let (rules, edges, scopes) = read_config_files(args);
    let rule_graph = RuleGraph::new(&edges, &rules);
    let mut rule_store = RuleStore {
      rule_graph,
      rule_query_cache: HashMap::new(),
      rules_by_name: rules.iter().map(|r| (r.name(), r.clone())).collect(),
      global_rules: vec![],
      scopes,
      piranha_args: args.clone(),
    };

    for (_, rule) in rule_store.rules_by_name.clone() {
      if rule.is_feature_flag_cleanup() {
        rule_store.add_to_global_rules(&rule, args.input_substitutions());
      }
    }
    rule_store
  }

  pub(crate) fn global_rules(&self) -> Vec<Rule> {
    self.global_rules.clone()
  }

  pub(crate) fn language(&self) -> Language {
    self.piranha_args.language()
  }

  pub(crate) fn language_name(&self) -> &str {
    self.piranha_args.language_name()
  }

  pub(crate) fn input_substitutions(&self) -> HashMap<String, String> {
    self.piranha_args.input_substitutions().clone()
  }

  /// Add a new global rule, along with grep heuristics (If it doesn't already exist)
  pub(crate) fn add_to_global_rules(
    &mut self, rule: &Rule, tag_captures: &HashMap<String, String>,
  ) {
    if let Ok(mut r) = rule.try_instantiate(tag_captures) {
      if !self.global_rules.iter().any(|r| {
        r.name().eq(&rule.name())
          && r.replace().eq(&rule.replace())
          && r.query().eq(&rule.query())
      }) {
        r.add_grep_heuristics_for_global_rules(tag_captures);
        #[rustfmt::skip]
        info!("{}", format!("Added Global Rule : {:?} - {}", r.name(), r.query()).bright_blue());
        self.global_rules.push(r);
      }
    }
  }

  /// Get the compiled query for the `query_str` from the cache
  /// else compile it, add it to the cache and return it.
  pub(crate) fn query(&mut self, query_str: &String) -> &Query {
    let language = self.language();
    self
      .rule_query_cache
      .entry(query_str.to_string())
      .or_insert_with(|| query_str.create_query(language))
  }

  /// Get the next rules to be applied grouped by the scope in which they should be performed.
  pub(crate) fn get_next(
    &self, rule_name: &String, tag_matches: &HashMap<String, String>,
  ) -> HashMap<String, Vec<Rule>> {
    // let rule_name = rule.name();
    let mut next_rules: HashMap<String, Vec<Rule>> = HashMap::new();
    // Iterate over each entry (Edge) in the adjacency list corresponding to `rule_name`
    for (scope, to_rule) in self.rule_graph.get_neighbors(&rule_name) {
      let to_rule_name = &self.rules_by_name[&to_rule];
      // If the to_rule_name is a dummy rule, skip it and rather return it's next rules.
      if to_rule_name.is_dummy_rule() {
        // Call this method recursively on the dummy node
        for (next_next_rules_scope, next_next_rules) in self.get_next(&to_rule_name.name(), tag_matches) {
          for next_next_rule in next_next_rules {
            // Group the next rules based on the scope
            next_rules.collect(
              String::from(&next_next_rules_scope),
              next_next_rule.instantiate(tag_matches),
            )
          }
        }
      } else {
        // Group the next rules based on the scope
        next_rules.collect(String::from(&scope), to_rule_name.instantiate(tag_matches));
      }
    }
    // Add empty entry, incase no next rule was found for a particular scope
    for scope in [PARENT, GLOBAL] {
      next_rules.entry(scope.to_string()).or_default();
    }
    next_rules
  }

  // For the given scope level, get the ScopeQueryGenerator from the `scope_config.toml` file
  pub(crate) fn get_scope_query_generators(&self, scope_level: &str) -> Vec<ScopeQueryGenerator> {
    self
      .scopes
      .iter()
      .find(|level| level.name().eq(scope_level))
      .map(|scope| scope.rules())
      .unwrap_or_else(Vec::new)
  }
}

#[cfg(test)]
impl RuleStore {
  pub(crate) fn dummy() -> RuleStore {
    RuleStore {
      rule_graph: RuleGraph::dummy(),
      rule_query_cache: HashMap::new(),
      rules_by_name: HashMap::new(),
      global_rules: vec![],
      piranha_args: PiranhaArguments::dummy(),
      scopes: vec![],
    }
  }

  pub(crate) fn dummy_with_scope(scopes: Vec<ScopeGenerator>) -> RuleStore {
    RuleStore {
      rule_graph: RuleGraph::dummy(),
      rule_query_cache: HashMap::new(),
      rules_by_name: HashMap::new(),
      global_rules: vec![],
      piranha_args: PiranhaArguments::dummy(),
      scopes,
    }
  }
}
