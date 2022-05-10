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

use itertools::Itertools;
use serde_derive::Deserialize;

use crate::utilities::tree_sitter_utilities::{
  get_node_for_range, substitute_tags, PiranhaHelpers,
};

use super::{rule_store::RuleStore, source_code_unit::SourceCodeUnit};

// Represents the content in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeConfig {
  scopes: Vec<ScopeGenerator>,
}

impl ScopeConfig {
  /// Get a reference to the scope `config's` scopes.
  #[must_use]
  pub(crate) fn scopes(&self) -> Vec<ScopeGenerator> {
    self.scopes.iter().cloned().collect_vec()
  }
}

// Represents an entry in the `scope_config.toml` file
#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeGenerator {
  name: String,
  rules: Vec<ScopeQueryGenerator>,
}

impl ScopeGenerator {
  pub(crate) fn name(&self) -> &str {
    self.name.as_ref()
  }

  pub(crate) fn rules(&self) -> Vec<ScopeQueryGenerator> {
    self.rules.iter().cloned().collect_vec()
  }

  /// Generate a tree-sitter based query representing the scope of the previous edit.
  /// We generate these scope queries by matching the rules provided in `<lang>_scopes.toml`.
  pub(crate) fn get_scope_query(
    source_code_unit: SourceCodeUnit, scope_level: &str, start_byte: usize, end_byte: usize,
    rules_store: &mut RuleStore,
  ) -> String {
    let root_node = source_code_unit.root_node();
    let mut changed_node = get_node_for_range(root_node, start_byte, end_byte);

    // Get the scope matchers for `scope_level` from the `scope_config.toml`.
    let scope_matchers = rules_store.get_scope_query_generators(scope_level);

    // Match the `scope_matcher.matcher` to the parent
    while let Some(parent) = changed_node.parent() {
      for m in &scope_matchers {
        if let Some((_, captures_by_tag)) = parent.get_match_for_query(
          &source_code_unit.code(),
          rules_store.get_query(&m.matcher()),
          false,
        ) {
          // Generate the scope query for the specific context by substituting the
          // the tags with code snippets appropriately in the `generator` query.
          return substitute_tags(m.generator(), &captures_by_tag);
        } else {
          changed_node = parent;
        }
      }
    }
    panic!("Could not create scope query for {:?}", scope_level);
  }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub(crate) struct ScopeQueryGenerator {
  matcher: String, // a tree-sitter query matching some enclosing AST pattern (like method or class)
  generator: String, // a tree-sitter query matching the exact AST node
}

impl ScopeQueryGenerator {
  pub(crate) fn matcher(&self) -> String {
    String::from(&self.matcher)
  }

  pub(crate) fn generator(&self) -> String {
    String::from(&self.generator)
  }
}

mod test {

  #[cfg(test)]
  use super::{ScopeGenerator, ScopeQueryGenerator};

  #[cfg(test)]
  use crate::{
    models::{rule_store::RuleStore, source_code_unit::SourceCodeUnit},
    utilities::eq_without_whitespace,
    utilities::tree_sitter_utilities::get_parser,
  };
  #[cfg(test)]
  use std::{collections::HashMap, path::PathBuf};
  #[cfg(test)]
  impl ScopeQueryGenerator {
    fn new(matcher: &str, generator: &str) -> ScopeQueryGenerator {
      ScopeQueryGenerator {
        matcher: matcher.to_string(),
        generator: generator.to_string(),
      }
    }
  }
  #[cfg(test)]
  impl ScopeGenerator {
    fn new(name: &str, rules: Vec<ScopeQueryGenerator>) -> ScopeGenerator {
      ScopeGenerator {
        name: name.to_string(),
        rules,
      }
    }
  }

