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

use crate::models::concrete_syntax::interpreter::get_all_matches_for_concrete_syntax;
use crate::models::concrete_syntax::parser::ConcreteSyntax;
use crate::models::concrete_syntax::resolver::resolve_concrete_syntax;
use crate::models::concrete_syntax::tree_sitter_adapter::TreeSitterAdapter;

// External tree-sitter language parsers
extern crate tree_sitter_go;
extern crate tree_sitter_java;
extern crate tree_sitter_kotlin;
extern crate tree_sitter_python;
extern crate tree_sitter_ruby;
extern crate tree_sitter_swift;
extern crate tree_sitter_typescript;

// Language constants
const JAVA: &str = "java";
const KOTLIN: &str = "kotlin";
const GO: &str = "go";
const PYTHON: &str = "python";
const SWIFT: &str = "swift";
const RUBY: &str = "ruby";
const TYPESCRIPT: &str = "typescript";

/// Run a concrete syntax pattern matching test
fn run_test(
  source_code: &str, pattern: &str, expected_count: usize,
  expected_captures: Vec<Vec<(&str, &str)>>, language: &str,
) {
  // Parse the pattern
  let concrete_syntax = ConcreteSyntax::parse(pattern)
    .unwrap_or_else(|e| panic!("Failed to parse pattern '{pattern}': {e}"));

  let resolved = resolve_concrete_syntax(&concrete_syntax);

  // Set up tree-sitter parser for the specified language
  let mut parser = tree_sitter::Parser::new();

  match language {
    JAVA => {
      parser
        .set_language(tree_sitter_java::language())
        .expect("Error loading Java grammar");
    }
    KOTLIN => {
      parser
        .set_language(tree_sitter_kotlin::language())
        .expect("Error loading Kotlin grammar");
    }
    GO => {
      parser
        .set_language(tree_sitter_go::language())
        .expect("Error loading Go grammar");
    }
    PYTHON => {
      parser
        .set_language(tree_sitter_python::language())
        .expect("Error loading Python grammar");
    }
    SWIFT => {
      parser
        .set_language(tree_sitter_swift::language())
        .expect("Error loading Swift grammar");
    }
    RUBY => {
      parser
        .set_language(tree_sitter_ruby::language())
        .expect("Error loading Ruby grammar");
    }
    TYPESCRIPT => {
      parser
        .set_language(tree_sitter_typescript::language_typescript())
        .expect("Error loading TypeScript grammar");
    }
    _ => panic!("Unsupported language: {language}"),
  }

  // Parse the source code
  let tree = parser
    .parse(source_code, None)
    .unwrap_or_else(|| panic!("Failed to parse source code"));

  let root_node = tree.root_node();
  let wrapped_node = TreeSitterAdapter::wrap_node(root_node);
  let source_bytes = source_code.as_bytes();

  // Run pattern matching
  let matches = get_all_matches_for_concrete_syntax(
    &wrapped_node,
    source_bytes,
    &resolved,
    true, // recursive
    None, // replace_node
  );

  // Validate match count
  assert_eq!(
    matches.len(),
    expected_count,
    "Expected {} matches for pattern '{}' in '{}', but got {}.\nActual matches: {:#?}",
    expected_count,
    pattern,
    source_code.replace('\n', "\\n"),
    matches.len(),
    matches
  );

  // Validate captures if matches were expected
  if expected_count > 0 && !expected_captures.is_empty() {
    for (i, (actual_match, expected_capture_set)) in
      matches.iter().zip(expected_captures.iter()).enumerate()
    {
      for (capture_name, expected_value) in expected_capture_set.iter() {
        let actual_value = actual_match.matches.get(*capture_name).unwrap_or_else(|| {
          panic!(
            "Match {} missing expected capture '{}'. Available captures: {:?}",
            i,
            capture_name,
            actual_match.matches.keys().collect::<Vec<_>>()
          )
        });

        let actual_text = actual_value.text.trim();
        assert_eq!(
          actual_text, *expected_value,
          "Match {i} capture '{capture_name}': expected '{expected_value}', got '{actual_text}'"
        );
      }
    }
  }
}

#[test]
fn test_single_match_kotlin() {
  run_test(
    "class Something : BaseParameters(namespace = \"data\") {
              val something_else: Boolean by parameter(\"something_else\")
          }",
    "val :[stale_property_name] : Boolean by parameter(\"something_else\")",
    1,
    vec![vec![("stale_property_name", "something_else")]],
    KOTLIN,
  );
}

#[test]
fn test_single_match() {
  run_test(
    "class Example { public int a = 10; }",
    "public int :[name] = :[value];",
    1,
    vec![vec![("name", "a"), ("value", "10")]],
    JAVA,
  );
}

#[test]
fn test_optional_match_single() {
  run_test(
    "package something; func (r *receiver) someName() { }",
    "func :[receiver?] someName() { }",
    1,
    vec![vec![("receiver", "(r *receiver)")]],
    GO,
  );
}

#[test]
fn test_optional_match_zero() {
  run_test(
    "package something; func someName() { }",
    "func :[receiver?] someName() { }",
    1,
    vec![vec![("receiver", "")]],
    GO,
  );
}

