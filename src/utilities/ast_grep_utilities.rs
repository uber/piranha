use std::{collections::HashMap, hash::Hash};

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
use ast_grep_core::{language::TSLanguage, AstGrep, Matcher, Pattern, StrDoc};
use tree_sitter::Node;

use crate::models::matches::Match;

/// Applies the query upon the given `node`, and gets all the matches
/// # Arguments
/// * `node` - the root node to apply the query upon
/// * `source_code` - the corresponding source code string for the node.
/// * `recursive` - if `true` it matches the query to `self` and `self`'s sub-ASTs, else it matches the `query` only to `self`.
/// * `replace_node` - node to replace
///
/// # Returns
/// List containing all the matches against `node`
pub(crate) fn get_all_matches_for_ast_grep_pattern(
  node: &Node, source_code: String, pattern: &Pattern<StrDoc<TSLanguage>>, recursive: bool,
  replace_node: Option<String>, language: tree_sitter::Language,
) -> Vec<Match> {
  let x = AstGrep::new(&source_code, TSLanguage::from(language));

  let all_captures = x.root().find_all(pattern);
  let mut all_matches = vec![];
  for captures in all_captures {
    let range_matches_node =
      node.start_byte() == captures.range().start && node.end_byte() == captures.range().end;
    let range_matches_inside_node =
      node.start_byte() <= captures.range().start && node.end_byte() >= captures.range().end;
    if (recursive && range_matches_inside_node) || range_matches_node {
      let replace_node_match = if let Some(ref rn) = replace_node {
        captures
          .get_env()
          .get_match(rn)
          .unwrap_or_else(|| panic!("The tag {rn} provided in the replace node is not present"))
      } else {
        captures.get_node()
      };
      let matches = extract_captures(&captures);
      all_matches.push(Match::from_ast_grep_captures(
        &replace_node_match,
        matches,
        &source_code,
      ));
    }
  }
  vec![]
}

fn extract_captures(
  captures: &ast_grep_core::NodeMatch<'_, StrDoc<TSLanguage>>,
) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for v in captures.get_env().get_matched_variables() {
    let name = match v {
      ast_grep_core::meta_var::MetaVariable::Named(name, _) => Some(name),
      ast_grep_core::meta_var::MetaVariable::Anonymous(_) => None,
      ast_grep_core::meta_var::MetaVariable::Ellipsis => None,
      ast_grep_core::meta_var::MetaVariable::NamedEllipsis(name) => Some(name),
    };
    if let Some(n) = name {
      map.insert(
        n.to_string(),
        captures.get_env().get_match(&n).unwrap().text().to_string(),
      );
    }
  }
  return map;
}