  #[test]
  fn test_get_scope_query_positive() {
    let scope_generator_method = ScopeGenerator::new(
      "Method",
      vec![ScopeQueryGenerator::new(
        "((method_declaration 
          name : (_) @n
                parameters : (formal_parameters
                    (formal_parameter type:(_) @t0)
                        (formal_parameter type:(_) @t1)
                        (formal_parameter type:(_) @t2))) @xd2)",
        "(((method_declaration 
                        name : (_) @z
                              parameters : (formal_parameters
                                  (formal_parameter type:(_) @r0)
                                      (formal_parameter type:(_) @r1)
                                      (formal_parameter type:(_) @r2))) @qd)
                  (#eq? @z \"@n\")
                  (#eq? @r0 \"@t0\")
                  (#eq? @r1 \"@t1\")
                  (#eq? @r2 \"@t2\")
                  )",
      )],
    );

    let scope_generator_class = ScopeGenerator::new(
      "Class",
      vec![ScopeQueryGenerator::new(
        "(class_declaration name:(_) @n) @c",
        "(
          ((class_declaration name:(_) @z) @qc)
          (#eq? @z \"@n\")
          )",
      )],
    );

    let source_code = "class Test {
      pub void foobar(int a, int b, int c){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          System.out.println(a + b + c);
        }
      }
    }";

    let mut rule_store =
      RuleStore::dummy_with_scope(vec![scope_generator_method, scope_generator_class]);
    let mut parser = get_parser(String::from("java"));

    let source_code_unit = SourceCodeUnit::new(
      &mut parser,
      source_code.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
    );

    let scope_query_method = ScopeGenerator::get_scope_query(
      source_code_unit.clone(),
      "Method",
      133,
      134,
      &mut rule_store,
    );

    assert!(eq_without_whitespace(
      scope_query_method.as_str(),
      "(((method_declaration 
      name : (_) @z
            parameters : (formal_parameters
                (formal_parameter type:(_) @r0)
                    (formal_parameter type:(_) @r1)
                    (formal_parameter type:(_) @r2))) @qd)
            (#eq? @z \"foobar\")
            (#eq? @r0 \"int\")
            (#eq? @r1 \"int\")
            (#eq? @r2 \"int\")
            )"
    ));

    let scope_query_class =
      ScopeGenerator::get_scope_query(source_code_unit.clone(), "Class", 133, 134, &mut rule_store);
    assert!(eq_without_whitespace(
      scope_query_class.as_str(),
      "(
        ((class_declaration name:(_) @z) @qc)
        (#eq? @z \"Test\")
        )"
    ));
  }

  #[test]
  #[should_panic]
  fn test_get_scope_query_negative() {
    let scope_generator_method = ScopeGenerator::new(
      "Method",
      vec![ScopeQueryGenerator::new(
        "((method_declaration 
          name : (_) @n
                parameters : (formal_parameters
                    (formal_parameter type:(_) @t0)
                        (formal_parameter type:(_) @t1)
                        (formal_parameter type:(_) @t2))) @xd2)",
        "(((method_declaration 
                        name : (_) @z
                              parameters : (formal_parameters
                                  (formal_parameter type:(_) @r0)
                                      (formal_parameter type:(_) @r1)
                                      (formal_parameter type:(_) @r2))) @qd)
                  (#eq? @z \"@n\")
                  (#eq? @r0 \"@t0\")
                  (#eq? @r1 \"@t1\")
                  (#eq? @r2 \"@t2\")
                  )",
      )],
    );

    let scope_generator_class = ScopeGenerator::new(
      "Class",
      vec![ScopeQueryGenerator::new(
        "(class_declaration name:(_) @n) @c",
        "(
          ((class_declaration name:(_) @z) @qc)
          (#eq? @z \"@n\")
          )",
      )],
    );

    let source_code = "class Test {
      pub void foobar(int a, int b, int c, int d){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          System.out.println(a + b + c + d);
        }
      }
    }";

    let mut rule_store =
      RuleStore::dummy_with_scope(vec![scope_generator_method, scope_generator_class]);
    let mut parser = get_parser(String::from("java"));

    let source_code_unit = SourceCodeUnit::new(
      &mut parser,
      source_code.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
    );

    let _ = ScopeGenerator::get_scope_query(
      source_code_unit.clone(),
      "Method",
      133,
      134,
      &mut rule_store,
    );
  }
}
