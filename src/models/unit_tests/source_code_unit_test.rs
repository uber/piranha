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

use tree_sitter::Parser;

use crate::{
  constraint,
  models::{
    default_configs::{JAVA, SWIFT, UNUSED_CODE_PATH},
    language::PiranhaLanguage,
    piranha_arguments::PiranhaArgumentsBuilder,
    rule::InstantiatedRule,
    rule_store::RuleStore,
  },
  piranha_rule,
  utilities::eq_without_whitespace,
};
use {
  super::SourceCodeUnit,
  crate::models::edit::Edit,
  std::{collections::HashMap, path::PathBuf},
  tree_sitter::Range,
};

impl SourceCodeUnit {
  pub(crate) fn default(content: &str, parser: &mut Parser, language_name: String) -> Self {
    SourceCodeUnit::new(
      parser,
      content.to_string(),
      &HashMap::new(),
      PathBuf::new().as_path(),
      &PiranhaArgumentsBuilder::default()
        .path_to_codebase("some/test/path/".to_string())
        .language(PiranhaLanguage::from(language_name.as_str()))
        .build(),
    )
  }
}

fn range(
  start_byte: usize, end_byte: usize, start_row: usize, start_column: usize, end_row: usize,
  end_column: usize,
) -> Range {
  Range {
    start_byte,
    end_byte,
    start_point: tree_sitter::Point {
      row: start_row,
      column: start_column,
    },
    end_point: tree_sitter::Point {
      row: end_row,
      column: end_column,
    },
  }
}

fn get_java_tree_sitter_language() -> PiranhaLanguage {
  PiranhaLanguage::from(JAVA)
}

/// Positive test of an edit being applied  given replacement range  and replacement string.
#[test]
fn test_apply_edit_positive() {
  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = true;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }";

  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();

  let mut source_code_unit =
    SourceCodeUnit::default(source_code, &mut parser, java.name().to_string());

  let _ = source_code_unit.apply_edit(
    &Edit::delete_range(source_code, range(49, 78, 3, 9, 3, 38)),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("boolean isFlagTreated = true;", ""),
    source_code_unit.code()
  ));
}

/// Negative test of an edit being applied given invalid replacement range and replacement string.
#[test]
#[should_panic(expected = "byte index 1000 is out of bounds")]
fn test_apply_edit_negative() {
  let source_code = "class Test {
      public void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }";

  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let mut source_code_unit =
    SourceCodeUnit::default(source_code, &mut parser, java.name().to_string());

  let _ = source_code_unit.apply_edit(
    &Edit::delete_range(source_code, range(1000, 2000, 0, 0, 0, 0)),
    &mut parser,
  );
}

/// Positive test of an edit being applied  given replacement range  and replacement string.
/// This scenario checks the logic that removes the comma identified by tree-sitter.
#[test]
fn test_apply_edit_comma_handling_via_grammar() {
  let source_code = "class Test {
      @SuppressWarnings(\"NullAway\",\"FooBar\")
      public void is_valid(@Nullable String s){
        return s != null && check(s);
      }
    }";

  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();

  let mut source_code_unit =
    SourceCodeUnit::default(source_code, &mut parser, java.name().to_string());

  let _ = source_code_unit.apply_edit(
    &Edit::delete_range(source_code, range(37, 47, 2, 26, 2, 36)),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("\"NullAway\",", ""),
    source_code_unit.code()
  ));
}

/// Positive test of an edit being applied  given replacement range  and replacement string.
/// Currently swift grammar does not always identify extra commas, we use regex replace at this point.
/// This test scenario checks the regex replacement logic.
#[test]
fn test_apply_edit_comma_handling_via_regex() {
  let source_code = "class Test {
    func some_func() {
      var bike1 = Bike(name: \"BMX Bike\", gear: 2)
      print(\"Name: \\(bike1.name) and Gear: \\(bike1.gear)\")
  }
}";

  let swift = PiranhaLanguage::from(SWIFT);

  let mut parser = swift.parser();

  let mut source_code_unit =
    SourceCodeUnit::default(source_code, &mut parser, swift.name().to_string());

  let _ = source_code_unit.apply_edit(
    &Edit::delete_range(source_code, range(59, 75, 3, 23, 3, 41)),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("name: \"BMX Bike\",", ""),
    source_code_unit.code()
  ));
}

#[test]
fn test_satisfies_constraints_positive() {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          value: [(true) (false)] @init)) @variable_declaration)
      )",
    replace_node= "variable_declaration",
    replace= "",
    constraints= [constraint!{
      matcher= "(method_declaration) @md",
      queries= ["(
        ((assignment_expression
                        left: (_) @a.lhs
                        right: (_) @a.rhs) @assignment)
        (#eq? @a.lhs \"@variable_name\")
        (#not-eq? @a.rhs \"@init\")
      )",]
    }]

  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
      pub void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = true;
        if (isFlagTreated) {
        // Do something;
        }
       }
      }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_args = PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    &piranha_args,
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  assert!(source_code_unit.is_satisfied(
    *node,
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string())
    ]),
    &mut rule_store,
  ));
}

#[test]
fn test_satisfies_constraints_negative() {
  let _rule = piranha_rule! {
    name= "test",
    query= "(
      ((local_variable_declaration
          declarator: (variable_declarator
          name: (_) @variable_name
          value: [(true) (false)] @init)) @variable_declaration)
      )",
    replace_node= "variable_declaration",
    replace= "",
    constraints= [constraint!{
      matcher= "(method_declaration) @md",
      queries= ["(
        ((assignment_expression
                        left: (_) @a.lhs
                        right: (_) @a.rhs) @assignment)
        (#eq? @a.lhs \"@variable_name\")
        (#not-eq? @a.rhs \"@init\")
      )",]
    }]
  };
  let rule = InstantiatedRule::new(&_rule, &HashMap::new());
  let source_code = "class Test {
      pub void foobar(){
        boolean isFlagTreated = true;
        isFlagTreated = false;
        if (isFlagTreated) {
        // Do something;
        }
       }
      }";

  let mut rule_store = RuleStore::default();
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let piranha_arguments = &PiranhaArgumentsBuilder::default()
    .path_to_codebase(UNUSED_CODE_PATH.to_string())
    .language(java)
    .build();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
    piranha_arguments,
  );

  let node = &source_code_unit
    .root_node()
    .descendant_for_byte_range(50, 72)
    .unwrap();

  assert!(!source_code_unit.is_satisfied(
    *node,
    &rule,
    &HashMap::from([
      ("variable_name".to_string(), "isFlagTreated".to_string()),
      ("init".to_string(), "true".to_string())
    ]),
    &mut rule_store,
  ));
}
