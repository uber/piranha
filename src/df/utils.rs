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

use std::string::String;

/// Find defined tags in a query
pub fn get_tags_from_matcher(node: &Rule) -> Vec<String> {

  if node.query().pattern().is_empty() {
    return vec![];
  }

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
  let tags = output_summaries
    .get(0)
    .map(|summary| {
      summary
        .matches()
        .iter()
        .map(|m| m.1.matched_string().clone())
        .collect::<Vec<String>>()
    })
    .unwrap_or_else(Vec::new);

  println!("tags: {:?}", tags);

  tags
}

/// Find all tags used in predicates
pub fn get_tags_usage_from_matcher(node: &Rule) -> Vec<String> {

  if node.query().pattern().is_empty() {
    return vec![];
  }

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
    .code_snippet(node.query().pattern())
    .build();

  let output_summaries = execute_piranha(&piranha_arguments);

  // maps matched strings to a vec of strings
  let matched_strings = output_summaries
    .get(0)
    .map(|summary| {
      summary
        .matches()
        .iter()
        .map(|m| m.1.matched_string().clone())
        .collect::<Vec<String>>()
    })
    .unwrap_or_else(Vec::new);

  let mut tags = vec![];
  for mut candidate_tag in matched_strings {
    candidate_tag = candidate_tag.replace('\"', "");
    tags.push(candidate_tag);
  }

  println!("tags: {:?}", tags);
  tags

}
