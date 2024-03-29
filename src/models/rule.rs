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

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use colored::Colorize;
use derive_builder::Builder;
use getset::Getters;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::Deserialize;

use crate::utilities::Instantiate;

use super::{
  capture_group_patterns::CGPattern,
  default_configs::{
    default_filters, default_groups, default_holes, default_is_seed_rule, default_query,
    default_replace, default_replace_idx, default_replace_node, default_rule_name,
  },
  filter::Filter,
  Validator,
};

#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
// Represents the `rules.toml` file
pub struct Rules {
  pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone, Default, PartialEq, Getters, Builder, Eq)]
#[pyclass]
pub struct Rule {
  /// Name of the rule. (It is unique)
  #[builder(default = "default_rule_name()")]
  #[get = "pub"]
  #[pyo3(get)]
  name: String,
  /// Tree-sitter query as string
  #[builder(default = "default_query()")]
  #[serde(default = "default_query")]
  #[get = "pub"]
  #[pyo3(get)]
  query: CGPattern,
  /// The tag corresponding to the node to be replaced
  #[builder(default = "default_replace_node()")]
  #[serde(default = "default_replace_node")]
  #[get = "pub"]
  #[pyo3(get)]
  replace_node: String,
  /// The i'th child of node corresponding to the replace_node tag will be replaced
  #[builder(default = "default_replace_idx()")]
  #[serde(default = "default_replace_idx")]
  #[get = "pub"]
  #[pyo3(get)]
  replace_idx: u8,
  /// Replacement pattern
  #[builder(default = "default_replace()")]
  #[serde(default = "default_replace")]
  #[get = "pub"]
  #[pyo3(get)]
  replace: String,
  /// Group(s) to which the rule belongs
  #[builder(default = "default_groups()")]
  #[serde(default = "default_groups")]
  #[get = "pub"]
  #[pyo3(get)]
  groups: HashSet<String>,
  /// Holes that need to be filled, in order to instantiate a rule
  #[builder(default = "default_holes()")]
  #[serde(default = "default_holes")]
  #[get = "pub"]
  #[pyo3(get)]
  holes: HashSet<String>,
  /// Filters to test before applying a rule
  #[builder(default = "default_filters()")]
  #[serde(default = "default_filters")]
  #[get = "pub"]
  #[pyo3(get)]
  filters: HashSet<Filter>,

  /// Marks a rule as a seed rule
  #[builder(default = "default_is_seed_rule()")]
  #[serde(default = "default_is_seed_rule")]
  #[get = "pub"]
  #[pyo3(get)]
  is_seed_rule: bool,
}

impl Rule {
  /// Dummy rules are helper rules that make it easier to define the rule graph
  pub(crate) fn is_dummy_rule(&self) -> bool {
    *self.query() == default_query() && *self.replace_node() == default_replace_node()
  }

  /// Checks if a rule is `match-only` i.e. it has a query but no replace_node
  pub(crate) fn is_match_only_rule(&self) -> bool {
    *self.query() != default_query() && *self.replace_node() == default_replace_node()
  }
}

#[macro_export]
/// This macro can be used to construct a Rule (via the builder).'
/// Allows to use builder pattern more "dynamically"
///
/// Usage:
///
/// ```
/// piranha_rule! {
///   name = "Some Rule".to_string(),
///   query= "(method_invocation name: (_) @name) @mi".to_string()
/// }
/// ```
///
/// expands to
///
/// ```
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
                $(, replace_idx = $replace_idx:expr)?
                $(, replace = $replace:expr)?
                $(, holes = [$($hole: expr)*])?
                $(, is_seed_rule = $is_seed_rule:expr)?
                $(, groups = [$($group_name: expr)*])?
                $(, filters = [$($filter:tt)*])?
              ) => {
    $crate::models::rule::RuleBuilder::default()
    .name($name.to_string())
    $(.query($crate::models::capture_group_patterns::CGPattern::new($query.to_string())))?
    $(.replace_node($replace_node.to_string()))?
    $(.replace_idx($replace_idx.to_string()))?
    $(.replace($replace.to_string()))?
    $(.is_seed_rule($is_seed_rule))?
    $(.holes(std::collections::HashSet::from([$($hole.to_string(),)*])))?
    $(.groups(std::collections::HashSet::from([$($group_name.to_string(),)*])))?
    $(.filters(std::collections::HashSet::from([$($filter)*])))?
    .build().unwrap()
  };
}

