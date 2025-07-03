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

use crate::models::default_configs::TS_SCHEME;
use crate::models::language::PiranhaLanguage;
use crate::models::piranha_arguments::PiranhaArgumentsBuilder;
use crate::models::rule::Rule;
use crate::models::rule_graph::RuleGraphBuilder;

use crate::execute_piranha;
use crate::models::filter::filter;
use crate::models::rule::piranha_rule;

use crate::models::capture_group_patterns::PatternType;
use regex::Regex;
use std::string::String;

/// Find defined tags in a query
pub fn get_capture_groups_from_matcher(node: &Rule) -> Vec<String> {
  if node.query().pattern().is_empty() {
    return vec![];
  }

  match &node.query().pattern_type() {
    PatternType::Tsq => get_capture_groups_from_tsq(node.query().pattern()),
    PatternType::Regex => get_capture_groups_from_regex(node.query().extract_regex().unwrap()),
    PatternType::Cs => vec![],
    PatternType::Unknown => vec![],
  }
}

/// Find all tags used in predicates
pub fn get_capture_group_usage_from_matcher(node: &Rule) -> Vec<String> {
  if node.query().pattern().is_empty() {
    return vec![];
  }

  match &node.query().pattern_type() {
    PatternType::Tsq => get_capture_group_usage_from_tsq(node.query().pattern()),
    PatternType::Regex => get_capture_group_usage_from_regex(node.query().pattern()),
    PatternType::Cs => vec![],
    PatternType::Unknown => vec![],
  }
}

/// Find defined tags in a regex pattern
pub fn get_capture_groups_from_regex(re: Regex) -> Vec<String> {
  let mut tags = Vec::new();

  // Check all capture names (i.e., named groups) in the pattern
  for capture_name in re.capture_names().flatten() {
    let tag = format!("@{capture_name}");
    tags.push(tag);
  }
  tags
}

/// Find all tags used in a regex pattern
pub fn get_capture_group_usage_from_regex(pattern: String) -> Vec<String> {
  let re = Regex::new(r"@\w+").unwrap();
  let mut capture_groups = Vec::new();

  // Check all matches of @ followed by one or more word characters
  for mat in re.find_iter(pattern.as_str()) {
    capture_groups.push(mat.as_str().to_owned());
  }
  capture_groups
}

/// Find defined capture_groups in a query
pub fn get_capture_groups_from_tsq(pattern: String) -> Vec<String> {
  let rules = vec![piranha_rule! {
    name = "capture_groups",
    query = "(capture) @cap",
    filters =[
        filter! {
          ,not_enclosing_node = "(predicate) @pred"
        }
    ]
  }];

  let graph = RuleGraphBuilder::default().rules(rules).build();

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .rule_graph(graph)
    .language(PiranhaLanguage::from(TS_SCHEME))
    .code_snippet(pattern)
    .should_validate(false)
    .build();

  let output_summaries = execute_piranha(&piranha_arguments);

  // maps matched strings to a vec of strings
  let capture_groups = output_summaries
    .first()
    .map(|summary| {
      summary
        .matches()
        .iter()
        .map(|m| m.1.matched_string().clone())
        .collect::<Vec<String>>()
    })
    .unwrap_or_default();

  log::debug!("capture_groups: {capture_groups:?}");

  capture_groups
}

/// Find all capture_groups used in predicates
pub fn get_capture_group_usage_from_tsq(pattern: String) -> Vec<String> {
  let rules = vec![piranha_rule! {
    name = "capture_groups",
    query = "(
      [ (capture) @cap
        (identifier) @id
        (string) @str]
      (#match? @cap \"@\")
      (#match? @id \"@\")
      (#match? @str \"@\")
    )",
    filters =[
        filter! {
          enclosing_node = "(predicate) @pred"
        }
    ]
  }];

  let graph = RuleGraphBuilder::default().rules(rules).build();

  let piranha_arguments = PiranhaArgumentsBuilder::default()
    .rule_graph(graph)
    .language(PiranhaLanguage::from(TS_SCHEME))
    .code_snippet(pattern)
    .should_validate(false)
    .build();

  let output_summaries = execute_piranha(&piranha_arguments);

  // maps matched strings to a vec of strings
  let matched_strings = output_summaries
    .first()
    .map(|summary| {
      summary
        .matches()
        .iter()
        .map(|m| m.1.matched_string().clone())
        .collect::<Vec<String>>()
    })
    .unwrap_or_default();

  let re = Regex::new(r"@[\w_]+").unwrap();
  let capture_groups: Vec<String> = matched_strings
    .iter()
    .flat_map(|candidate_tag| re.find_iter(candidate_tag))
    .map(|mat| mat.as_str().to_owned())
    .collect();

  log::debug!("capture_groups: {capture_groups:?}");
  capture_groups
}
