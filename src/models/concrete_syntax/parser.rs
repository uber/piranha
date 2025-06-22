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

impl ConcreteSyntax {
  pub fn parse(input: &str) -> Result<Self, String> {
    use Rule::*;

    let mut pairs = ConcreteSyntaxParser::parse(concrete_syntax, input)
      .map_err(|e| format!("Parse error: {}", e))?;

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
    let elements: Result<Vec<_>, _> = pair.into_inner().map(Self::parse_element).collect();
    Ok(CsPattern {
      sequence: elements?,
    })
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
}

#[cfg(test)]
#[path = "unit_tests/test_parser.rs"]
mod test_parser;
