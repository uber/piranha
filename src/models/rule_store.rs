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

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use colored::Colorize;
use getset::Getters;
use itertools::Itertools;
use jwalk::WalkDir;
use log::{debug, info, trace};
use regex::Regex;
use tree_sitter::Query;

use crate::{
  models::piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
  models::{rule::Rule, rule_graph::RuleGraph, scopes::ScopeQueryGenerator},
  utilities::{read_file, read_toml, MapOfVec},
};

use super::{
  outgoing_edges::{Edges, OutgoingEdges},
  rule::{InstantiatedRule, Rules},
};

pub(crate) static GLOBAL: &str = "Global";
pub(crate) static PARENT: &str = "Parent";
/// This maintains the state for Piranha.
#[derive(Debug, Getters)]
pub(crate) struct RuleStore {
  // A graph that captures the flow amongst the rules
  #[get = "pub"]
  rule_graph: RuleGraph,
  // Caches the compiled tree-sitter queries.
  rule_query_cache: HashMap<String, Query>,
  // Current global rules to be applied.
  #[get = "pub"]
  global_rules: Vec<InstantiatedRule>,
  // Command line arguments passed to piranha
  #[get = "pub"]
  piranha_args: PiranhaArguments,
}

impl From<PiranhaArguments> for RuleStore {
  fn from(piranha_args: PiranhaArguments) -> Self {
    RuleStore {
      piranha_args,
      ..Default::default()
    }
  }
}

impl RuleStore {
  pub(crate) fn new(args: &PiranhaArguments) -> RuleStore {
    let (rules, edges) = read_config_files(args);
    let rule_graph = RuleGraph::new(&edges, &rules);
    let mut rule_store = RuleStore {
      rule_graph,
      piranha_args: args.clone(),
      ..Default::default()
    };

    for (_, rule) in rule_store.rule_graph().rules_by_name().clone() {
      if rule.is_seed_rule() {
        rule_store.add_to_global_rules(&InstantiatedRule::new(&rule, args.input_substitutions()));
      }
    }
    info!(
      "Number of rules and edges loaded : {:?}",
      rule_store.rule_graph.get_number_of_rules_and_edges()
    );
    trace!("Rule Store {}", format!("{rule_store:#?}"));
    rule_store
  }

  /// Add a new global rule, along with grep heuristics (If it doesn't already exist)
  pub(crate) fn add_to_global_rules(&mut self, rule: &InstantiatedRule) {
    let r = rule.clone();
    if !self.global_rules.iter().any(|r| {
      r.name().eq(&rule.name()) && r.replace().eq(&rule.replace()) && r.query().eq(&rule.query())
    }) {
      #[rustfmt::skip]
      debug!("{}", format!("Added Global Rule : {:?} - {}", r.name(), r.query()).bright_blue());
      self.global_rules.push(r);
    }
  }

  /// Get the compiled query for the `query_str` from the cache
  /// else compile it, add it to the cache and return it.
  pub(crate) fn query(&mut self, query_str: &String) -> &Query {
    self
      .rule_query_cache
      .entry(query_str.to_string())
      .or_insert_with(|| {
        self
          .piranha_args
          .piranha_language()
          .create_query(query_str.to_string())
      })
  }

