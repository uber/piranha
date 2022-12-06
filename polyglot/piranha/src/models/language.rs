use getset::Getters;
use serde_derive::Deserialize;
use tree_sitter::{Parser, Query};

use crate::utilities::parse_toml;

use super::{
  default_configs::default_language,
  outgoing_edges::Edges,
  rule::Rules,
  scopes::{ScopeConfig, ScopeGenerator},
};

#[derive(Debug, Clone, Getters, PartialEq)]
pub struct PiranhaLanguage {
  #[get = "pub"]
  name: String,
  #[get = "pub"]
  supported_language: SupportedLanguage,
  #[get = "pub"]
  language: tree_sitter::Language,
  #[get = "pub(crate)"]
  rules: Option<Rules>,
  #[get = "pub(crate)"]
  edges: Option<Edges>,
  #[get = "pub(crate)"]
  scopes: Vec<ScopeGenerator>,
  #[get = "pub"]
  comment_nodes: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum SupportedLanguage {
  Java,
  Kotlin,
  Go,
  Swift,
  Strings,
  Ts,
  Tsx,
  Python,
}

impl Default for SupportedLanguage {
  fn default() -> Self {
    SupportedLanguage::Java
  }
}

impl PiranhaLanguage {
  pub fn is_comment(&self, kind: String) -> bool {
    self.comment_nodes().contains(&kind)
  }

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
}

impl Default for PiranhaLanguage {
  fn default() -> Self {
    let rules: Rules = parse_toml(include_str!("../cleanup_rules/java/rules.toml"));
    let edges: Edges = parse_toml(include_str!("../cleanup_rules/java/edges.toml"));
    PiranhaLanguage {
      name: default_language(),
      supported_language: SupportedLanguage::default(),
      language: tree_sitter_java::language(),
      rules: Some(rules),
      edges: Some(edges),
      scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/java/scope_config.toml"))
        .scopes()
        .to_vec(),
      comment_nodes: vec!["line_comment".to_string(), "block_comment".to_string()],
    }
  }
}

impl From<&str> for PiranhaLanguage {
  fn from(language: &str) -> Self {
    match language {
      "java" => PiranhaLanguage::default(),
      "go" => PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Go,
        language: tree_sitter_go::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      },
      "kt" => {
        let rules: Rules = parse_toml(include_str!("../cleanup_rules/kt/rules.toml"));
        let edges: Edges = parse_toml(include_str!("../cleanup_rules/kt/edges.toml"));
        PiranhaLanguage {
          name: language.to_string(),
          supported_language: SupportedLanguage::Kotlin,
          language: tree_sitter_kotlin::language(),
          rules: Some(rules),
          edges: Some(edges),
          scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/kt/scope_config.toml"))
            .scopes()
            .to_vec(),
          comment_nodes: vec!["comment".to_string()],
        }
      }
      "py" => PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Python,
        language: tree_sitter_python::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      },
      "swift" => PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Swift,
        language: tree_sitter_swift::language(),
        scopes: parse_toml::<ScopeConfig>(include_str!("../cleanup_rules/swift/scope_config.toml"))
          .scopes()
          .to_vec(),
        comment_nodes: vec!["comment".to_string(), "multiline_comment".to_string()],
        rules: None,
        edges: None,
      },
      "strings" => PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Strings,
        language: tree_sitter_strings::language(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      },
      "ts" => PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Ts,
        language: tree_sitter_typescript::language_typescript(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      },
      "tsx" => PiranhaLanguage {
        name: language.to_string(),
        supported_language: SupportedLanguage::Tsx,
        language: tree_sitter_typescript::language_tsx(),
        rules: None,
        edges: None,
        scopes: vec![],
        comment_nodes: vec![],
      },
      _ => panic!("Language not supported"),
    }
  }
}
