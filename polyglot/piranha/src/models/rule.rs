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
  collections::{HashMap, HashSet},
  fmt,
};

use colored::Colorize;
use itertools::Itertools;
use serde_derive::Deserialize;

use crate::utilities::{tree_sitter_utilities::substitute_tags, MapOfVec};

use super::constraint::Constraint;

static SEED: &str = "Seed Rule";
static CLEAN_UP: &str = "Cleanup Rule";

#[derive(Deserialize, Debug, Clone, Default, PartialEq)]
// Represents the `rules.toml` file
pub(crate) struct Rules {
  pub(crate) rules: Vec<Rule>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum Rule {
  Rewrite {
    /// Name of the rule. (It is unique)
    name: String,
    /// Tree-sitter query as string
    query: String,
    /// The tag corresponding to the node to be replaced
    replace_node: String,
    /// Replacement pattern
    replace: String,
    /// Group(s) to which the rule belongs
    groups: Option<HashSet<String>>,
    /// Holes that need to be filled, in order to instantiate a rule
    holes: Option<HashSet<String>>,
    /// Additional constraints for matching the rule
    constraints: Option<HashSet<Constraint>>,
    /// Heuristics for identifying potential files containing occurrence of the rule.
    grep_heuristics: Option<HashSet<String>>,
  },

  MatchOnly {
    /// Name of the rule. (It is unique)
    name: String,
    /// Tree-sitter query as string
    query: String,
    /// Group(s) to which the rule belongs
    groups: Option<HashSet<String>>,
    /// Holes that need to be filled, in order to instantiate a rule
    holes: Option<HashSet<String>>,
    /// Additional constraints for matching the rule
    constraints: Option<HashSet<Constraint>>,
    /// Heuristics for identifying potential files containing occurrence of the rule.
    grep_heuristics: Option<HashSet<String>>,
  },

  Dummy {
    /// Name of the rule. (It is unique)
    name: String,
  },
}

impl Default for Rule {
  fn default() -> Self {
    Self::Dummy {
      name: "Dummy Rule".to_string(),
    }
  }
}

impl fmt::Display for Rule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Rule::Rewrite { name, query, replace, .. } => {
        write!(f, "Rewrite Rule: {}\n Query: {}\n Replace: {}", name, query, replace)
      }
      Rule::MatchOnly { name, query, .. } => {
        write!(f, "Match Only Rule: {}\n Query: {}", name, query)
      }
      Rule::Dummy { name } => write!(f, "Dummy Rule: {}", name),
    }
  }
}

impl PartialEq for Rule {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (
        Self::Rewrite {
          name: l_name,
          query: l_query,
          replace_node: l_replace_node,
          replace: l_replace,
          ..
        },
        Self::Rewrite {
          name: r_name,
          query: r_query,
          replace_node: r_replace_node,
          replace: r_replace,
          ..
        },
      ) => {
        l_name == r_name
          && l_query == r_query
          && l_replace_node == r_replace_node
          && l_replace == r_replace
      }

      (
        Self::MatchOnly {
          name: l_name,
          query: l_query,
          ..
        },
        Self::MatchOnly {
          name: r_name,
          query: r_query,
          ..
        },
      ) => l_name == r_name && l_query == r_query,
      (Self::Dummy { name: l_name }, Self::Dummy { name: r_name }) => l_name == r_name,
      _ => false,
    }
  }
}

impl Rule {
  pub(crate) fn is_seed_rule(&self) -> bool {
    self.groups().contains(&SEED.to_string())
  }

  /// Instantiate `self` with substitutions or panic.
  pub(crate) fn instantiate(&self, substitutions: &HashMap<String, String>) -> Rule {
    if let Ok(r) = self.try_instantiate(substitutions) {
      return r;
    }
    #[rustfmt::skip]
      panic!("{}", format!("Could not instantiate the rule {:?} with substitutions {:?}", self, substitutions).red());
  }

  fn _get_substitution_for_holes(
    &self, substitutions: &HashMap<String, String>,
  ) -> Result<HashMap<String, String>, String> {
    let relevant_substitutions = self
      .holes()
      .iter()
      .filter_map(|hole| substitutions.get(hole).map(|subs| (hole, subs)))
      .map(|(a, b)| (a.clone(), b.clone()))
      .collect::<HashMap<String, String>>();

    if relevant_substitutions.len() != self.holes().len() {
      return Err(format!(
        "Could not instantiate a rule - {:?}. Some Holes {:?} not found in table {:?}",
        self,
        self.holes(),
        substitutions
      ));
    }
    Ok(relevant_substitutions)
  }
  /// Tries to instantiate the rule (`self`) based on the substitutions.
  /// Note this could fail if the `substitutions` doesn't contain mappings for each hole.
  pub(crate) fn try_instantiate(
    &self, substitutions: &HashMap<String, String>,
  ) -> Result<Rule, String> {
    self
      ._get_substitution_for_holes(substitutions)
      .map(|x| self.apply_substitutions(&x))
  }

