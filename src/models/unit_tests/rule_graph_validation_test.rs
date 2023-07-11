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

use crate::models::{
  capture_group_patterns::CGPattern, filter::FilterBuilder, rule_graph::RuleGraphBuilder,
};
use crate::piranha_rule;

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `at_least` or `at_most` is set, but `contains` is empty !!!"
)]
fn test_filter_bad_arg_at_least() {
  FilterBuilder::default().at_least(2).build();
}

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `at_least` or `at_most` is set, but `contains` is empty !!!"
)]
fn test_filter_bad_arg_at_most() {
  FilterBuilder::default().at_least(5).build();
}

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `contains` and `not_contains` cannot be set at the same time !!! Please use two filters instead."
)]
fn test_filter_bad_arguments_contains_not_contains() {
  FilterBuilder::default()
    .contains(CGPattern::new(String::from("(if_statement) @if_stmt")))
    .not_contains(vec![CGPattern::new(String::from("(for_statement) @for"))])
    .build();
}

#[test]
#[should_panic(
  expected = "Invalid Filter Argument. `at_least` should be less than or equal to `at_most` !!!"
)]
fn test_filter_bad_range() {
  FilterBuilder::default()
    .contains(CGPattern::new(String::from("(if_statement) @if_stmt")))
    .at_least(5)
    .at_most(4)
    .build();
}

#[test]
#[should_panic(expected = "Cannot parse")]
fn test_filter_syntactically_incorrect_contains() {
  FilterBuilder::default()
    .contains(CGPattern::new(String::from("(if_statement @if_stmt")))
    .build();
}

#[test]
#[should_panic(expected = "Cannot parse")]
fn test_filter_syntactically_incorrect_not_contains() {
  FilterBuilder::default()
    .not_contains(vec![CGPattern::new(String::from("(if_statement @if_stmt"))])
    .build();
}

#[test]
#[should_panic(expected = "Cannot parse")]
fn test_filter_syntactically_incorrect_enclosing_node() {
  FilterBuilder::default()
    .enclosing_node(CGPattern::new(String::from("(if_statement @if_stmt")))
    .build();
}

#[test]
#[should_panic(expected = "Cannot parse")]
fn test_filter_syntactically_incorrect_not_enclosing_node() {
  FilterBuilder::default()
    .not_enclosing_node(CGPattern::new(String::from("(if_statement @if_stmt")))
    .build();
}

#[test]
#[should_panic(expected = "Cannot parse")]
fn test_rule_graph_incorrect_query() {
  RuleGraphBuilder::default()
    .rules(vec![
      piranha_rule! {name = "Test rule", query = "(if_statement"},
    ])
    .build();
}

#[test]
#[should_panic(
  expected = "The child/sibling count operator is not compatible with (not) enclosing node and (not) contains operator"
)]
fn test_filter_bad_arg_contains_n_children() {
  FilterBuilder::default()
    .enclosing_node(CGPattern::new("(method_declaration) @i".to_string()))
    .child_count(2)
    .build();
}

#[test]
#[should_panic(
  expected = "The child/sibling count operator is not compatible with (not) enclosing node and (not) contains operator"
)]
fn test_filter_bad_arg_contains_n_sibling() {
  FilterBuilder::default()
    .enclosing_node(CGPattern::new("(method_declaration) @i".to_string()))
    .sibling_count(2)
    .build();
}
