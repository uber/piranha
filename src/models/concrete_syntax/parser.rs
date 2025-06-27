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

#[derive(Parser)]
#[grammar = "models/concrete_syntax/concrete_syntax.pest"]
pub struct ConcreteSyntaxParser;

/// AST nodes for the concrete syntax language

#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteSyntax {
  pub pattern: CsPattern,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CsPattern {
  pub sequence: Vec<CsElement>,
  pub constraints: Vec<CsConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CsElement {
  Capture { name: String, mode: CaptureMode },
  Literal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CsConstraint {
  In { capture: String, items: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureMode {
  Single,   // :[var]
  OnePlus,  // :[var+]
  ZeroPlus, // :[var*]
}

/// Decode \" \\ \n \t … inside a string literal.
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
      _ => Err(format!("Unexpected constraint type: {:?}", pair.as_rule())),
    }
  }

  fn parse_in_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();

    // First should be the capture
    let capture_pair = inner.next().ok_or("Expected capture in in_constraint")?;
    let capture_name = Self::extract_capture_name(capture_pair)?;

    // Then we should have list_items (optional)
    let mut items = Vec::new();
    if let Some(list_items_pair) = inner.next() {
      items = Self::parse_list_items(list_items_pair)?;
    }

    Ok(CsConstraint::In {
      capture: capture_name,
      items,
    })
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

  fn parse_element(pair: Pair<Rule>) -> Result<Vec<CsElement>, String> {
    use Rule::*;

    match pair.as_rule() {
      capture => Ok(vec![Self::parse_capture_single(pair)?]),
      literal_text => {
        // Split the literal text on whitespace, similar to Python's .split()
        let text = pair.as_str().trim();
        let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();

        Ok(tokens.into_iter().map(CsElement::Literal).collect())
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

#[cfg(test)]
#[path = "unit_tests/parser_test.rs"]
mod parser_test;
