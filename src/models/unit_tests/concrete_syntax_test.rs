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

use crate::models::capture_group_patterns::ConcreteSyntax;
use crate::models::concrete_syntax::get_all_matches_for_concrete_syntax;
use crate::models::default_configs::GO;
use crate::models::{default_configs::JAVA, language::PiranhaLanguage};

fn run_test(
  code: &str, pattern: &str, expected_matches: usize, expected_vars: Vec<Vec<(&str, &str)>>,
  language: &str,
) {
  let java = PiranhaLanguage::from(language);
  let mut parser = java.parser();
  let tree = parser.parse(code.as_bytes(), None).unwrap();
  let meta = ConcreteSyntax(String::from(pattern));

  let (matches, _is_match_found) =
    get_all_matches_for_concrete_syntax(&tree.root_node(), code.as_bytes(), &meta, true, None);

  assert_eq!(matches.len(), expected_matches);

  for (i, vars) in expected_vars.iter().enumerate() {
    let match_item = &matches[i];
    for &(var, expected_val) in vars {
      let val = match_item.matches.get(var).unwrap();
      assert_eq!(val, expected_val);
    }
  }
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
