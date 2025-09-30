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

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[grammar = "concrete_syntax.pest"]
pub struct ConcreteSyntaxParser;

/// AST nodes for the concrete syntax language

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcreteSyntax {
  pub pattern: CsPattern,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsPattern {
  pub sequence: Vec<CsElement>,
  pub constraints: Vec<CsConstraint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CsElement {
  Capture { name: String, mode: CaptureMode },
  Literal(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CsConstraint {
  In {
    capture: String,
    items: Vec<String>,
  },
  Regex {
    capture: String,
    pattern: String,
  },
  Type {
    target: String,
    types: Vec<String>,
  },
  Contains {
    target: String,
    pattern: Vec<CsElement>,
    #[serde(skip)] // Don't serialize the resolved pattern
    resolved_pattern: Option<Box<super::resolver::ResolvedConcreteSyntax>>,
  },
  Not(Box<CsConstraint>),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CaptureMode {
  Single,   // :[var]
  OnePlus,  // :[var+]
  ZeroPlus, // :[var*]
  Optional, // :[var?]
}

/// Decode \" \\ \n \t \/ \@ \: … inside a string literal.
fn unescape(src: &str) -> String {
  let mut out = String::with_capacity(src.len());
  let mut chars = src.chars();
  while let Some(c) = chars.next() {
    if c == '\\' {
      match chars.next() {
        Some('"') => out.push('"'),
        Some('\\') => out.push('\\'),
        Some('n') => out.push('\n'),
        Some('t') => out.push('\t'),
        Some('/') => out.push('/'),
        Some('@') => out.push('@'),
        Some(':') => out.push(':'),
        Some(other) => {
          out.push('\\');
          out.push(other);
        }
        None => break, // trailing back-slash
      }
    } else {
      out.push(c);
    }
  }
  out
}

impl ConcreteSyntax {
  pub fn parse(input: &str) -> Result<Self, String> {
    use Rule::*;

    let mut pairs = ConcreteSyntaxParser::parse(concrete_syntax, input)
      .map_err(|e| format!("Parse error: {e}"))?;

    let main_pair = pairs.next().unwrap();

    let mut parsed_pattern = None;

    for pair in main_pair.into_inner() {
      match pair.as_rule() {
        pattern => {
          parsed_pattern = Some(Self::parse_pattern(pair)?);
        }
        EOI => {} // End of input, ignore
        _ => {}
      }
    }

    Ok(ConcreteSyntax {
      pattern: parsed_pattern.ok_or("No pattern found")?,
    })
  }

  fn parse_pattern(pair: Pair<Rule>) -> Result<CsPattern, String> {
    let mut sequence = Vec::new();
    let mut constraints = Vec::new();

    for inner_pair in pair.into_inner() {
      match inner_pair.as_rule() {
        Rule::constraints => {
          constraints = Self::parse_constraints(inner_pair)?;
        }
        _ => {
          // This is an element (capture or literal) - parse_element now returns Vec<CsElement>
          let mut elements = Self::parse_element(inner_pair)?;
          sequence.append(&mut elements);
        }
      }
    }

    Ok(CsPattern {
      sequence,
      constraints,
    })
  }

  fn parse_constraints(pair: Pair<Rule>) -> Result<Vec<CsConstraint>, String> {
    let mut constraints = Vec::new();

    for constraint_pair in pair.into_inner() {
      constraints.push(Self::parse_constraint(constraint_pair)?);
    }

    Ok(constraints)
  }

  fn parse_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    use Rule::*;

    match pair.as_rule() {
      constraint => {
        // The constraint rule wraps the actual constraint type
        let inner = pair.into_inner().next().unwrap();
        Self::parse_constraint(inner)
      }
      in_constraint => Self::parse_in_constraint(pair),
      regex_constraint => Self::parse_regex_constraint(pair),
      type_constraint => Self::parse_type_constraint(pair),
      contains_constraint => Self::parse_contains_constraint(pair),
      _ => Err(format!("Unexpected constraint type: {:?}", pair.as_rule())),
    }
  }

  fn parse_in_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();

    // First should be the capture
    let capture_pair = inner.next().ok_or("Expected capture in in_constraint")?;
    let capture_name = Self::extract_capture_name(capture_pair)?;

    // Check for optional "not" keyword
    let mut negated = false;
    let mut items = Vec::new();

    for next_pair in inner {
      match next_pair.as_rule() {
        Rule::not_keyword => negated = true,
        Rule::list_items => items = Self::parse_list_items(next_pair)?,
        _ => continue, // Skip other tokens like "in"
      }
    }

    let base_constraint = CsConstraint::In {
      capture: capture_name,
      items,
    };

    if negated {
      Ok(CsConstraint::Not(Box::new(base_constraint)))
    } else {
      Ok(base_constraint)
    }
  }

  fn parse_regex_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();

    // First should be the capture
    let capture_pair = inner.next().ok_or("Expected capture in regex_constraint")?;
    let capture_name = Self::extract_capture_name(capture_pair)?;

    // Check for optional "not" keyword
    let mut negated = false;
    let mut pattern = String::new();

    for next_pair in inner {
      match next_pair.as_rule() {
        Rule::not_keyword => negated = true,
        Rule::regex_pattern => pattern = Self::parse_regex_pattern(next_pair)?,
        _ => continue, // Skip other tokens like "matches"
      }
    }

    let base_constraint = CsConstraint::Regex {
      capture: capture_name,
      pattern,
    };

    if negated {
      Ok(CsConstraint::Not(Box::new(base_constraint)))
    } else {
      Ok(base_constraint)
    }
  }

  fn parse_type_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();

    // First should be constraint_target
    let target_pair = inner
      .next()
      .ok_or("Expected constraint_target in type_constraint")?;
    let target_name = Self::extract_target_name(target_pair)?;

    // Parse the list items (node types)
    let mut types = Vec::new();

    for next_pair in inner {
      match next_pair.as_rule() {
        Rule::list_items => types = Self::parse_list_items(next_pair)?,
        _ => continue, // Skip other tokens like ".type", "in"
      }
    }

    Ok(CsConstraint::Type {
      target: target_name,
      types,
    })
  }

  fn parse_contains_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();

    // First should be constraint_target
    let target_pair = inner
      .next()
      .ok_or("Expected constraint_target in contains_constraint")?;
    let target_name = Self::extract_target_name(target_pair)?;

    // Check for optional "not" keyword and delimited_pattern
    let mut negated = false;
    let mut pattern_elements = Vec::new();

    for next_pair in inner {
      match next_pair.as_rule() {
        Rule::not_keyword => negated = true,
        Rule::delimited_pattern => pattern_elements = Self::parse_delimited_pattern(next_pair)?,
        _ => continue, // Skip other tokens like "contains"
      }
    }

    let base_constraint = CsConstraint::Contains {
      target: target_name,
      pattern: pattern_elements,
      resolved_pattern: None, // Initialize as None
    };

    if negated {
      Ok(CsConstraint::Not(Box::new(base_constraint)))
    } else {
      Ok(base_constraint)
    }
  }

  fn parse_delimited_pattern(pair: Pair<Rule>) -> Result<Vec<CsElement>, String> {
    // delimited_pattern contains sub_pattern
    let mut inner = pair.into_inner();
    let sub_pattern_pair = inner
      .next()
      .ok_or("Expected sub_pattern in delimited_pattern")?;

    let mut elements = Vec::new();
    for element_pair in sub_pattern_pair.into_inner() {
      let mut parsed_elements = Self::parse_element(element_pair)?;
      elements.append(&mut parsed_elements);
    }

    Ok(elements)
  }

  fn parse_regex_pattern(pair: Pair<Rule>) -> Result<String, String> {
    // The regex_pattern contains regex_content
    let mut inner = pair.into_inner();
    let content_pair = inner.next().ok_or("Expected regex content")?;

    // Use the existing unescape function for proper string unescaping
    let raw_content = content_pair.as_str();
    let unescaped = unescape(raw_content);

    Ok(unescaped)
  }

  fn extract_capture_name(pair: Pair<Rule>) -> Result<String, String> {
    match pair.as_rule() {
      Rule::capture => {
        // Parse the capture to get its name
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        Ok(identifier)
      }
      _ => Err(format!("Expected capture, got: {:?}", pair.as_rule())),
    }
  }

  fn extract_target_name(pair: Pair<Rule>) -> Result<String, String> {
    match pair.as_rule() {
      Rule::constraint_target => {
        // constraint_target contains either capture or root_keyword
        let mut inner = pair.into_inner();
        let target_pair = inner.next().unwrap();
        Self::extract_target_name(target_pair)
      }
      Rule::capture => {
        // Parse the capture to get its name
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        Ok(identifier)
      }
      Rule::root_keyword => Ok("root".to_string()),
      _ => Err(format!(
        "Expected constraint_target, capture, or root_keyword, got: {:?}",
        pair.as_rule()
      )),
    }
  }

  fn parse_list_items(pair: Pair<Rule>) -> Result<Vec<String>, String> {
    let mut items = Vec::new();

    for quoted_string_pair in pair.into_inner() {
      if quoted_string_pair.as_rule() == Rule::quoted_string {
        let raw = quoted_string_pair.as_str();
        // Remove surrounding quotes
        let unquoted = &raw[1..raw.len() - 1];
        let cooked = unescape(unquoted);
        items.push(cooked.to_string());
      }
    }

    Ok(items)
  }

  fn parse_literal_tokens(text: &str) -> Vec<CsElement> {
    let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
    tokens.into_iter().map(CsElement::Literal).collect()
  }

  fn parse_element(pair: Pair<Rule>) -> Result<Vec<CsElement>, String> {
    use Rule::*;

    match pair.as_rule() {
      capture => Ok(vec![Self::parse_capture_single(pair)?]),
      literal_text => {
        // Split the literal text on whitespace, similar to Python's .split()
        let text = pair.as_str();
        let unescaped_text = unescape(text);
        Ok(Self::parse_literal_tokens(&unescaped_text))
      }
      delimited_literal => {
        // Same as literal_text but with escape handling for \/
        let raw_text = pair.as_str();
        let unescaped_text = unescape(raw_text);
        Ok(Self::parse_literal_tokens(&unescaped_text))
      }
      _ => Err(format!("Unexpected element: {:?}", pair.as_rule())),
    }
  }

  fn parse_capture_single(pair: Pair<Rule>) -> Result<CsElement, String> {
    let mut inner = pair.into_inner();
    let identifier = inner.next().unwrap().as_str().to_string();

    let mode = if let Some(mode_pair) = inner.next() {
      match mode_pair.as_str() {
        "+" => CaptureMode::OnePlus,
        "*" => CaptureMode::ZeroPlus,
        "?" => CaptureMode::Optional,
        _ => return Err(format!("Unknown capture mode: {}", mode_pair.as_str())),
      }
    } else {
      CaptureMode::Single
    };

    Ok(CsElement::Capture {
      name: identifier,
      mode,
    })
  }
}

