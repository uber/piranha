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
use crate::models::{default_configs::JAVA, language::PiranhaLanguage};

fn run_test(
  code: &str, pattern: &str, expected_matches: usize, expected_vars: Vec<Vec<(&str, &str)>>,
) {
  let java = PiranhaLanguage::from(JAVA);
  let mut parser = java.parser();
  let tree = parser.parse(code.as_bytes(), None).unwrap();
  let meta = ConcreteSyntax(String::from(pattern));

  let (matches, _is_match_found) = get_all_matches_for_concrete_syntax(
    &tree.root_node().child(0).unwrap(),
    code.as_bytes(),
    &meta,
    true,
    None,
  );

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
  );
}

#[test]
fn test_no_match() {
  run_test(
    "class Example { public int a = 10; }",
    "public String :[name] = :[value];",
    0,
    vec![],
  );
}
