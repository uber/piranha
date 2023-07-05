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

use std::collections::HashMap;

use itertools::Itertools;
use regex::Regex;
use tree_sitter::Node;

use crate::models::matches::Match;

/// Applies the query upon the given `node`, and gets the first match
/// # Arguments
/// * `node` - the root node to apply the query upon
/// * `source_code` - the corresponding source code string for the node.
/// * `query` - the query to be applied
/// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
///
/// # Returns
/// The range of the match in the source code and the corresponding mapping from tags to code snippets.
pub(crate) fn get_all_matches_for_query(
  node: &Node, source_code: String, regex: &Regex, recursive: bool, replace_node: Option<String>,
) -> Vec<Match> {
  //   let all_matches = regex.find_iter(&source_code).collect_vec();
  let all_captures = regex.captures_iter(&source_code).collect_vec();
  let names = regex.capture_names().collect_vec();

  for captures in all_captures {
    for m in captures.iter() {}
  }

  //   for mtch in all_matches {

  let matches = extract_captures(&all_captures, mtch, &names);

  let m = Match::from_regex(&mtch, matches, &source_code);

  //   }

  return vec![];
}

fn extract_captures(
  all_captures: &Vec<regex::Captures<'_>>, mtch: regex::Match<'_>, names: &Vec<Option<&str>>,
) -> HashMap<String, String> {
  all_captures
    .iter()
    .filter(|captures| captures[0].to_string() == mtch.as_str().to_string())
    .flat_map(|captures| {
      names.iter().flatten().flat_map(|x| {
        captures
          .name(x)
          .map(|v| (x.to_string(), v.as_str().to_string()))
      })
    })
    .collect()
}
