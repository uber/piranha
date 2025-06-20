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

//! Concrete Syntax Parser V3
//!
//! This module provides a grammar-based parser for concrete syntax patterns using the Pest parser generator.
//! It supports advanced features like alternations, constraints, and proper AST representation.

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "models/concrete_syntax/concrete_syntax.pest"]
pub struct ConcreteSyntaxParser;

/// AST nodes for the concrete syntax language
#[derive(Debug, Clone, PartialEq)]
pub enum CsPattern {
  Alternation(Vec<CsPattern>),
  Sequence(Vec<CsElement>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CsElement {
  Capture { name: String, mode: CaptureMode },
  Literal(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureMode {
  Single,   // :[var]
  OnePlus,  // :[var+]
  ZeroPlus, // :[var*]
}

#[derive(Debug, Clone, PartialEq)]
pub struct CsConstraint {
  pub kind: ConstraintKind,
}

#[derive(Debug, Clone)]
pub enum ConstraintKind {
  Matches {
    capture: String,
    regex: Regex,
  },
  Length {
    capture: String,
    op: ComparisonOp,
    value: usize,
  },
  Comparison {
    capture: String,
    op: ComparisonOp,
    value: CsValue,
  },
}

impl PartialEq for ConstraintKind {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (
        ConstraintKind::Matches {
          capture: c1,
          regex: r1,
        },
        ConstraintKind::Matches {
          capture: c2,
          regex: r2,
        },
      ) => c1 == c2 && r1.as_str() == r2.as_str(),
      (
        ConstraintKind::Length {
          capture: c1,
          op: o1,
          value: v1,
        },
        ConstraintKind::Length {
          capture: c2,
          op: o2,
          value: v2,
        },
      ) => c1 == c2 && o1 == o2 && v1 == v2,
      (
        ConstraintKind::Comparison {
          capture: c1,
          op: o1,
          value: v1,
        },
        ConstraintKind::Comparison {
          capture: c2,
          op: o2,
          value: v2,
        },
      ) => c1 == c2 && o1 == o2 && v1 == v2,
      _ => false,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonOp {
  Eq,
  NotEq,
  Lt,
  Le,
  Gt,
  Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CsValue {
  String(String),
  Number(f64),
  Boolean(bool),
}

/// Complete concrete syntax expression with pattern and optional constraints
#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteSyntax {
  pub pattern: CsPattern,
  pub constraints: Vec<CsConstraint>,
}

impl ConcreteSyntax {
  /// Parse a concrete syntax string into an AST
  pub fn parse(input: &str) -> Result<Self, String> {
    use Rule::*;

    let mut pairs = ConcreteSyntaxParser::parse(concrete_syntax, input)
      .map_err(|e| format!("Parse error: {}", e))?;

    let main_pair = pairs.next().unwrap();

    let mut parsed_pattern = None;
    let mut constraints = Vec::new();

    for pair in main_pair.into_inner() {
      match pair.as_rule() {
        pattern => {
          parsed_pattern = Some(Self::parse_pattern(pair)?);
        }
        where_clause => {
          constraints = Self::parse_where_clause(pair)?;
        }
        EOI => {} // End of input, ignore
        _ => {}
      }
    }

    Ok(ConcreteSyntax {
      pattern: parsed_pattern.ok_or("No pattern found")?,
      constraints,
    })
  }

  fn parse_pattern(pair: Pair<Rule>) -> Result<CsPattern, String> {
    // The pattern rule contains an alternation
    let alternation_pair = pair.into_inner().next().unwrap();
    Self::parse_alternation(alternation_pair)
  }

  fn parse_alternation(pair: Pair<Rule>) -> Result<CsPattern, String> {
    let sequences: Result<Vec<_>, _> = pair.into_inner().map(Self::parse_sequence).collect();

    let sequences = sequences?;

    if sequences.len() == 1 {
      Ok(sequences.into_iter().next().unwrap())
    } else {
      Ok(CsPattern::Alternation(sequences))
    }
  }

  fn parse_sequence(pair: Pair<Rule>) -> Result<CsPattern, String> {
    let elements: Result<Vec<_>, _> = pair.into_inner().map(Self::parse_element).collect();

    Ok(CsPattern::Sequence(elements?))
  }

  fn parse_element(pair: Pair<Rule>) -> Result<CsElement, String> {
    use Rule::*;

    match pair.as_rule() {
      capture => Self::parse_capture(pair),
      literal_text => Ok(CsElement::Literal(pair.as_str().trim().to_string())),
      _ => Err(format!("Unexpected element: {:?}", pair.as_rule())),
    }
  }

  fn parse_capture(pair: Pair<Rule>) -> Result<CsElement, String> {
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

  fn parse_where_clause(pair: Pair<Rule>) -> Result<Vec<CsConstraint>, String> {
    let constraints: Result<Vec<_>, _> = pair.into_inner().map(Self::parse_constraint).collect();

    constraints
  }

  fn parse_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    use Rule::*;

    match pair.as_rule() {
      length_constraint => Self::parse_length_constraint(pair),
      matches_constraint => Self::parse_matches_constraint(pair),
      comparison_constraint => Self::parse_comparison_constraint(pair),
      _ => Err(format!("Unknown constraint type: {:?}", pair.as_rule())),
    }
  }

  fn parse_length_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();
    let capture_ref = inner.next().unwrap();
    let capture_name = Self::parse_capture_ref(capture_ref)?;
    let op = Self::parse_comparison_op(inner.next().unwrap())?;
    let value = Self::parse_number(inner.next().unwrap())? as usize;

    Ok(CsConstraint {
      kind: ConstraintKind::Length {
        capture: capture_name,
        op,
        value,
      },
    })
  }

  fn parse_matches_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();
    let capture_ref = inner.next().unwrap();
    let capture_name = Self::parse_capture_ref(capture_ref)?;
    let regex_pair = inner.next().unwrap();
    let regex_str = Self::parse_regex(regex_pair)?;
    let regex = Regex::new(&regex_str).map_err(|e| format!("Invalid regex: {}", e))?;

    Ok(CsConstraint {
      kind: ConstraintKind::Matches {
        capture: capture_name,
        regex,
      },
    })
  }

  fn parse_comparison_constraint(pair: Pair<Rule>) -> Result<CsConstraint, String> {
    let mut inner = pair.into_inner();
    let capture_ref = inner.next().unwrap();
    let capture_name = Self::parse_capture_ref(capture_ref)?;
    let op = Self::parse_comparison_op(inner.next().unwrap())?;
    let value = Self::parse_value(inner.next().unwrap())?;

    Ok(CsConstraint {
      kind: ConstraintKind::Comparison {
        capture: capture_name,
        op,
        value,
      },
    })
  }

  fn parse_capture_ref(pair: Pair<Rule>) -> Result<String, String> {
    let identifier = pair.into_inner().next().unwrap();
    Ok(identifier.as_str().to_string())
  }

  fn parse_comparison_op(pair: Pair<Rule>) -> Result<ComparisonOp, String> {
    match pair.as_str() {
      ">=" => Ok(ComparisonOp::Ge),
      "<=" => Ok(ComparisonOp::Le),
      "==" => Ok(ComparisonOp::Eq),
      "!=" => Ok(ComparisonOp::NotEq),
      ">" => Ok(ComparisonOp::Gt),
      "<" => Ok(ComparisonOp::Lt),
      _ => Err(format!("Unknown comparison operator: {}", pair.as_str())),
    }
  }

  fn parse_value(pair: Pair<Rule>) -> Result<CsValue, String> {
    use Rule::*;

    match pair.as_rule() {
      string => Ok(CsValue::String(Self::parse_string(pair)?)),
      number => Ok(CsValue::Number(Self::parse_number(pair)?)),
      boolean => Ok(CsValue::Boolean(Self::parse_boolean(pair)?)),
      _ => Err(format!("Unknown value type: {:?}", pair.as_rule())),
    }
  }

  fn parse_string(pair: Pair<Rule>) -> Result<String, String> {
    let full_text = pair.as_str();
    // Remove the leading and trailing quote delimiters
    if full_text.len() >= 2 && full_text.starts_with('"') && full_text.ends_with('"') {
      Ok(full_text[1..full_text.len() - 1].to_string())
    } else {
      Err(format!("Invalid string format: {}", full_text))
    }
  }

  fn parse_number(pair: Pair<Rule>) -> Result<f64, String> {
    pair
      .as_str()
      .parse()
      .map_err(|_| format!("Invalid number: {}", pair.as_str()))
  }

  fn parse_boolean(pair: Pair<Rule>) -> Result<bool, String> {
    match pair.as_str() {
      "true" => Ok(true),
      "false" => Ok(false),
      _ => Err(format!("Invalid boolean: {}", pair.as_str())),
    }
  }

  fn parse_regex(pair: Pair<Rule>) -> Result<String, String> {
    let full_text = pair.as_str();
    // Remove the leading and trailing "/" delimiters
    if full_text.len() >= 2 && full_text.starts_with('/') && full_text.ends_with('/') {
      Ok(full_text[1..full_text.len() - 1].to_string())
    } else {
      Err(format!("Invalid regex format: {}", full_text))
    }
  }
}
