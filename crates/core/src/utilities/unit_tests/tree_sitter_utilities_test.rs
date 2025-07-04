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
use std::collections::HashMap;

use tree_sitter::Query;

use crate::{
  models::{capture_group_patterns::CGPattern, default_configs::JAVA, language::PiranhaLanguage},
  utilities::{tree_sitter_utilities::get_all_matches_for_query, Instantiate},
};

#[test]
fn test_get_all_matches_for_query_positive() {
  let source_code = r#"
      class Test {
        void foobar(Experiment exp) {
          if (exp.isFlagTreated(SOME_FLAG)){
            //Do Something
          }
          // Do something else
          
          if (abc && exp.isFlagTreated(SOME_FLAG)){
            // Do this too!
          }
        }
      }
    "#;
  let language = PiranhaLanguage::from(JAVA);
  let query = Query::new(
    *language.language(),
    r#"((
        (method_invocation 
          name : (_) @name
          arguments: ((argument_list 
                          ([
                            (field_access field: (_)@argument)
                            (_) @argument
                           ])) )
              
       ) @method_invocation
      )
      (#eq? @name "isFlagTreated")
      (#eq? @argument "SOME_FLAG")
      )"#,
  )
  .unwrap();

  let mut parser = PiranhaLanguage::from(JAVA).parser();
  let ast = parser
    .parse(source_code, None)
    .expect("Could not parse code");
  let node = ast.root_node();

  let matches = get_all_matches_for_query(
    &node,
    source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
    None,
  );
  assert_eq!(matches.len(), 2);
}

#[test]
fn test_get_all_matches_for_query_negative() {
  let source_code = r#"
      class Test {
        void foobar(Experiment exp) {
          if (exp.isFlagTreated(SOME_OTHER)){
            //Do Something
          }
          // Do something else
          
          if (abc && exp.isFlagTreated(SOME_OTHER)){
            // Do this too!
          }
        }
      }
    "#;
  let language = PiranhaLanguage::from(JAVA);
  let query = Query::new(
    *language.language(),
    r#"((
        (method_invocation 
          name : (_) @name
          arguments: ((argument_list 
                          ([
                            (field_access field: (_)@argument)
                            (_) @argument
                           ])) )
              
       ) @method_invocation
      )
      (#eq? @name "isFlagTreated")
      (#eq? @argument "SOME_FLAG")
      )"#,
  )
  .unwrap();

  let mut parser = PiranhaLanguage::from(JAVA).parser();
  let ast = parser
    .parse(source_code, None)
    .expect("Could not parse code");
  let node = ast.root_node();

  let matches = get_all_matches_for_query(
    &node,
    source_code.to_string(),
    &query,
    true,
    Some("method_invocation".to_string()),
    None,
  );
  assert!(matches.is_empty());
}

#[test]
fn test_instantiate() {
  let substitutions = HashMap::from([
    ("variable_name".to_string(), "isFlagTreated".to_string()),
    ("init".to_string(), "true".to_string()),
  ]);
  assert_eq!(
    CGPattern("@variable_name foo bar @init".to_string())
      .instantiate(&substitutions)
      .0,
    "isFlagTreated foo bar true"
  )
}

#[test]
fn test_instantiate_cs() {
  let substitutions = HashMap::from([
    ("variable_name".to_string(), "isFlagTreated".to_string()),
    ("init".to_string(), "true".to_string()),
  ]);
  assert_eq!(
    CGPattern(":[variable_name] foo bar :[init]".to_string())
      .instantiate(&substitutions)
      .0,
    "isFlagTreated foo bar true"
  )
}
