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

use std::{path::PathBuf, str::FromStr};

use getset::Getters;
use serde_derive::Deserialize;
use tree_sitter::{Parser, Query};

use crate::utilities::parse_toml;

use super::{
  default_configs::{
    default_language, GO, JAVA, JAVA_CS, KOTLIN, PYTHON, SCALA, STRINGS, SWIFT, THRIFT, TSX,
    TS_SCHEME, TYPESCRIPT, RUBY,
  },
  outgoing_edges::Edges,
  rule::Rules,
  scopes::{ScopeConfig, ScopeGenerator},
};

#[derive(Debug, Clone, Getters, PartialEq)]
pub struct PiranhaLanguage {
  /// The extension of the language FIXME: - https://github.com/uber/piranha/issues/365
  #[get = "pub"]
  extension: String,
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
  Thrift,
  Strings,
  TsScheme,
  Scala,
  Ruby,
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

  pub(crate) fn can_parse(&self, path: &PathBuf) -> bool {
    path
      .extension()
      .and_then(|e| e.to_str().filter(|x| x.eq(&self.extension())))
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
          extension: language.to_string(),
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
        })
      }
      JAVA_CS => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/java_cs/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/java_cs/edges.toml"));
        Ok(Self {
          extension: language.to_string(),
          supported_language: SupportedLanguage::Java,
          language: tree_sitter_java::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!(
            "../cleanup_rules/java_cs/scope_config.toml"
          ))
          .scopes()
          .to_vec(),
          comment_nodes: vec!["line_comment".to_string(), "block_comment".to_string()],
        })
      }
      GO => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/go/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/go/edges.toml"));
        Ok(PiranhaLanguage {
          extension: language.to_string(),
          supported_language: SupportedLanguage::Go,
          language: tree_sitter_go::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/go/scope_config.toml"))
            .scopes()
            .to_vec(),
          comment_nodes: vec!["comment".to_string()],
        })
      }
      KOTLIN => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/kt/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/kt/edges.toml"));
        Ok(PiranhaLanguage {
          extension: language.to_string(),
          supported_language: SupportedLanguage::Kotlin,
          language: tree_sitter_kotlin::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/kt/scope_config.toml"))
            .scopes()
            .to_vec(),
          comment_nodes: vec![
            "comment".to_string(),
            "line_comment".to_string(),
            "multiline_comment".to_string(),
          ],
        })
      }
      PYTHON => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::Python,
        language: tree_sitter_python::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      }),
      SWIFT => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/swift/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/swift/edges.toml"));
        Ok(PiranhaLanguage {
          extension: language.to_string(),
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
        })
      }
      TYPESCRIPT => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::Ts,
        language: tree_sitter_typescript::language_typescript(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      }),
      TSX => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::Tsx,
        language: tree_sitter_typescript::language_tsx(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      }),
      THRIFT => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::Thrift,
        language: tree_sitter_thrift::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      }),
      STRINGS => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::Strings,
        language: tree_sitter_strings::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      }),
      TS_SCHEME => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::TsScheme,
        language: tree_sitter_query::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      }),
      SCALA => Ok(PiranhaLanguage {
        extension: language.to_string(),
        supported_language: SupportedLanguage::Scala,
        language: tree_sitter_scala::language(),
        rules: None,
        edges: None,
        scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/scala/scope_config.toml"))
          .scopes()
          .to_vec(),
        comment_nodes: vec![],
      }),
      RUBY => {
        let ruby_rules: Rules = parse_toml(include_str!("../cleanup_rules/ruby/rules.toml"));
        let ruby_edges: Edges = parse_toml(include_str!("../cleanup_rules/ruby/edges.toml"));
        Ok(PiranhaLanguage {
          extension: language.to_string(),
          supported_language: SupportedLanguage::Ruby,
          language: tree_sitter_ruby::language(),
          scopes: parse_toml::<ScopeConfig>(include_str!(
            "../cleanup_rules/ruby/scope_config.toml"
          ))
          .scopes()
          .to_vec(),          
          comment_nodes: vec![],
          rules: Some(ruby_rules),
          edges: Some(ruby_edges),
        })
      }     
      _ => Err("Language not supported"),
    }
  }
}
