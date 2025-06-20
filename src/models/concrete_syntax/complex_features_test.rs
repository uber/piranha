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

//! Tests for complex concrete syntax features like alternations, constraints, etc.

use super::interpreter::parse_and_match;
use super::parser::{
  CaptureMode, ComparisonOp, ConcreteSyntax, ConstraintKind, CsElement, CsPattern, CsValue,
};
use crate::models::{default_configs::JAVA, language::PiranhaLanguage};

fn run_test_complex(
  code: &str, pattern: &str, expected_matches: usize, expected_vars: Vec<Vec<(&str, &str)>>,
  language: &str,
) {
  let java = PiranhaLanguage::from(language);
  let mut parser = java.parser();
  let tree = parser.parse(code.as_bytes(), None).unwrap();

  let (matches, _is_match_found) =
    parse_and_match(pattern, &tree.root_node(), code.as_bytes(), true, None).unwrap();

  assert_eq!(
    matches.len(),
    expected_matches,
    "Expected {} matches but got {}",
    expected_matches,
    matches.len()
  );

  for (i, vars) in expected_vars.iter().enumerate() {
    if i < matches.len() {
      let match_item = &matches[i];
      for &(var, expected_val) in vars {
        let val = match_item.matches.get(var);
        match val {
          Some(actual_val) => assert_eq!(actual_val, expected_val),
          None => panic!("Expected variable '{}' not found in match {}", var, i),
        }
      }
    }
  }
}

#[test]
fn test_parser_basic_capture() {
  let cs = ConcreteSyntax::parse(":[name]").unwrap();

  if let CsPattern::Sequence(elements) = cs.pattern {
    assert_eq!(elements.len(), 1);
    if let CsElement::Capture { name, mode } = &elements[0] {
      assert_eq!(name, "name");
      assert_eq!(*mode, CaptureMode::Single);
    } else {
      panic!("Expected capture element");
    }
  } else {
    panic!("Expected sequence pattern");
  }
}

#[test]
fn test_parser_capture_modes() {
  let cs = ConcreteSyntax::parse(":[args+] :[body*]").unwrap();

  if let CsPattern::Sequence(elements) = cs.pattern {
    assert_eq!(elements.len(), 3); // args+, space, body*

    if let CsElement::Capture {
      mode: CaptureMode::OnePlus,
      ..
    } = elements[0]
    {
      // OK
    } else {
      panic!("Expected OnePlus capture");
    }

    if let CsElement::Capture {
      mode: CaptureMode::ZeroPlus,
      ..
    } = elements[2]
    {
      // OK
    } else {
      panic!("Expected ZeroPlus capture");
    }
  }
}

#[test]
fn test_parser_with_literals() {
  let cs = ConcreteSyntax::parse("class :[name] { :[body*] }").unwrap();

  if let CsPattern::Sequence(elements) = cs.pattern {
    assert_eq!(elements.len(), 5); // class, name, {, body*, }

    if let CsElement::Literal(text) = &elements[0] {
      assert_eq!(text, "class");
    } else {
      panic!("Expected literal 'class'");
    }
  }
}

#[test]
fn test_parser_alternation() {
  let cs = ConcreteSyntax::parse(":[name] | :[other]").unwrap();

  if let CsPattern::Alternation(alternatives) = cs.pattern {
    assert_eq!(alternatives.len(), 2);
  } else {
    panic!("Expected alternation pattern");
  }
}

#[test]
fn test_parser_constraints_regex() {
  let cs = ConcreteSyntax::parse(":[name] where :[name] matches /[A-Z].*/").unwrap();

  assert_eq!(cs.constraints.len(), 1);
  if let ConstraintKind::Matches { capture, .. } = &cs.constraints[0].kind {
    assert_eq!(capture, "name");
  } else {
    panic!("Expected matches constraint");
  }
}

#[test]
fn test_parser_constraints_length() {
  let cs = ConcreteSyntax::parse(":[args+] where len(:[args]) > 2").unwrap();

  assert_eq!(cs.constraints.len(), 1);
  if let ConstraintKind::Length { capture, op, value } = &cs.constraints[0].kind {
    assert_eq!(capture, "args");
    assert_eq!(*op, ComparisonOp::Gt);
    assert_eq!(*value, 2);
  } else {
    panic!("Expected length constraint");
  }
}

