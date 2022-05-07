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
use serde_derive::Deserialize;

use crate::utilities::MapOfVec;

static FEATURE_FLAG_API_GROUP: &str = "Feature-flag API cleanup";

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
// Represents the `rules.toml` file
pub struct Rules {
  pub(crate) rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct Rule {
  /// Name of the rule. (It is unique)
  name: String,
  /// Tree-sitter query as string
  query: String,
  /// The tag corresponding to the node to be replaced
  replace_node: String,
  /// Replacement pattern
  replace: String,
  /// Group(s) to which the rule belongs
  groups: Option<Vec<String>>,
  /// Holes that need to be filled, in order to instantiate a rule
  holes: Option<Vec<String>>,
  /// Additional constraints for matching the rule
  constraints: Option<Vec<Constraint>>,
  /// Heuristics for identifying potential files containing occurrence of the rule.
  grep_heuristics: Option<Vec<String>>,
}

impl Rule {
  /// Create a new query from `self` with the input `query` and `replace`
  pub(crate) fn update(
    &self, query: String, replace: String, substitutions: &HashMap<String, String>,
  ) -> Result<Rule, String> {
    if substitutions.len() != self.holes().len() {
      #[rustfmt::skip]
        return Err(format!("Could not instantiate a rule - {:?}. Some Holes {:?} not found in table {:?}", self, self.holes(), substitutions));
    } else {
      let mut updated_rule = self.clone();
      if !updated_rule.holes().is_empty() {
        updated_rule.set_query(query.substitute_tags(substitutions));
        updated_rule.set_replace(replace.substitute_tags(substitutions));
      }
      Ok(updated_rule)
    }
  }

  pub(crate) fn is_feature_flag_cleanup(&self) -> bool {
    self.groups().iter().any(|t| t.eq(FEATURE_FLAG_API_GROUP))
  }

  // Dummy rules are helper rules that make it easier to define the rule graph.
  pub(crate) fn is_dummy_rule(&self) -> bool {
    self.query.is_empty() && self.replace.is_empty()
  }

  /// Instantiate `self` with substitutions or panic.
  pub(crate) fn instantiate(&self, substitutions: &HashMap<String, String>) -> Rule {
    if let Ok(r) = self.try_instantiate(substitutions) {
      return r;
    }
    #[rustfmt::skip]
      panic!("{}", format!("Could not instantiate the rule {:?} with substitutions {:?}", self, substitutions).red());
  }

  /// Groups the rules based on the field `rule.groups`
  /// Note: a rule can belong to more than one group.
  pub(crate) fn get_grouped_rules(
    rules: &Vec<Rule>,
  ) -> (HashMap<String, Rule>, HashMap<String, Vec<String>>) {
    let mut rules_by_name = HashMap::new();
    let mut rules_by_group = HashMap::new();
    for rule in rules {
      rules_by_name.insert(rule.name.to_string(), rule.clone());
      for tag in rule.groups() {
        rules_by_group.collect(tag.to_string(), rule.name.to_string());
      }
    }
    (rules_by_name, rules_by_group)
  }

  /// Tries to instantiate the rule (`self`) based on the substitutions.
  /// Note this could fail if the `substitutions` doesn't contain mappings for each hole.
  pub(crate) fn try_instantiate(
    &self, substitutions: &HashMap<String, String>,
  ) -> Result<Rule, String> {
    let relevant_substitutions = self
      .holes()
      .iter()
      .filter_map(|hole| substitutions.get(hole).map(|subs| (hole, subs)))
      .map(|(a, b)| (a.clone(), b.clone()))
      .collect::<HashMap<String, String>>();
    self.update(self.query(), self.replace(), &relevant_substitutions)
  }

  /// Records the string that should be grepped in order to find files that
  /// potentially could match this global rule.
  pub(crate) fn add_grep_heuristics_for_global_rules(
    &mut self, substitutions: &HashMap<String, String>,
  ) {
    let mut gh = vec![];
    for hole in self.holes() {
      if let Some(x) = substitutions.get(&hole) {
        gh.push(x.clone());
      }
    }
    self.grep_heuristics = Some(gh.clone());
  }

  pub(crate) fn get_query(&self) -> String {
    self.query.clone()
  }

  /// Adds the rule to a new group - "Feature-flag API cleanup"
  pub(crate) fn add_to_feature_flag_api_group(&mut self) {
    let group_name: String = FEATURE_FLAG_API_GROUP.to_string();
    match self.groups.as_mut() {
      None => self.groups = Some(vec![group_name]),
      Some(_groups) => _groups.push(group_name),
    }
  }

  pub(crate) fn replace_node(&self) -> String {
    String::from(&self.replace_node)
  }

  fn query(&self) -> String {
    String::from(&self.query)
  }

  pub(crate) fn replace(&self) -> String {
    String::from(&self.replace)
  }

  pub(crate) fn constraints(&self) -> Vec<Constraint> {
    match &self.constraints {
      Some(cs) => cs.clone(),
      None => vec![],
    }
  }

  pub(crate) fn grep_heuristics(&self) -> Vec<String> {
    match &self.grep_heuristics {
      Some(cs) => cs.clone(),
      None => vec![],
    }
  }

  fn holes(&self) -> Vec<String> {
    match &self.holes {
      Some(cs) => cs.clone(),
      None => vec![],
    }
  }

  fn groups(&self) -> Vec<String> {
    match &self.groups {
      Some(cs) => cs.clone(),
      None => vec![],
    }
  }

  pub(crate) fn set_replace(&mut self, replace: String) {
    self.replace = replace;
  }

  pub(crate) fn set_query(&mut self, query: String) {
    self.query = query;
  }

  pub(crate) fn name(&self) -> String {
    String::from(&self.name)
  }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Constraint {
  /// Scope in which the constraint query has to be applied
  matcher: String,
  /// The Tree-sitter queries that need to be applied in the `matcher` scope
  queries: Vec<String>,
}

impl Constraint {
  pub(crate) fn queries(&self) -> &[String] {
    &self.queries
  }

  pub(crate) fn matcher(&self) -> String {
    String::from(&self.matcher)
  }
}

pub trait RuleHelper {
  /// replaces the all the occurrences of keys (of `substitutions` map) in the string with its corresponding value.
  fn substitute_tags(&self, substitutions: &HashMap<String, String>) -> String;
}

impl RuleHelper for String {
  fn substitute_tags(&self, substitutions: &HashMap<String, String>) -> String {
    let mut output = String::from(self);
    for (tag, substitute) in substitutions {
      // Before replacing the key, it is transformed to a tree-sitter tag by adding `@` as prefix
      let key = format!("@{}", tag);
      output = output.replace(&key, substitute)
    }
    output
  }
}
