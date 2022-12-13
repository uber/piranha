use crate::models::{language::PiranhaLanguage, default_configs::JAVA};

/*
Copyright (c) 2022 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/
use {
  super::{ScopeGenerator, ScopeGeneratorBuilder, ScopeQueryGenerator, ScopeQueryGeneratorBuilder},
  crate::{
    models::{rule_store::RuleStore, source_code_unit::SourceCodeUnit},
    utilities::eq_without_whitespace,
  },
  std::{collections::HashMap, path::PathBuf},
};

fn _get_class_scope() -> ScopeGenerator {
  let scope_query_generator_class: ScopeQueryGenerator = ScopeQueryGeneratorBuilder::default()
    .matcher("(class_declaration name:(_) @n) @c".to_string())
    .generator("(
      ((class_declaration name:(_) @z) @qc)
      (#eq? @z \"@n\")
    )"
      .to_string(),
    )
    .build()
    .unwrap();
  ScopeGeneratorBuilder::default()
    .name("Class".to_string())
    .rules(vec![scope_query_generator_class])
    .build()
    .unwrap()
}

fn _get_method_scope() -> ScopeGenerator {
  let scope_query_generator_method: ScopeQueryGenerator = ScopeQueryGeneratorBuilder::default()
    .matcher(
      "(
    [(method_declaration 
              name : (_) @n
              parameters : (formal_parameters)@fp)
     (constructor_declaration 
              name: (_) @n
              parameters : (formal_parameters)@fp)
    ]@xdn)"
        .to_string(),
    )
    .generator(
      "(
      [(((method_declaration 
                name : (_) @z
                parameters : (formal_parameters)@tp))
        (#eq? @z \"@n\")
        (#eq? @tp \"@fp\")                  
        )
       (((constructor_declaration 
                name: (_) @z
                parameters : (formal_parameters)@tp))
        (#eq? @z \"@n\")
        (#eq? @tp \"@fp\")
        )
      ])@qdn"
        .to_string(),
    )
    .build()
    .unwrap();

  return ScopeGeneratorBuilder::default()
    .name("Method".to_string())
    .rules(vec![scope_query_generator_method])
    .build()
    .unwrap();
}


fn _get_rule_store() -> RuleStore {
  return RuleStore::default_with_scopes(vec![_get_method_scope(), _get_class_scope()])
}

/// Positive test for the generated scope query, given scope generators, source code and position of pervious edit.
#[test]
fn test_get_scope_query_positive() {
  let source_code = "class Test {
      pub void foobar(int a, int b, int c){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          System.out.println(a + b + c);
        }
      }
    }";
  
  let mut rule_store =  _get_rule_store();
  let mut parser = PiranhaLanguage::from(JAVA).parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    rule_store.piranha_args(),
  );

  let scope_query_method = source_code_unit.get_scope_query(
    "Method",
    133,
    134,
    &mut rule_store,
  );

  println!("{}", scope_query_method.as_str());
  assert!(eq_without_whitespace(
    scope_query_method.as_str(),
    "(
      [(((method_declaration 
                name : (_) @z
                parameters : (formal_parameters)@tp))
        (#eq? @z \"foobar\")
        (#eq? @tp \"(int a, int b, int c)\")                  
        )
       (((constructor_declaration 
                name: (_) @z
                parameters : (formal_parameters)@tp))
        (#eq? @z \"foobar\")
        (#eq? @tp \"(int a, int b, int c)\")
        )
      ]
    )@qdn"
  ));

  let scope_query_class =
  source_code_unit.get_scope_query("Class", 133, 134, &mut rule_store);
  assert!(eq_without_whitespace(
    scope_query_class.as_str(),
    "(
        ((class_declaration name:(_) @z) @qc)
        (#eq? @z \"Test\")
        )"
  ));
}

/// Negative test for the generated scope query, given scope generators, source code and position of pervious edit.
#[test]
#[should_panic]
fn test_get_scope_query_negative() {
  let source_code = "class Test {
      pub void foobar(int a, int b, int c, int d){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          System.out.println(a + b + c + d);
        }
      }
    }";
  let mut rule_store = _get_rule_store();
  let mut parser = PiranhaLanguage::from(JAVA).parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    rule_store.piranha_args(),
  );

  let _ = source_code_unit.get_scope_query("Method", 9, 10, &mut rule_store);
}
