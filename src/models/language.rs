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

use std::str::FromStr;

use getset::Getters;
use serde_derive::Deserialize;
use tree_sitter::{Node, Parser, Query, Range};
use tree_sitter_traversal::{traverse, Order};

use crate::utilities::parse_toml;

use super::{
  default_configs::{default_language, GO, JAVA, KOTLIN, PYTHON, SWIFT, TSX, TYPESCRIPT},
  edit::Edit,
  outgoing_edges::Edges,
  rule::Rules,
  scopes::{ScopeConfig, ScopeGenerator},
  source_code_unit::SourceCodeUnit,
};

#[derive(Debug, Clone, Getters, PartialEq)]
pub struct PiranhaLanguage {
  /// The extension of the language FIXME: - https://github.com/uber/piranha/issues/365
  #[get = "pub"]
  name: String,
  /// the language (enum)
  #[get = "pub"]
  supported_language: SupportedLanguage,
  /// the language (As tree sitter model)
  #[get = "pub"]
  language: tree_sitter::Language,
  /// Built-in rules for the language
  #[get = "pub(crate)"]
  rules: Option<Rules>,
  /// Built-in edges for the language
  #[get = "pub(crate)"]
  edges: Option<Edges>,
  /// Scope configurations for the language
  #[get = "pub(crate)"]
  scopes: Vec<ScopeGenerator>,
  /// The node kinds to be considered when searching for comments
  #[get = "pub"]
  comment_nodes: Vec<String>,
  /// The node kinds to be ignored when searching for comments
  #[get = "pub"]
  ignore_nodes_for_comments: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub enum SupportedLanguage {
  #[default]
  Java,
  Kotlin,
  Go,
  Swift,
  Ts,
  Tsx,
  Python,
}

impl PiranhaLanguage {
  pub fn create_query(&self, query_str: String) -> Query {
    let query = Query::new(self.language, query_str.as_str());
    if let Ok(q) = query {
      return q;
    }
    panic!(
      "Could not parse the query : {:?} {:?}",
      query_str,
      query.err()
    );
  }

  pub fn parser(&self) -> Parser {
    let mut parser = Parser::new();
    parser
      .set_language(self.language)
      .expect("Could not set the language for the parser.");
    parser
  }

  pub(crate) fn can_parse(&self, de: &jwalk::DirEntry<((), ())>) -> bool {
    de.path()
      .extension()
      .and_then(|e| e.to_str().filter(|x| x.eq(&self.name())))
      .is_some()
  }

  #[cfg(test)]
  pub(crate) fn set_scopes(&mut self, scopes: Vec<ScopeGenerator>) {
    self.scopes = scopes;
  }
}

impl Default for PiranhaLanguage {
  fn default() -> Self {
    PiranhaLanguage::from(default_language().as_str())
  }
}

impl From<&str> for PiranhaLanguage {
  fn from(language: &str) -> Self {
    PiranhaLanguage::from_str(language).unwrap()
  }
}

impl std::str::FromStr for PiranhaLanguage {
  type Err = &'static str;
  /// This method is leveraged by `clap` to parse the command line
  /// argument into PiranhaLanguage
  fn from_str(language: &str) -> Result<Self, Self::Err> {
    match language {
      JAVA => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/java/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/java/edges.toml"));
        Ok(Self {
          name: language.to_string(),
          supported_language: SupportedLanguage::Java,
          language: tree_sitter_java::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!(
            "../cleanup_rules/java/scope_config.toml"
          ))
          .scopes()
          .to_vec(),
          comment_nodes: vec!["line_comment".to_string(), "block_comment".to_string()],
          ignore_nodes_for_comments: vec![],
        })
      }
      GO => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/go/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/go/edges.toml"));
        Ok(PiranhaLanguage {
          name: language.to_string(),
          supported_language: SupportedLanguage::Go,
          language: tree_sitter_go::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/go/scope_config.toml"))
            .scopes()
            .to_vec(),
          comment_nodes: vec!["comment".to_string()],
          ignore_nodes_for_comments: vec!["block".to_string(), "statement_list".to_string()],
        })
      }
      KOTLIN => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/kt/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/kt/edges.toml"));
        Ok(PiranhaLanguage {
          name: language.to_string(),
          supported_language: SupportedLanguage::Kotlin,
          language: tree_sitter_kotlin::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/kt/scope_config.toml"))
            .scopes()
            .to_vec(),
          comment_nodes: vec!["comment".to_string()],
          ignore_nodes_for_comments: vec![],
        })
      }
      PYTHON => Ok(PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Python,
        language: tree_sitter_python::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
        ignore_nodes_for_comments: vec![],
      }),
      SWIFT => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/swift/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/swift/edges.toml"));
        Ok(PiranhaLanguage {
          name: language.to_string(),
          supported_language: SupportedLanguage::Swift,
          language: tree_sitter_swift::language(),
          scopes: parse_toml::<ScopeConfig>(include_str!(
            "../cleanup_rules/swift/scope_config.toml"
          ))
          .scopes()
          .to_vec(),
          comment_nodes: vec!["comment".to_string(), "multiline_comment".to_string()],
          rules: Some(rules),
          edges: Some(edges),
          ignore_nodes_for_comments: vec![],
        })
      }
      TYPESCRIPT => Ok(PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Ts,
        language: tree_sitter_typescript::language_typescript(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
        ignore_nodes_for_comments: vec![],
      }),
      TSX => Ok(PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Tsx,
        language: tree_sitter_typescript::language_tsx(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
        ignore_nodes_for_comments: vec![],
      }),
      _ => Err("Language not supported"),
    }
  }
}