#[pymethods]
impl Rule {
  #[new]
  fn py_new(
    name: String, query: Option<String>, replace: Option<String>, replace_idx: Option<u8>,
    replace_node: Option<String>, holes: Option<HashSet<String>>, groups: Option<HashSet<String>>,
    filters: Option<HashSet<Filter>>, is_seed_rule: Option<bool>,
  ) -> Self {
    let mut rule_builder = RuleBuilder::default();

    rule_builder.name(name);
    if let Some(q) = query {
      rule_builder.query(CGPattern::new(q));
    }

    if let Some(replace) = replace {
      rule_builder.replace(replace);
    }

    if let Some(replace_idx) = replace_idx {
      rule_builder.replace_idx(replace_idx);
    }

    if let Some(replace_node) = replace_node {
      rule_builder.replace_node(replace_node);
    }

    if let Some(holes) = holes {
      rule_builder.holes(holes);
    }

    if let Some(groups) = groups {
      rule_builder.groups(groups);
    }

    if let Some(filters) = filters {
      rule_builder.filters(filters);
    }

    if let Some(is_seed_rule) = is_seed_rule {
      rule_builder.is_seed_rule(is_seed_rule);
    }

    rule_builder.build().unwrap()
  }

  fn __repr__(&self) -> String {
    format!("{:?}", self)
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}

impl Validator for Rule {
  fn validate(&self) -> Result<(), String> {
    let validation = self
      .query()
      .validate()
      .and_then(|_: ()| self.filters().iter().try_for_each(|f| f.validate()));
    validation
  }
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
    let substitutions_for_holes: HashMap<String, String> = rule
      .holes()
      .iter()
      .filter_map(|h| substitutions.get(h).map(|s| (h.to_string(), s.to_string())))
      .collect();
    // Since filter_map (above) discards any element of `rules.holes()` for which there isn't a valid substitution,
    // checking that the lengths match is enough to verify all holes have a matching substitution.
    if substitutions_for_holes.len() != rule.holes().len() {
      #[rustfmt::skip]
      panic!("{}", format!( "Could not instantiate the rule {rule:?} with substitutions {substitutions_for_holes:?}").red());
    }
    InstantiatedRule {
      rule: rule.instantiate(&substitutions_for_holes),
      substitutions: substitutions_for_holes,
    }
  }

  pub fn name(&self) -> String {
    self.rule().name().to_string()
  }

  pub fn replace(&self) -> String {
    self.rule().replace().to_string()
  }

  pub fn query(&self) -> CGPattern {
    self.rule().query().clone()
  }

  pub fn replace_node(&self) -> Option<String> {
    if *self.rule().replace_node() != default_replace_node() {
      return Some(self.rule().replace_node().to_string());
    }
    None
  }

  pub fn replace_idx(&self) -> Option<u8> {
    if *self.rule().replace_idx() != default_replace_idx() {
      return Some(*self.rule().replace_idx());
    }
    None
  }

  pub fn holes(&self) -> &HashSet<String> {
    self.rule().holes()
  }

  pub fn filters(&self) -> &HashSet<Filter> {
    self.rule().filters()
  }
}

impl Instantiate for Rule {
  /// Create a new query from `self` by updating the `query` and `replace` based on the substitutions.
  /// This functions assumes that each hole in the rule can be substituted.
  /// i.e. It assumes that `substitutions_for_holes` is exhaustive and complete
  fn instantiate(&self, substitutions_for_holes: &HashMap<String, String>) -> Rule {
    let updated_rule = self.clone();
    Rule {
      query: updated_rule.query().instantiate(substitutions_for_holes),
      replace: updated_rule.replace().instantiate(substitutions_for_holes),
      ..updated_rule
    }
  }
}

impl Hash for Rule {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name().hash(state);
  }
}

#[cfg(test)]
#[path = "unit_tests/rule_test.rs"]
mod rule_test;
