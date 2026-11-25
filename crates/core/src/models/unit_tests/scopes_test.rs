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

use crate::models::{
  capture_group_patterns::CGPattern,
  default_configs::{JAVA, PYTHON, TYPESCRIPT},
  language::PiranhaLanguage,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
};

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
    .enclosing_node(CGPattern::new(
      "(class_declaration name:(_) @n) @c".to_string(),
    ))
    .scope(CGPattern::new(
      "(
      ((class_declaration name:(_) @z) @qc)
      (#eq? @z \"@n\")
    )"
      .to_string(),
    ))
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
    .enclosing_node(CGPattern::new(
      "(
    [(method_declaration 
              name : (_) @n
              parameters : (formal_parameters)@fp)
     (constructor_declaration 
              name: (_) @n
              parameters : (formal_parameters)@fp)
    ]@xdn)"
        .to_string(),
    ))
    .scope(CGPattern::new(
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
    ))
    .build()
    .unwrap();

  ScopeGeneratorBuilder::default()
    .name("Method".to_string())
    .rules(vec![scope_query_generator_method])
    .build()
    .unwrap()
}

fn _get_piranha_args() -> PiranhaArguments {
  let mut piranha_language = PiranhaLanguage::from(JAVA);
  piranha_language.set_scopes(vec![_get_method_scope(), _get_class_scope()]);
  PiranhaArgumentsBuilder::default()
    .language(piranha_language)
    .create()
    .unwrap()
}

/// Positive test for the generated scope query, given scope generators, source code and position of pervious edit.
#[test]
fn test_get_scope_query_positive() {
  let source_code = "class Test {
      public void foobar(int a, int b, int c){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          System.out.println(a + b + c);
        }
      }
    }";

  let piranha_args = _get_piranha_args();
  let mut parser = PiranhaLanguage::from(JAVA).parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );
  let mut rule_store = RuleStore::new(&piranha_args);
  let scope_query_method = source_code_unit.get_scope_query("Method", 133, 134, &mut rule_store);

  println!("{}", scope_query_method.pattern().as_str());
  assert!(eq_without_whitespace(
    scope_query_method.pattern().as_str(),
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

  let scope_query_class = source_code_unit.get_scope_query("Class", 133, 134, &mut rule_store);
  assert!(eq_without_whitespace(
    scope_query_class.pattern().as_str(),
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
  let piranha_args = _get_piranha_args();
  let mut parser = PiranhaLanguage::from(JAVA).parser();

  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );
  let mut rule_store = RuleStore::new(&piranha_args);
  let _ = source_code_unit.get_scope_query("Method", 9, 10, &mut rule_store);
}

/// Calls `get_scope_query` with a scope level that `language` does not define. Expected to panic.
fn _get_scope_query_for_undefined_level(language: &str, source_code: &str, scope_level: &str) {
  let piranha_args = PiranhaArgumentsBuilder::default()
    .language(PiranhaLanguage::from(language))
    .create()
    .unwrap();
  let mut parser = PiranhaLanguage::from(language).parser();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );
  let mut rule_store = RuleStore::new(&piranha_args);
  let _ = source_code_unit.get_scope_query(scope_level, 0, 1, &mut rule_store);
}

/// A language without a `scope_config.toml` reports that no scope levels are defined.
#[test]
#[should_panic(
  expected = "Scope level \"Method\" is not defined for language Python. Scope levels defined for this language: []. Scope levels available for every language: Parent, ParentIterative, Global, Directory, DirectoryRecursive."
)]
fn test_get_scope_query_undefined_scope_level_no_scopes() {
  _get_scope_query_for_undefined_level(PYTHON, "def foo():\n  x = True\n", "Method");
}

/// A language with a `scope_config.toml` lists the scope levels it defines.
#[test]
#[should_panic(
  expected = "Scope level \"Function\" is not defined for language Ts. Scope levels defined for this language: [Method, Class, File]."
)]
fn test_get_scope_query_undefined_scope_level_lists_defined_levels() {
  _get_scope_query_for_undefined_level(
    TYPESCRIPT,
    "function foo() {\n  const x = true;\n}\n",
    "Function",
  );
}
