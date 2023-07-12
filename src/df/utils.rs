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

use std::str::FromStr;
use tree_sitter::{Node, Parser, Query};

use crate::models::default_configs::TS_SCHEME;
use crate::models::language::PiranhaLanguage;
use crate::models::piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder};
use crate::models::rule::Rule;
use crate::models::rule_graph::{RuleGraph, RuleGraphBuilder};
use crate::utilities::tree_sitter_utilities::{
  get_all_matches_for_query, get_node_for_range,
};
use crate::{execute_piranha};
use crate::models::rule::piranha_rule;
use crate::models::filter::filter;
use regex::Regex;
use std::string::String;

/// Find defined tags in a query
pub fn get_tags_from_matcher(node: &Rule) -> Vec<String> {
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
    .code_snippet(node.query().pattern())
    .build();

  let output_summaries = execute_piranha(&piranha_arguments);

  // maps matched strings to a vec of strings
  let tags = output_summaries[0]
    .matches()
    .iter()
    .map(|m| m.0.clone())
    .collect::<Vec<String>>();
  tags
}

/// Find all tags used in predicates
pub fn get_tags_usage_from_matcher(node: &Rule) -> Vec<String> {


  let rules = vec![piranha_rule! {
    name = "capture_groups",
    query = "(
    [ (capture) @cap
      (identifier) @id
      (string) @str]
    (#match? @cap \"@\")
    (#match? @id \"@\")
    (#match? @str \"@\")
    ",
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
      .code_snippet(node.query().pattern())
      .build();

  let output_summaries = execute_piranha(&piranha_arguments);


  // maps matched strings to a vec of strings
  let matched_strings = output_summaries[0]
      .matches()
      .iter()
      .map(|m| m.0.clone())
      .collect::<Vec<String>>();

  let mut tags = vec![];
  for mut candidate_tag in matched_strings {
    candidate_tag = candidate_tag.replace('\"', "");
    tags.push(candidate_tag);
  }
  tags
}