#[test]
fn test_optional_no_match() {
  run_test(
    "package something; func (r *receiver) someName() { }",
    "func :[receiver_and_name?]() { }",
    0,
    vec![],
    GO,
  );
}

#[test]
fn test_multiple_match() {
  run_test(
    "class Example { public int a = 10; public int b = 20; }",
    "public int :[name] = :[value];",
    2,
    vec![
      vec![("name", "a"), ("value", "10")],
      vec![("name", "b"), ("value", "20")],
    ],
    JAVA,
  );
}

#[test]
fn test_no_match() {
  run_test(
    "class Example { public int a = 10; }",
    "public String :[name] = :[value];",
    0,
    vec![],
    JAVA,
  );
}

#[test]
fn test_trailing_comma() {
  run_test(
    "a.foo(x, // something about the first argument
           y, // something about the second argument
           );",
    ":[var].foo(:[arg1], :[arg2+])",
    2,
    vec![vec![("var", "a"), ("arg1", "x"), ("arg2", "y,")]],
    GO,
  );
}

#[test]
fn test_sequential_siblings_matching() {
  run_test(
    "a.foo(x, y, z);",
    ":[var].foo(:[arg1+], z)",
    2,
    vec![vec![("var", "a"), ("arg1", "x, y")]],
    GO,
  );
}

#[test]
fn test_sequential_siblings_stmts() {
  // Find all usages of foo, whose last element is z.
  run_test(
    "{ int x = 2; x = x + 1; while(x > 0) { x = x - 1} } ",
    "int :[stmt1] = 2; \
            :[stmt2] = :[stmt2] + 1;",
    1,
    vec![vec![("stmt1", "x"), ("stmt2", "x")]],
    JAVA,
  );
}

#[test]
fn test_sequential_siblings_stmts2() {
  // Find all usages of foo, whose last element is z.
  run_test(
    "x.foo(1,2,3,4);",
    ":[var].foo(:[args+]);",
    2,
    vec![vec![("var", "x"), ("args", "1,2,3,4")]],
    JAVA,
  );
}

#[test]
fn test_complex_template() {
  // Test matching the given code against the template
  run_test(
    "void main() {
    // Some comment
    int some = 0;
    while(some < 100) {
        float length = 3.14;
        float area = length * length;
        some++;
    }}",
    "int :[var] = 0;
    while(:[var] < 100) {
      :[body+]
      :[var] ++;
    }",
    1,
    vec![vec![
      ("var", "some"),
      (
        "body",
        "float length = 3.14;\n        float area = length * length;",
      ),
    ]],
    JAVA,
  );
}

#[test]
fn test_match_anything() {
  // Test matching the given code against the template
  run_test(
    "public static void main(String args) {  }",
    ":[x]",
    1,
    vec![vec![("x", "public static void main(String args) {  }")]],
    JAVA,
  );
}

#[test]
fn test_asterisk_zero_or_more() {
  // Test asterisk (zero or more) matching
  run_test(
    "class Example { }",
    "class :[name] { :[body*] }",
    2,
    vec![vec![("name", "Example"), ("body", "")]],
    JAVA,
  );
}

#[test]
fn test_asterisk_one_or_more() {
  // Test asterisk (zero or more) matching with actual content
  run_test(
    "import java.util.ArrayList; class Example { int x = 1; int y = 2; }",
    "class :[name] { :[body*] }",
    1,
    vec![vec![("name", "Example"), ("body", "int x = 1; int y = 2;")]],
    JAVA,
  );
}

#[test]
fn test_simple_where_clause() {
  run_test(
    r#"foo(ab); foo("cd"); foo(1,2,3);"#,
    r#"foo(:[args]) |> :[args] in ["ab", "\"cd\""]"#,
    2, // now two matches
    vec![
      vec![("args", "ab")],
      vec![("args", "\"cd\"")], // note the quotes are included
    ],
    JAVA,
  );
}

#[test]
fn test_mutliple_where_clause() {
  run_test(
    r#"foo(c, d); foo(1,4);"#,
    r#"foo(:[arg1], :[arg2]) |> :[arg1] in ["1"], :[arg2] in ["4"]"#,
    1, // now two matches
    vec![vec![("arg1", "1"), ("arg2", "4")]],
    JAVA,
  );
}

#[test]
fn test_regex_constraint() {
  run_test(
    r#"
    public String getName() { return name; }
    public void setName(String n) { name = n; }
    public int getAge() { return age; }
    public void process() { return; }
    "#,
    r#"public :[returnType] :[methodName]() |> :[methodName] matches /^get.+/"#,
    2, // only getName and getAge should match
    vec![
      vec![("returnType", "String"), ("methodName", "getName")],
      vec![("returnType", "int"), ("methodName", "getAge")],
    ],
    JAVA,
  );
}

#[test]
fn test_mixed_constraints() {
  run_test(
    r#"getValue("test"); getValue("other"); setValue("test"); getName("test");"#,
    r#":[method](:[arg]) |> :[method] matches /^get.*/, :[arg] in ["\"test\""]"#,
    2, // getValue("test") and getName("test")
    vec![
      vec![("method", "getValue"), ("arg", "\"test\"")],
      vec![("method", "getName"), ("arg", "\"test\"")],
    ],
    JAVA,
  );
}
