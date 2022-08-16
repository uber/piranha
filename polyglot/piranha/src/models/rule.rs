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
use tree_sitter::{Node};

use crate::utilities::{
  tree_sitter_utilities::{get_context, get_node_for_range, substitute_tags, PiranhaHelpers},
  MapOfVec,
};

use super::{
  constraint::Constraint, edit::Edit, rule_store::RuleStore, source_code_unit::SourceCodeUnit, matches::Match,
};

static FEATURE_FLAG_API_GROUP: &str = "Feature-flag API cleanup";

#[derive(Deserialize, Debug, Clone, Hash, Default)]
// Represents the `rules.toml` file
pub(crate) struct Rules {
  pub(crate) rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone, Hash, Default)]
pub(crate) struct Rule {
  /// Name of the rule. (It is unique)
  name: String,
  /// Tree-sitter query as string
  query: Option<String>,
  /// The tag corresponding to the node to be replaced
  replace_node: Option<String>,
  /// Replacement pattern
  replace: Option<String>,
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
  pub(crate) fn is_feature_flag_cleanup(&self) -> bool {
    self.groups().iter().any(|t| t.eq(FEATURE_FLAG_API_GROUP))
  }

  // Dummy rules are helper rules that make it easier to define the rule graph.
  pub(crate) fn is_dummy_rule(&self) -> bool {
    self.query.is_none() && self.replace.is_none()
  }

  // Checks if a rule is `match-only` i.e. it has a query but no replace. 
  pub(crate) fn is_match_only_rule(&self) -> bool {
    self.query.is_some() && self.replace.is_none()
  }

  /// Instantiate `self` with substitutions or panic.
  pub(crate) fn instantiate(&self, substitutions: &HashMap<String, String>) -> Rule {
    if let Ok(r) = self.try_instantiate(substitutions) {
      return r;
    }
    #[rustfmt::skip]
      panic!("{}", format!("Could not instantiate the rule {:?} with substitutions {:?}", self, substitutions).red());
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
    self.update(&relevant_substitutions)
  }

  /// Create a new query from `self` by updating the `query` and `replace` based on the substitutions.
  fn update(&self, substitutions: &HashMap<String, String>) -> Result<Rule, String> {
    if substitutions.len() != self.holes().len() {
      #[rustfmt::skip]
        return Err(format!("Could not instantiate a rule - {:?}. Some Holes {:?} not found in table {:?}", self, self.holes(), substitutions));
    } else {
      let mut updated_rule = self.clone();
      if !updated_rule.holes().is_empty() {
        updated_rule.update_query(substitutions);
        updated_rule.update_replace(substitutions);
      }
      Ok(updated_rule)
    }
  }

  /// Groups the rules based on the field `rule.groups`
  /// Note: a rule can belong to more than one group.
  pub(crate) fn group_rules(
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

  /// Adds the rule to a new group - "Feature-flag API cleanup"
  pub(crate) fn add_to_feature_flag_api_group(&mut self) {
    let group_name: String = FEATURE_FLAG_API_GROUP.to_string();
    match self.groups.as_mut() {
      None => self.groups = Some(vec![group_name]),
      Some(_groups) => _groups.push(group_name),
    }
  }

  pub(crate) fn replace_node(&self) -> String {
    if let Some(rn) = &self.replace_node{
      return rn.to_string();
    }
    panic!("No replace_node pattern!")
  }

  pub(crate) fn query(&self) -> String {
    if let Some(q) = &self.query{
      return q.to_string();
    }
    panic!("No query pattern!")
  }

  pub(crate) fn replace(&self) -> String {
    if let Some(rp) = &self.replace{
      return rp.to_string();
    }
    panic!("No replace pattern!")
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

  pub(crate) fn holes(&self) -> Vec<String> {
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

  pub(crate) fn update_replace(&mut self, substitutions: &HashMap<String, String>) {
    self.set_replace(substitute_tags(self.replace(), substitutions));
  }

  pub(crate) fn update_query(&mut self, substitutions: &HashMap<String, String>) {
    self.set_query(substitute_tags(self.query(), substitutions));
  }

  pub(crate) fn name(&self) -> String {
    String::from(&self.name)
  }

  // Apply all the `rules` to the node, parent, grand parent and great grand parent.
  // Short-circuit on the first match.
  pub(crate) fn get_edit_for_context(
    source_code_unit: &SourceCodeUnit, previous_edit_start: usize, previous_edit_end: usize,
    rules_store: &mut RuleStore, rules: &Vec<Rule>,
  ) -> Option<Edit> {
    let changed_node = get_node_for_range(
      source_code_unit.root_node(),
      previous_edit_start,
      previous_edit_end,
    );
    // Context contains -  the changed node in the previous edit, its's parent, grand parent and great grand parent
    let context = || {
      get_context(
        source_code_unit.root_node(),
        changed_node,
        source_code_unit.code(),
        4,
      )
    };
    for rule in rules {
      for ancestor in &context() {
        if let Some(edit) = rule.get_edit(&source_code_unit.clone(), rules_store, *ancestor, false)
        {
          return Some(edit);
        }
      }
    }
    None
  }

  /// Gets the first match for the rule in `self`
  pub(crate) fn get_matches(
    &self, source_code_unit: &SourceCodeUnit, rule_store: &mut RuleStore, node: Node,
    recursive: bool,
  ) -> Vec<Match> {
    let mut output: Vec<Match> = vec![];
    // Get all matches for the query in the given scope `node`.
    let replace_node_tag = if self.is_match_only_rule() || self.is_dummy_rule() { None } else {Some(self.replace_node())};
    let all_query_matches = node.get_all_matches_for_query(
      source_code_unit.code(),
      rule_store.query(&self.query()),
      recursive,
      replace_node_tag
    );

    // Return the first match that satisfies constraint of the rule
    for p_match in all_query_matches {
      let matched_node = get_node_for_range(
        source_code_unit.root_node(),
        p_match.range().start_byte,
        p_match.range().end_byte,
      );
      
      if matched_node.satisfies_constraint(
        source_code_unit.clone(),
        self,
        p_match.matches(),
        rule_store,
      ) {
        output.push(p_match);
      }
    }
    output
  }


  /// Gets the first match for the rule in `self`
  pub(crate) fn get_edit(
    &self, source_code_unit: &SourceCodeUnit, rule_store: &mut RuleStore, node: Node,
    recursive: bool,
  ) -> Option<Edit> {
    // Get all matches for the query in the given scope `node`.

    return self.get_matches(source_code_unit, rule_store, node, recursive)
      .first().map(|p_match| {
        let replacement = substitute_tags(self.replace(), p_match.matches()).replace("\\n", "\n");
        return Edit::new(
          p_match.clone(),
          replacement,
          self.name()
        );
      });
  }

  pub(crate) fn set_replace(&mut self, replace: String) {
    self.replace = Some(replace);
  }

  pub(crate) fn set_query(&mut self, query: String) {
    self.query = Some(query);
  }
}

#[cfg(test)]
impl Rule {
  pub(crate) fn new(
    name: &str, query: &str, replace_node: &str, replace: &str, holes: Option<Vec<String>>,
    constraints: Option<Vec<Constraint>>,
  ) -> Self {
    Self {
      name: name.to_string(),
      query: Some(query.to_string()),
      replace_node: Some(replace_node.to_string()),
      replace: Some(replace.to_string()),
      groups: None,
      holes,
      constraints,
      grep_heuristics: None,
    }
  }
}

#[cfg(test)]
#[path = "unit_tests/rule_test.rs"]
mod rule_test;
