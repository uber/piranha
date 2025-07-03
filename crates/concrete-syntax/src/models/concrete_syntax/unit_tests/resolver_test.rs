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
use crate::models::concrete_syntax::parser::*;
use crate::models::concrete_syntax::resolver::*;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::concrete_syntax::parser::CsPattern;

  #[test]
  fn test_resolve_with_constraints() {
    let concrete = ConcreteSyntax {
      pattern: CsPattern {
        sequence: vec![CsElement::Capture {
          name: "var".to_string(),
          mode: CaptureMode::OnePlus,
        }],
        constraints: vec![CsConstraint::In {
          capture: "var".to_string(),
          items: vec!["a".to_string(), "b".to_string()],
        }],
      },
    };

    let resolved = concrete.resolve().unwrap();

    assert_eq!(resolved.pattern.sequence.len(), 1);
    match &resolved.pattern.sequence[0] {
      ResolvedCsElement::Capture {
        name,
        mode,
        constraints,
      } => {
        assert_eq!(name, "var");
        assert_eq!(*mode, CaptureMode::OnePlus);
        assert_eq!(constraints.len(), 1);
        match &constraints[0] {
          CsConstraint::In { capture, items } => {
            assert_eq!(capture, "var");
            assert_eq!(items, &vec!["a".to_string(), "b".to_string()]);
          }
        }
      }
      _ => panic!("Expected capture"),
    }
  }

  #[test]
  fn test_resolve_without_constraints() {
    let concrete = ConcreteSyntax {
      pattern: CsPattern {
        sequence: vec![
          CsElement::Capture {
            name: "var".to_string(),
            mode: CaptureMode::Single,
          },
          CsElement::Literal("test".to_string()),
        ],
        constraints: vec![],
      },
    };

    let resolved = concrete.resolve().unwrap();

    assert_eq!(resolved.pattern.sequence.len(), 2);
    match &resolved.pattern.sequence[0] {
      ResolvedCsElement::Capture {
        name,
        mode,
        constraints,
      } => {
        assert_eq!(name, "var");
        assert_eq!(*mode, CaptureMode::Single);
        assert_eq!(constraints.len(), 0);
      }
      _ => panic!("Expected capture"),
    }

    match &resolved.pattern.sequence[1] {
      ResolvedCsElement::Literal(text) => {
        assert_eq!(text, "test");
      }
      _ => panic!("Expected literal"),
    }
  }

  #[test]
  fn test_resolve_unresolved_constraint() {
    let concrete = ConcreteSyntax {
      pattern: CsPattern {
        sequence: vec![CsElement::Capture {
          name: "var".to_string(),
          mode: CaptureMode::Single,
        }],
        constraints: vec![CsConstraint::In {
          capture: "nonexistent".to_string(),
          items: vec!["a".to_string()],
        }],
      },
    };

    let result = concrete.resolve();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unresolved constraints"));
  }
}
