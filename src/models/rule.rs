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

use std::collections::{HashMap, HashSet};

use colored::Colorize;
use derive_builder::Builder;
use getset::Getters;
use serde_derive::Deserialize;

use crate::utilities::tree_sitter_utilities::substitute_tags;

use super::{
  constraint::Constraint,
  default_configs::{
    default_constraints, default_groups, default_holes, default_query, default_replace,
    default_replace_node, default_rule_name,
  },
};

static SEED: &str = "Seed Rule";
static CLEAN_UP: &str = "Cleanup Rule";

#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
// Represents the `rules.toml` file
pub(crate) struct Rules {
  pub(crate) rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Getters, Builder)]
pub(crate) struct Rule {
  /// Name of the rule. (It is unique)
  #[builder(default = "default_rule_name()")]
  #[get = "pub"]
  name: String,
  /// Tree-sitter query as string
  #[builder(default = "default_query()")]
  #[serde(default = "default_query")]
  #[get = "pub"]
  query: String,
  /// The tag corresponding to the node to be replaced
  #[builder(default = "default_replace_node()")]
  #[serde(default = "default_replace_node")]
  #[get = "pub"]
  replace_node: String,
  /// Replacement pattern
  #[builder(default = "default_replace()")]
  #[serde(default = "default_replace")]
  #[get = "pub"]
  replace: String,
  /// Group(s) to which the rule belongs

  #[builder(default = "default_groups()")]
  #[serde(default = "default_groups")]
  #[get = "pub"]
  groups: HashSet<String>,
  /// Holes that need to be filled, in order to instantiate a rule
  #[builder(default = "default_holes()")]
  #[serde(default = "default_holes")]
  #[get = "pub"]
  holes: HashSet<String>,
  /// Additional constraints for matching the rule
  #[builder(default = "default_constraints()")]
  #[serde(default = "default_constraints")]
  #[get = "pub"]
  constraints: HashSet<Constraint>,
}

impl Rule {
  /// Dummy rules are helper rules that make it easier to define the rule graph
  pub(crate) fn is_dummy_rule(&self) -> bool {
    *self.query() == default_query() && *self.replace_node() == default_replace_node()
  }

  /// Checks if `self` is a seed rule
  pub(crate) fn is_seed_rule(&self) -> bool {
    self.groups().contains(&SEED.to_string())
  }

  /// Checks if a rule is `match-only` i.e. it has a query but no replace_node
  pub(crate) fn is_match_only_rule(&self) -> bool {
    *self.query() != default_query() && *self.replace_node() == default_replace_node()
  }

  /// Create a new query from `self` by updating the `query` and `replace` based on the substitutions.
  fn substitute(&self, substitutions_for_holes: &HashMap<String, String>) -> Rule {
    if substitutions_for_holes.len() != self.holes().len() {
      #[rustfmt::skip]
      panic!("{}", format!( "Could not instantiate the rule {self:?} with substitutions {substitutions_for_holes:?}").red());
    }
    let updated_rule = self.clone();
    Rule {
      query: substitute_tags(updated_rule.query(), substitutions_for_holes, false),
      replace: substitute_tags(updated_rule.replace(), substitutions_for_holes, false),
      ..updated_rule
    }
  }

  /// Adds the rule to a new group - "SEED" if applicable.
  pub(crate) fn add_to_seed_rules_group(&mut self) {
    if self.groups().contains(&CLEAN_UP.to_string()) {
      return;
    }
    self.groups.insert(SEED.to_string());
  }
}

#[macro_export]
/// This macro can be used to construct a Rule (via the builder).'
/// Allows to use builder pattern more "dynamically"
///
/// Usage:
///
/// ```ignore
/// piranha_rule! {
///   name = "Some Rule".to_string(),
///   query= "(method_invocation name: (_) @name) @mi".to_string()
/// }
/// ```
///
/// expands to
///
/// ```ignore
/// RuleBuilder::default()
///      .name("Some Rule".to_string())
///      .query("(method_invocation name: (_) @name) @mi".to_string)
///      .build()
/// ```
///
macro_rules! piranha_rule {
  (name = $name:expr
                $(, query =$query: expr)?
                $(, replace_node = $replace_node:expr)?
                $(, replace = $replace:expr)?
                $(, holes = [$($hole: expr)*])?
                $(, groups = [$($group_name: expr)*])?
                $(, constraints = [$($constraint:tt)*])?
              ) => {
    $crate::models::rule::RuleBuilder::default()
    .name($name.to_string())
    $(.query($query.to_string()))?
    $(.replace_node($replace_node.to_string()))?
    $(.replace($replace.to_string()))?
    $(.holes(HashSet::from([$($hole.to_string(),)*])))?
    $(.groups(HashSet::from([$($group_name.to_string(),)*])))?
    $(.constraints(HashSet::from([$($constraint)*])))?
    .build().unwrap()
  };
}
pub use piranha_rule;

#[derive(Debug, Getters, Clone)]
pub(crate) struct InstantiatedRule {
  #[get = "pub"]
  rule: Rule,
  #[get = "pub"]
  substitutions: HashMap<String, String>,
}

impl InstantiatedRule {
  pub(crate) fn new(rule: &Rule, substitutions: &HashMap<String, String>) -> Self {
    let substitutions_for_holes = rule
      .holes()
      .iter()
      .filter_map(|h| substitutions.get(h).map(|s| (h.to_string(), s.to_string())))
      .collect();
    InstantiatedRule {
      rule: rule.substitute(&substitutions_for_holes),
      substitutions: substitutions_for_holes,
    }
  }

  pub fn name(&self) -> String {
    self.rule().name().to_string()
  }

  pub fn replace(&self) -> String {
    self.rule().replace().to_string()
  }

  pub fn query(&self) -> String {
    self.rule().query().to_string()
  }

  pub fn replace_node(&self) -> String {
    self.rule().replace_node().to_string()
  }

  pub fn holes(&self) -> &HashSet<String> {
    self.rule().holes()
  }

  pub fn constraints(&self) -> &HashSet<Constraint> {
    self.rule().constraints()
  }
}

#[cfg(test)]
#[path = "unit_tests/rule_test.rs"]
mod rule_test;
