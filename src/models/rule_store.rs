/*
Copyright (c) 2023 Uber Technologies, Inc.

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
use log::{debug, trace};
use regex::Regex;

use crate::{
  models::capture_group_patterns::CGPattern, models::piranha_arguments::PiranhaArguments,
  models::scopes::ScopeQueryGenerator, utilities::read_file,
};

use super::{
  capture_group_patterns::CompiledCGPattern, language::PiranhaLanguage, rule::InstantiatedRule,
};
use glob::Pattern;

/// This maintains the state for Piranha.
#[derive(Debug, Getters, Default)]
pub(crate) struct RuleStore {
  // Caches the compiled tree-sitter queries.
  rule_query_cache: HashMap<String, CompiledCGPattern>,
  // Current global rules to be applied.
  #[get = "pub"]
  global_rules: Vec<InstantiatedRule>,

  #[get = "pub"]
  language: PiranhaLanguage,
}

impl RuleStore {
  pub(crate) fn new(args: &PiranhaArguments) -> RuleStore {
    let mut rule_store = RuleStore {
      language: args.language().clone(),
      ..Default::default()
    };

    for rule in args.rule_graph().rules().clone() {
      if *rule.is_seed_rule() {
        rule_store.add_to_global_rules(&InstantiatedRule::new(&rule, &args.input_substitutions()));
      }
    }
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
      debug!("{}", format!("Added Global Rule : {:?} - {}", r.name(), r.query().pattern()).bright_blue());
      self.global_rules.push(r);
    }
  }

  /// Get the compiled query for the `query_str` from the cache
  /// else compile it, add it to the cache and return it.
  pub(crate) fn query(&mut self, cg_pattern: &CGPattern) -> &CompiledCGPattern {
    let pattern = cg_pattern.pattern();
    if pattern.starts_with("rgx ") {
      return &*self
        .rule_query_cache
        .entry(pattern)
        .or_insert_with(|| CompiledCGPattern::R(cg_pattern.extract_regex().unwrap()));
    }

    &*self
      .rule_query_cache
      .entry(pattern.to_string())
      .or_insert_with(|| CompiledCGPattern::Q(self.language.create_query(pattern)))
  }

  // For the given scope level, get the ScopeQueryGenerator from the `scope_config.toml` file
  pub(crate) fn get_scope_query_generators(&self, scope_level: &str) -> Vec<ScopeQueryGenerator> {
    self
      .language()
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

  /// Checks if any global rule has a hole
  pub(crate) fn any_global_rules_has_holes(&self) -> bool {
    self.global_rules().iter().any(|x| !x.holes().is_empty())
  }

  /// Gets all the files from the code base that (i) have the language appropriate file extension, and (ii) contains the grep pattern.
  /// Note that `WalkDir` traverses the directory with parallelism.
  /// If all the global rules have no holes (i.e. we will have no grep patterns), we will try to find a match for each global rule in every file in the target.
  pub(crate) fn get_relevant_files(
    &self, path_to_codebase: &str, include: &Vec<Pattern>, exclude: &Vec<Pattern>,
  ) -> HashMap<PathBuf, String> {
    let _path_to_codebase = Path::new(path_to_codebase).to_path_buf();

    //If the path_to_codebase is a file, then execute piranha on it
    if _path_to_codebase.is_file() {
      return HashMap::from_iter([(
        _path_to_codebase.clone(),
        read_file(&_path_to_codebase).unwrap(),
      )]);
    }

    let mut files: HashMap<PathBuf, String> = WalkDir::new(path_to_codebase)
      // walk over the entire code base
      .into_iter()
      // ignore errors
      .filter_map(|e| e.ok())
      // only retain the included paths (if any)
      .filter(|f| include.is_empty() || include.iter().any(|p| p.matches_path(&f.path())))
      // filter out all excluded paths (if any)
      .filter(|f| exclude.is_empty() || exclude.iter().all(|p| !p.matches_path(&f.path())))
      // filter files with the desired extension
      .filter(|de| self.language().can_parse(de))
      // read the file
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
