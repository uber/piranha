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

use crate::{
    models::{default_configs::JAVA, language::PiranhaLanguage},
    tests::substitutions,
};
use crate::models::capture_group_patterns::MetaSyntax;
use crate::models::concrete_syntax::get_all_matches_for_metasyntax;

#[test]
fn test_single_match() {
    let java = PiranhaLanguage::from(JAVA);
    let mut parser = java.parser();

    let code = "class Example { public int a = 10; }";
    let tree = parser.parse(code.as_bytes(), None).unwrap();

    let meta = MetaSyntax(String::from("public int :[name] = :[value];"));

    let (matches, is_match_found) = get_all_matches_for_metasyntax(&tree.root_node().child(0).unwrap(), code.as_bytes(), &meta, true);

    assert_eq!(matches.len(), 1);

    let first_match = &matches[0];
    let var_name = first_match.matches.get("name").unwrap();
    let var_value = first_match.matches.get("value").unwrap();
    assert_eq!(var_name, "a");
    assert_eq!(var_value, "10");
}


#[test]
fn test_multiple_match() {
    let java = PiranhaLanguage::from(JAVA);
    let mut parser = java.parser();

    let code = "class Example { public int a = 10; public int b = 20; }";
    let tree = parser.parse(code.as_bytes(), None).unwrap();

    let meta = MetaSyntax(String::from("public int :[name] = :[value];"));

    let (matches, is_match_found) = get_all_matches_for_metasyntax(&tree.root_node().child(0).unwrap(), code.as_bytes(), &meta, true);

    assert_eq!(matches.len(), 2);

    let first_match = &matches[0];
    let first_var_name = first_match.matches.get("name").unwrap();
    let first_var_value = first_match.matches.get("value").unwrap();
    assert_eq!(first_var_name, "a");
    assert_eq!(first_var_value, "10");

    let second_match = &matches[1];
    let second_var_name = second_match.matches.get("name").unwrap();
    let second_var_value = second_match.matches.get("value").unwrap();
    assert_eq!(second_var_name, "b");
    assert_eq!(second_var_value, "20");
}

#[test]
fn test_no_match() {
    let java = PiranhaLanguage::from(JAVA);
    let mut parser = java.parser();

    let code = "class Example { public int a = 10; }";
    let tree = parser.parse(code.as_bytes(), None).unwrap();

    let meta = MetaSyntax(String::from("public String :[name] = :[value];"));

    let (matches, is_match_found) = get_all_matches_for_metasyntax(&tree.root_node().child(0).unwrap(), code.as_bytes(), &meta, true);

    assert_eq!(matches.len(), 0);
    assert!(!is_match_found);
}

