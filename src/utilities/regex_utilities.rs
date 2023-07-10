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

use crate::models::matches::Match;
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use tree_sitter::Node;

/// Applies the query upon the given `node`, and gets all the matches
/// # Arguments
/// * `node` - the root node to apply the query upon
/// * `source_code` - the corresponding source code string for the node.
/// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
/// * `replace_node` - node to replace
///
/// # Returns
/// List containing all the matches against `node`
pub(crate) fn get_all_matches_for_regex(
  node: &Node, source_code: String, regex: &Regex, recursive: bool, replace_node: Option<String>,
) -> Vec<Match> {
  let all_captures = regex.captures_iter(&source_code).collect_vec();
  let names = regex.capture_names().collect_vec();
  let mut all_matches = vec![];
  for captures in all_captures {
    // Check if the range of the self (node), and the range of outermost node captured by the query are equal.
    let range_matches_node = node.start_byte() == captures.get(0).unwrap().start()
      && node.end_byte() == captures.get(0).unwrap().end();
    let range_matches_inside_node = node.start_byte() <= captures.get(0).unwrap().start()
      && node.end_byte() >= captures.get(0).unwrap().end();
    if (recursive && range_matches_inside_node) || range_matches_node {
      let replace_node_match = if let Some(ref rn) = replace_node {
        captures
          .name(rn)
          .unwrap_or_else(|| panic!("The tag {rn} provided in the replace node is not present"))
      } else {
        captures.get(0).unwrap()
      };
      let matches = extract_captures(&captures, &names);
      all_matches.push(Match::from_regex(
        &replace_node_match,
        matches,
        &source_code,
      ));
    }
  }
  all_matches
}

// Creates a hashmap from the capture group(name) to the corresponding code snippet.
fn extract_captures(
  captures: &regex::Captures<'_>, names: &Vec<Option<&str>>,
) -> HashMap<String, String> {
  names
    .iter()
    .flatten()
    .flat_map(|x| {
      captures
        .name(x)
        .map(|v| (x.to_string(), v.as_str().to_string()))
    })
    .collect()
}