  /// Get the next rules to be applied grouped by the scope in which they should be performed.
  pub(crate) fn get_next(
    &self, rule_name: &String, tag_matches: &HashMap<String, String>,
  ) -> HashMap<String, Vec<InstantiatedRule>> {
    // let rule_name = rule.name();
    let mut next_rules: HashMap<String, Vec<InstantiatedRule>> = HashMap::new();
    // Iterate over each entry (Edge) in the adjacency list corresponding to `rule_name`
    for (scope, to_rule) in self.rule_graph.get_neighbors(rule_name) {
      let to_rule_name = &self.rule_graph().rules_by_name()[&to_rule];
      // If the to_rule_name is a dummy rule, skip it and rather return it's next rules.
      if to_rule_name.is_dummy_rule() {
        // Call this method recursively on the dummy node
        for (next_next_rules_scope, next_next_rules) in
          self.get_next(to_rule_name.name(), tag_matches)
        {
          for next_next_rule in next_next_rules {
            // Group the next rules based on the scope
            next_rules.collect(String::from(&next_next_rules_scope), next_next_rule)
          }
        }
      } else {
        // Group the next rules based on the scope
        next_rules.collect(
          String::from(&scope),
          InstantiatedRule::new(to_rule_name, tag_matches),
        );
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
      .piranha_args()
      .piranha_language()
      .scopes()
      .iter()
      .find(|level| level.name().eq(scope_level))
      .map(|scope| scope.rules().to_vec())
      .unwrap_or_else(Vec::new)
  }

  /// To create the current set of global rules, certain substitutions were applied.
  /// This method creates a regex pattern matching these substituted values.
  ///
  /// At the directory level, we would always look to perform global rules. However this is expensive because
  /// it requires parsing each file. To overcome this, we apply this simple
  /// heuristic to find the (upper bound) files that would match one of our current global rules.
  /// This heuristic reduces the number of files to parse.
  ///
  pub(crate) fn get_grep_heuristics(&self) -> Regex {
    let reg_x = self
      .global_rules()
      .iter()
      .flat_map(|r| r.substitutions().values())
      .sorted()
      //Remove duplicates
      .dedup()
      //FIXME: Dirty trick to remove true and false. Ideally, grep heuristic could be a field in itself for a rule.
      // Since not all "holes" could be used as grep heuristic.
      .filter(|x| {
        !x.is_empty() && !x.to_lowercase().eq("true") && !x.to_lowercase().as_str().eq("false")
      })
      .join("|");
    Regex::new(reg_x.as_str()).unwrap()
  }

  pub(crate) fn does_file_extension_match(&self, de: &jwalk::DirEntry<((), ())>) -> bool {
    de.path()
      .extension()
      .and_then(|e| {
        e.to_str()
          .filter(|x| x.eq(&self.piranha_args().get_language()))
      })
      .is_some()
  }

  /// Checks if any global rule has a hole
  pub(crate) fn any_global_rules_has_holes(&self) -> bool {
    self.global_rules().iter().any(|x| !x.holes().is_empty())
  }

  /// Gets all the files from the code base that (i) have the language appropriate file extension, and (ii) contains the grep pattern.
  /// Note that `WalkDir` traverses the directory with parallelism.
  /// If all the global rules have no holes (i.e. we will have no grep patterns), we will try to find a match for each global rule in every file in the target.
  pub(crate) fn get_relevant_files(&self, path_to_codebase: &str) -> HashMap<PathBuf, String> {
    let _path_to_codebase = Path::new(path_to_codebase).to_path_buf();

    //If the path_to_codebase is a file, then execute piranha on it
    if _path_to_codebase.is_file() {
      return HashMap::from_iter([(
        _path_to_codebase.clone(),
        read_file(&_path_to_codebase).unwrap(),
      )]);
    }
    let mut files: HashMap<PathBuf, String> = WalkDir::new(path_to_codebase)
      // Walk over the entire code base
      .into_iter()
      // Ignore errors
      .filter_map(|e| e.ok())
      // Filter files with the desired extension
      .filter(|de| self.does_file_extension_match(de))
      // Read the file
      .map(|f| (f.path(), read_file(&f.path()).unwrap()))
      .collect();

    if self.any_global_rules_has_holes() {
      let pattern = self.get_grep_heuristics();
      files = files
        .iter()
        // Filter the files containing the desired regex pattern
        .filter(|x| pattern.is_match(x.1.as_str()))
        .map(|(x, y)| (x.clone(), y.clone()))
        .collect();
    }
    debug!(
      "{}",
      format!("{} files will be analyzed.", files.len()).green()
    );
    files
  }
}

impl Default for RuleStore {
  fn default() -> Self {
    RuleStore {
      rule_graph: RuleGraph::default(),
      rule_query_cache: HashMap::default(),
      global_rules: Vec::default(),
      piranha_args: PiranhaArgumentsBuilder::default().build(),
    }
  }
}

fn read_config_files(args: &PiranhaArguments) -> (Vec<Rule>, Vec<OutgoingEdges>) {
  let path_to_config = Path::new(args.path_to_configurations());
  // Read the language specific cleanup rules and edges
  let language_rules: Rules = args.piranha_language().rules().clone().unwrap_or_default();
  let language_edges: Edges = args.piranha_language().edges().clone().unwrap_or_default();

  // Read the API specific cleanup rules and edges
  let mut input_rules: Rules = read_toml(&path_to_config.join("rules.toml"), true);
  let input_edges: Edges = read_toml(&path_to_config.join("edges.toml"), true);

  for r in input_rules.rules.iter_mut() {
    r.add_to_seed_rules_group();
  }

  let all_rules = [language_rules.rules, input_rules.rules].concat();
  let all_edges = [language_edges.edges, input_edges.edges].concat();

  (all_rules, all_edges)
}
