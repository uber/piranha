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
use tree_sitter::Node;

use crate::utilities::{
  tree_sitter_utilities::{get_context, get_node_for_range, substitute_tags, PiranhaHelpers},
  MapOfVec,
};

use super::{
  constraint::Constraint, edit::Edit, rule_store::RuleStore, source_code_unit::SourceCodeUnit,
};

static FEATURE_FLAG_API_GROUP: &str = "Feature-flag API cleanup";

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
// Represents the `rules.toml` file
pub(crate) struct Rules {
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

  pub(crate) fn update_replace(&mut self, substitutions: &HashMap<String, String>) {
    self.replace = substitute_tags(self.replace(), substitutions);
  }

  pub(crate) fn update_query(&mut self, substitutions: &HashMap<String, String>) {
    self.query = substitute_tags(self.query(), substitutions);
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
    // Context contains -  the changed node in the previous edit, its's parent, grand parent and great grand parent
    let context = || {
      get_context(
        source_code_unit.root_node(),
        previous_edit_start,
        previous_edit_end,
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
  pub(crate) fn get_edit(
    &self, source_code_unit: &SourceCodeUnit, rule_store: &mut RuleStore, node: Node,
    recursive: bool,
  ) -> Option<Edit> {
    // Get all matches for the query in the given scope `node`.
    let all_query_matches = node.get_all_matches_for_query(
      source_code_unit.code(),
      rule_store.get_query(&self.get_query()),
      recursive,
      Some(self.replace_node()),
    );

    // Return the first match that satisfies constraint of the rule
    for (range, tag_substitutions) in all_query_matches {
      let matched_node = get_node_for_range(
        source_code_unit.root_node(),
        range.start_byte,
        range.end_byte,
      );
      if matched_node.satisfies_constraint(
        source_code_unit.clone(),
        self,
        &tag_substitutions,
        rule_store,
      ) {
        let replacement = substitute_tags(self.replace(), &tag_substitutions);
        return Some(Edit::new(
          range,
          replacement,
          self.clone(),
          tag_substitutions,
        ));
      }
    }
    None
  }
}

mod test {

  #[cfg(test)]
  use {
    super::Rule,
    crate::{
      models::{constraint::Constraint, rule_store::RuleStore, source_code_unit::SourceCodeUnit},
      utilities::tree_sitter_utilities::get_parser,
    },
    std::collections::HashMap,
    std::path::PathBuf,
  };

  #[cfg(test)]
  impl Rule {
    pub(crate) fn new(
      name: &str, query: &str, replace_node: &str, replace: &str, holes: Option<Vec<String>>,
      constraints: Option<Vec<Constraint>>,
    ) -> Self {
      Self {
        name: name.to_string(),
        query: query.to_string(),
        replace_node: replace_node.to_string(),
        replace: replace.to_string(),
        groups: None,
        holes,
        constraints,
        grep_heuristics: None,
      }
    }
  }

  #[test]
  fn test_rule_try_instantiate_positive() {
    let rule = Rule::new("test","(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"@variable_name\"))",
    "@abc", "",Some(vec![String::from("variable_name")]), None);
    let substitutions: HashMap<String, String> = HashMap::from([
      (String::from("variable_name"), String::from("foobar")),
      (String::from("@a.lhs"), String::from("something")), // Should not substitute, since it `a.lhs` is not in `rule.holes`
    ]);
    let instantiated_rule = rule.try_instantiate(&substitutions);
    assert!(instantiated_rule.is_ok());
    assert_eq!(instantiated_rule.ok().unwrap().query(), "(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"foobar\"))")
  }

  #[test]
  fn test_rule_try_instantiate_negative() {
    let rule = Rule::new("test","(((assignment_expression left: (_) @a.lhs right: (_) @a.rhs) @abc) (#eq? @a.lhs \"@variable_name\"))",
    "abc", "",Some(vec![String::from("variable_name")]), None);
    let substitutions: HashMap<String, String> = HashMap::from([
      (String::from("@a.lhs"), String::from("something")), // Should not substitute, since it `a.lhs` is not in `rule.holes`
    ]);
    let instantiated_rule = rule.try_instantiate(&substitutions);
    assert!(instantiated_rule.is_err());
  }

  #[test]
  fn test_get_edit_positive_recursive() {
    let rule = Rule::new("test", "(
                       ((local_variable_declaration
                                       declarator: (variable_declarator
                                                           name: (_) @variable_name
                                                           value: [(true) (false)] @init)) @variable_declaration)
                       )", "variable_declaration", "" ,None, 
                       Some(vec![
                      Constraint::new(String::from("(method_declaration) @md"),
                        vec![String::from("(
                          ((assignment_expression
                                          left: (_) @a.lhs
                                          right: (_) @a.rhs) @assignment)
                          (#eq? @a.lhs \"@variable_name\")
                          (#not-eq? @a.rhs \"@init\")
                        )")]),
                      ]));
    let source_code = "class Test {
      pub void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = true;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }";

    let mut rule_store = RuleStore::dummy();

    let mut parser = get_parser(String::from("java"));

    let source_code_unit = SourceCodeUnit::new(
      &mut parser,
      source_code.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
    );
    let node = source_code_unit.root_node();
    let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
    assert!(edit.is_some());
  }

  #[test]
  fn test_get_edit_negative_recursive() {
    let rule = Rule::new("test", "(
                       ((local_variable_declaration
                                       declarator: (variable_declarator
                                                           name: (_) @variable_name
                                                           value: [(true) (false)] @init)) @variable_declaration)
                       )", "variable_declaration", "" ,None, 
                       Some(vec![
                      Constraint::new(String::from("(method_declaration) @md"),
                        vec![String::from("(
                          ((assignment_expression
                                          left: (_) @a.lhs
                                          right: (_) @a.rhs) @assignment)
                          (#eq? @a.lhs \"@variable_name\")
                          (#not-eq? @a.rhs \"@init\")
                        )")]),
                      ]));
    let source_code = "class Test {
      pub void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }";

    let mut rule_store = RuleStore::dummy();
    let mut parser = get_parser(String::from("java"));

    let source_code_unit = SourceCodeUnit::new(
      &mut parser,
      source_code.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
    );
    let node = source_code_unit.root_node();
    let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
    assert!(edit.is_none());
  }

  #[test]
  fn test_get_edit_for_context_positive() {
    let rule = Rule::new(
      "test",
      "(
      (binary_expression
          left : (_)* @lhs
          operator:\"&&\"
          right: [(true) (parenthesized_expression (true))]
      )
  @binary_expression)",
      "binary_expression",
      "",
      None,
      None,
    );

    let source_code = "class A {
      boolean f = something && true;
    }";

    let mut rule_store = RuleStore::dummy();

    let mut parser = get_parser(String::from("java"));

    let source_code_unit = SourceCodeUnit::new(
      &mut parser,
      source_code.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
    );
    let edit = Rule::get_edit_for_context(
      &source_code_unit,
      41 as usize,
      44 as usize,
      &mut rule_store,
      &vec![rule],
    );
    // let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
    assert!(edit.is_some());
  }

  #[test]
  fn test_get_edit_for_context_negative() {
    let rule = Rule::new(
      "test",
      "(
      (binary_expression
          left : (_)* @lhs
          operator:\"&&\"
          right: [(true) (parenthesized_expression (true))]
      )
  @binary_expression)",
      "binary_expression",
      "",
      None,
      None,
    );

    let source_code = "class A {
      boolean f = true;
      boolean x = something && true;
    }";

    let mut rule_store = RuleStore::dummy();

    let mut parser = get_parser(String::from("java"));

    let source_code_unit = SourceCodeUnit::new(
      &mut parser,
      source_code.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
    );
    let edit = Rule::get_edit_for_context(
      &source_code_unit,
      29 as usize,
      33 as usize,
      &mut rule_store,
      &vec![rule],
    );
    // let edit = rule.get_edit(&source_code_unit, &mut rule_store, node, true);
    assert!(edit.is_none());
  }
}