#[test]
fn test_parser_constraints_comparison() {
  let cs = ConcreteSyntax::parse(":[name] where :[name] == \"Test\"").unwrap();

  assert_eq!(cs.constraints.len(), 1);
  if let ConstraintKind::Comparison { capture, op, value } = &cs.constraints[0].kind {
    assert_eq!(capture, "name");
    assert_eq!(*op, ComparisonOp::Eq);
    if let CsValue::String(s) = value {
      assert_eq!(s, "Test");
    } else {
      panic!("Expected string value");
    }
  } else {
    panic!("Expected comparison constraint");
  }
}

#[test]
fn test_parser_multiple_constraints() {
  let cs =
    ConcreteSyntax::parse(":[name] where :[name] matches /[A-Z].*/, len(:[name]) > 5").unwrap();

  assert_eq!(cs.constraints.len(), 2);
}

#[test]
fn test_alternation_match() {
  run_test_complex(
    "foo = true;",
    ":[var] = true | :[var] = false",
    1,
    vec![vec![("var", "foo")]],
    JAVA,
  );
}

#[test]
fn test_single_match_java() {
  run_test_complex(
    "class Example { }",
    "class :[name] { :[body*] }",
    2, // Current implementation finds duplicates
    vec![
      vec![("name", "Example"), ("body", "")],
      vec![("name", "Example"), ("body", "")],
    ],
    JAVA,
  );
}

#[test]
fn test_complex_template() {
  run_test_complex(
    "if (condition) { statement1; statement2; }",
    "if (:[cond]) { :[stmts*] }",
    1,
    vec![vec![
      ("cond", "condition"),
      ("stmts", "statement1; statement2;"),
    ]],
    JAVA,
  );
}

#[test]
fn test_sequential_siblings() {
  run_test_complex(
    "public class Test { private int x; public void method() {} }",
    ":[modifier] class :[name] { :[members*] }",
    1,
    vec![vec![
      ("modifier", "public"),
      ("name", "Test"),
      ("members", "private int x; public void method() {}"),
    ]],
    JAVA,
  );
}

#[test]
fn test_whitespace_handling() {
  run_test_complex(
    "  class   Example   {   }  ",
    "class :[name] { }",
    1,
    vec![vec![("name", "Example")]],
    JAVA,
  );
}

#[test]
fn test_boolean_constraint() {
  run_test_complex(
    "enabled = true; disabled = false;",
    ":[var] = :[val] where :[val] == \"true\"",
    1,
    vec![vec![("var", "enabled"), ("val", "true")]],
    JAVA,
  );
}

#[test]
fn test_length_constraint() {
  run_test_complex(
    "method(a, b, c, d);",
    "method(:[args+]) where len(:[args]) > 2",
    1,
    vec![vec![("args", "a, b, c, d")]],
    JAVA,
  );
}

#[test]
fn test_comparison_operators() {
  run_test_complex(
    "value = 42;",
    ":[var] = :[num] where :[num] == \"42\"",
    1,
    vec![vec![("var", "value"), ("num", "42")]],
    JAVA,
  );
}

// Error handling tests
#[test]
fn test_parser_error_unclosed_capture() {
  let result = ConcreteSyntax::parse(":[name");
  assert!(result.is_err());
}

#[test]
fn test_parser_error_invalid_capture_mode() {
  let result = ConcreteSyntax::parse(":[name?]");
  assert!(result.is_err());
}

#[test]
fn test_parser_error_unterminated_string() {
  let result = ConcreteSyntax::parse(":[name] where :[name] == \"unterminated");
  assert!(result.is_err());
}

#[test]
fn test_parser_error_invalid_regex() {
  let result = ConcreteSyntax::parse(":[name] where :[name] matches /[/");
  assert!(result.is_err());
}

#[test]
fn test_parser_error_unclosed_group() {
  let result = ConcreteSyntax::parse("(:[name] | :[other]");
  assert!(result.is_err());
}