  fn apply_substitutions(&self, substitutions: &HashMap<String, String>) -> Rule {
    let update = |value: &String, is_tree_sitter_query: bool| {
      substitute_tags(value.to_string(), substitutions, is_tree_sitter_query)
    };

    let gh = HashSet::from_iter(
      self
        ._get_substitution_for_holes(substitutions)
        .unwrap()
        .values()
        .map(|x| x.to_string()),
    );

    match &self {
      r @ Rule::Rewrite {
        query,
        replace,
        replace_node,
        ..
      } => Rule::Rewrite {
        query: update(query, true),
        replace: update(replace, false),
        name: r.name(),
        replace_node: replace_node.to_string(),
        groups: Some(r.groups()),
        holes: Some(r.holes()),
        constraints: Some(r.constraints()),
        grep_heuristics: Some(HashSet::from(gh)),
      },
      r @ Rule::MatchOnly { query, .. } => Rule::MatchOnly {
        query: update(query, true),
        name: r.name(),
        groups: Some(r.groups()),
        holes: Some(r.holes()),
        constraints: Some(r.constraints()),
        grep_heuristics: Some(HashSet::from(gh)),
      },
      _ => self.clone(),
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
      rules_by_name.insert(rule.name(), rule.clone());
      for tag in rule.groups() {
        rules_by_group.collect(tag.to_string(), rule.name());
      }
    }
    (rules_by_name, rules_by_group)
  }

  /// Adds the rule to a new group - "SEED" if applicable.
  pub(crate) fn add_to_seed_rules_group(&mut self) {
    let mut grps = self.groups();

    if !self.groups().contains(&CLEAN_UP.to_string()) {
      grps.insert(SEED.to_string());
    }
    if let Rule::MatchOnly { groups, .. } | Rule::Rewrite { groups, .. } = self {
      *groups = Some(grps);
    }
  }

  // pub(crate) fn query(&self) -> String {
  //   match &self {
  //     Rule::Rewrite { query, .. } | Rule::MatchOnly { query, .. } => query.to_string(),
  //     _ => panic!("No query pattern!"),
  //   }
  // }

  pub(crate) fn constraints(&self) -> HashSet<Constraint> {
    match &self {
      Rule::MatchOnly {
        constraints: Some(cs),
        ..
      }
      | Rule::Rewrite {
        constraints: Some(cs),
        ..
      } => cs.clone(),
      _ => HashSet::new(),
    }
  }

  pub(crate) fn grep_heuristics(&self) -> HashSet<String> {
    match &self {
      Rule::MatchOnly {
        grep_heuristics: Some(cs),
        ..
      }
      | Rule::Rewrite {
        grep_heuristics: Some(cs),
        ..
      } => cs.clone(),
      _ => HashSet::new(),
    }
  }

  pub(crate) fn holes(&self) -> HashSet<String> {
    match &self.clone() {
      Rule::MatchOnly {
        holes: Some(cs), ..
      }
      | Rule::Rewrite {
        holes: Some(cs), ..
      } => cs.clone(),
      _ => HashSet::new(),
    }
  }

  fn groups(&self) -> HashSet<String> {
    if let Rule::MatchOnly {
      groups: Some(cs), ..
    }
    | Rule::Rewrite {
      groups: Some(cs), ..
    } = &self
    {
      cs.clone()
    } else {
      HashSet::new()
    }
  }

  pub(crate) fn name(&self) -> String {
    match &self {
      Rule::Dummy { name } => name,
      Rule::MatchOnly { name, .. } => name,
      Rule::Rewrite { name, .. } => name,
    }
    .to_string()
  }
}

#[cfg(test)]
impl Rule {
  pub(crate) fn new(
    name: &str, query: &str, replace_node: &str, replace: &str, holes: HashSet<String>,
    constraints: HashSet<Constraint>,
  ) -> Self {
    Rule::Rewrite {
      name: name.to_string(),
      query: query.to_string(),
      replace_node: replace_node.to_string(),
      replace: replace.to_string(),
      groups: None,
      holes: if holes.is_empty() { None } else { Some(holes) },
      constraints: if constraints.is_empty() {
        None
      } else {
        Some(constraints)
      },
      grep_heuristics: None,
    }
  }
}

#[cfg(test)]
#[path = "unit_tests/rule_test.rs"]
mod rule_test;
