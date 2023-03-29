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

use crate::{
  common_java_rules::{JavaRules, MATCH_ALL_PRIVATE_FIELD, MATCH_IDENTIFIER, MATCH_PRIVATE_FIELD},
  execute_piranha,
  models::{
    default_configs::{default_code_snippet, default_path_to_codebase},
    matches::Match,
    outgoing_edges::{OutgoingEdges, OutgoingEdgesBuilder},
    piranha_arguments::PiranhaArguments,
    piranha_output::PiranhaOutputSummary,
    rule::Rule,
    rule_graph::{RuleGraph, RuleGraphBuilder},
  },
  piranha_rule,
};
use itertools::Itertools;
use serde_json::json;
use std::collections::HashMap;

struct Step {
  rules: Vec<Rule>,
  edges: Vec<OutgoingEdges>,
  path_to_codebase: Option<String>,
  code_snippet: Option<String>,
  summaries: Vec<PiranhaOutputSummary>,
  dry_run: bool,
  piranha_arguments: PiranhaArguments,
}

impl Step {
  fn rule_graph(&self) -> RuleGraph {
    RuleGraphBuilder::default()
      .rules(self.rules.clone())
      .edges(self.edges.clone())
      .build()
  }

  fn execute(&mut self) {
    self.update_piranha_argument();
    self.summaries = execute_piranha(&self.piranha_arguments);
  }

  fn update_piranha_argument(&mut self) {
    self.piranha_arguments.set_path_to_codebase(
      self
        .path_to_codebase
        .clone()
        .unwrap_or(default_path_to_codebase()),
    );
    self
      .piranha_arguments
      .set_code_snippet(self.code_snippet.clone().unwrap_or(default_code_snippet()));
    self.piranha_arguments.set_dry_run(self.dry_run);
    self.piranha_arguments.set_rule_graph(self.rule_graph());
  }

  fn get_matches(&self, rule_name: &str) -> Vec<Match> {
    self
      .summaries
      .iter()
      .flat_map(|summary| {
        summary
          .matches()
          .iter()
          .filter(|(rule, _)| rule.eq(rule_name))
          .map(|(_, mtch)| mtch.clone())
      })
      .collect()
  }
}

pub fn delete_unused_private_fields(
  piranha_arguments: &PiranhaArguments, summaries: &Vec<PiranhaOutputSummary>,
) {
  let java_rules = JavaRules::new();
  for summary in summaries {
    if summary.content().is_empty() {
      continue;
    }
    let original_private_field_usages =
      get_private_field_usages(piranha_arguments, summary.original_content());
    let modified_private_field_usages =
      get_private_field_usages(piranha_arguments, summary.content());

    original_private_field_usages
      .iter()
      .for_each(|(identifier, before_frequency)| {
        let after_frequency = modified_private_field_usages.get(identifier).unwrap_or(&0);
        if before_frequency > after_frequency && after_frequency == &1 {
          let mut step = Step {
            rules: vec![piranha_rule! {
                name = MATCH_PRIVATE_FIELD,
                query = java_rules.render(MATCH_PRIVATE_FIELD, &json!({"name": identifier})),
                replace_node = "field",
                replace = ""
            }],
            edges: vec![],
            path_to_codebase: Some(summary.path().to_string()),
            code_snippet: None,
            summaries: vec![],
            dry_run: false,
            piranha_arguments: piranha_arguments.clone(),
          };

          step.execute();
        }
      });
  }
}

fn get_private_field_usages(
  piranha_arguments: &PiranhaArguments, code_snippet: &String,
) -> HashMap<String, u32> {
  let java_rules = JavaRules::new();
  let mut step = Step {
    rules: vec![
      piranha_rule! {
          name = MATCH_ALL_PRIVATE_FIELD,
          query = java_rules.render(MATCH_ALL_PRIVATE_FIELD, &())
      },
      piranha_rule! {
          name = MATCH_IDENTIFIER,
          query = java_rules.render(MATCH_IDENTIFIER, &json!({"name": "@name"})),
          holes= ["name".to_string()],
          is_seed_rule= false
      },
    ],
    edges: vec![OutgoingEdgesBuilder::default()
      .frm(MATCH_ALL_PRIVATE_FIELD.to_string())
      .to(vec![MATCH_IDENTIFIER.to_string()])
      .scope("Class".to_string())
      .build()
      .unwrap()],
    path_to_codebase: None,
    code_snippet: Some(code_snippet.to_string()),
    summaries: vec![],
    dry_run: true,
    piranha_arguments: piranha_arguments.clone(),
  };
  step.execute();

  return step
    .get_matches(MATCH_IDENTIFIER)
    .iter()
    .map(|m| m.matches().get("identifier").unwrap())
    .into_group_map_by(|m| m.to_string())
    .into_iter()
    .map(|(k, v)| (k, v.len() as u32))
    .collect();
}