impl SourceCodeUnit {
  /// Checks if the given node kind is a comment in the language (i.e. &self)
  pub(crate) fn is_comment(&self, kind: String) -> bool {
    self
      .piranha_arguments()
      .language()
      .comment_nodes()
      .contains(&kind)
  }

  /// Checks if the given node should be ignored when searching comments
  pub(crate) fn should_ignore_node_for_comment_search(&self, node: Node) -> bool {
    node.end_byte() - node.start_byte() == 1
      || self
        .piranha_arguments()
        .language()
        .ignore_nodes_for_comments()
        .contains(&node.kind().to_string())
  }

  /// This function reports the range of the comment associated to the deleted element.
  ///
  /// # Arguments:
  /// * delete_edit : The edit that deleted the element
  /// * buffer: Number of lines that we want to look up to find associated comment
  ///
  /// # Algorithm :
  /// Get all the nodes that either start and end at [row]
  /// If **all** nodes are comments
  /// * return the range of the comment
  /// If the [row] has no node that either starts/ends there:
  /// * recursively call this method for [row] -1 (until buffer is positive)
  /// This function reports the range of the comment associated to the deleted element.
  ///
  /// # Arguments:
  /// * row : The row number where the deleted element started
  /// * buffer: Number of lines that we want to look up to find associated comment
  ///
  /// # Algorithm :
  /// Get all the nodes that either start and end at [row]
  /// If **all** nodes are comments
  /// * return the range of the comment
  /// If the [row] has no node that either starts/ends there:
  /// * recursively call this method for [row] -1 (until buffer is positive)
  pub(crate) fn _get_nearest_comment_range(
    &mut self, delete_edit: &Edit, buffer: usize,
  ) -> Option<Range> {
    let start_byte = delete_edit.p_match().range().start_byte;
    let row = delete_edit.p_match().range().start_point.row - buffer;
    // Get all nodes that start or end on `updated_row`.
    let mut relevant_nodes_found = false;
    let mut relevant_nodes_are_comments = true;
    let mut comment_range = None;
    // Since the previous edit was a delete, the start and end of the replacement range is [start_byte].
    let node = self
      .root_node()
      .descendant_for_byte_range(start_byte, start_byte)
      .unwrap_or_else(|| self.root_node());

    // Search for all nodes starting/ending in the current row.
    // if it is not amongst the `language.ignore_node_for_comment`, then check if it is a comment
    // If the node is a comment, then return its range
    for node in traverse(node.walk(), Order::Post) {
      if self.should_ignore_node_for_comment_search(node) {
        continue;
      }

      if node.start_position().row == row || node.end_position().row == row {
        relevant_nodes_found = true;
        let is_comment: bool = self.is_comment(node.kind().to_string());
        relevant_nodes_are_comments = relevant_nodes_are_comments && is_comment;
        if is_comment {
          comment_range = Some(node.range());
        }
      }
    }

    if relevant_nodes_found {
      if relevant_nodes_are_comments {
        return comment_range;
      }
    } else if buffer <= *self.piranha_arguments().cleanup_comments_buffer() {
      // We pass [start_byte] itself, because we know that parent of the current row is the parent of the above row too.
      // If that's not the case, its okay, because we will not find any comments in these scenarios.
      return self._get_nearest_comment_range(delete_edit, buffer + 1);
    }
    None
  }
}
