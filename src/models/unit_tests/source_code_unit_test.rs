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
use std::{
  collections::HashSet,
  fs::{self},
  io,
};

use itertools::Itertools;
use tempdir::TempDir;

use tree_sitter::Parser;

use crate::models::{
  constraint::Constraint,
  default_configs::{JAVA, SWIFT},
  language::PiranhaLanguage,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
  rule::{InstantiatedRule, Rule},
  rule_store::RuleStore,
};
use {
  super::SourceCodeUnit,
  crate::{models::edit::Edit, utilities::eq_without_whitespace},
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
        .language(language_name)
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
fn execute_persist_in_temp_folder(
  source_code: &str, piranha_arguments: &PiranhaArguments,
  check_predicate: &dyn Fn(&TempDir) -> Result<bool, io::Error>,
) -> Result<bool, io::Error> {
  let java = get_java_tree_sitter_language();
  let mut parser = java.parser();
  let tmp_dir = TempDir::new("example")?;
  let file_path = &tmp_dir.path().join("Sample1.java");
  _ = fs::write(file_path.as_path(), source_code);
  let mut source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    file_path.as_path(),
    piranha_arguments,
  );
  source_code_unit.perform_delete_consecutive_new_lines();
  source_code_unit.persist(piranha_arguments);
  check_predicate(&tmp_dir)
}

#[test]
fn test_persist_delete_file_when_empty() -> Result<(), io::Error> {
  let args = PiranhaArgumentsBuilder::default()
    .delete_consecutive_new_lines(true)
    .build();
  println!("{args:?}");
  let source_code = "";
  fn check(temp_dir: &TempDir) -> Result<bool, io::Error> {
    let paths = fs::read_dir(temp_dir)?;
    Ok(paths.count() == 0)
  }
  assert!(execute_persist_in_temp_folder(source_code, &args, &check)?);
  Ok(())
}

#[test]
fn test_persist_do_not_delete_file_when_empty() -> Result<(), io::Error> {
  let args = PiranhaArgumentsBuilder::default()
    .delete_consecutive_new_lines(true)
    .delete_file_if_empty(false)
    .build();
  let source_code = "";
  fn check(temp_dir: &TempDir) -> Result<bool, io::Error> {
    let paths = fs::read_dir(temp_dir)?;
    Ok(paths.count() == 1)
  }

  assert!(execute_persist_in_temp_folder(source_code, &args, &check)?);
  Ok(())
}

#[test]
fn test_persist_delete_consecutive_lines() -> Result<(), io::Error> {
  let args = PiranhaArgumentsBuilder::default()
    .delete_consecutive_new_lines(true)
    .build();
  let source_code_test_1 = "class Test {
    public void foobar() {

      System.out.println(\"Hello World!\");


      System.out.println();
    }
  }";
  let source_code_test_2 = "class Test {
    public void foobar() {

      System.out.println(\"Hello World!\");




      System.out.println();
    }
  }";
  fn check(temp_dir: &TempDir) -> Result<bool, io::Error> {
    let paths = fs::read_dir(temp_dir)?;
    let path = paths.find_or_first(|_| true).unwrap()?;
    let expected_str = "class Test {
    public void foobar() {

      System.out.println(\"Hello World!\");

      System.out.println();
    }
  }";
    let actual_content = fs::read_to_string(path.path().as_path())?;
    Ok(actual_content.eq(&expected_str))
  }
  assert!(execute_persist_in_temp_folder(
    source_code_test_1,
    &args,
    &check
  )?);
  assert!(execute_persist_in_temp_folder(
    source_code_test_2,
    &args,
    &check
  )?);
  Ok(())
}

#[test]
fn test_persist_do_not_delete_consecutive_lines() -> Result<(), io::Error> {
  let args = PiranhaArgumentsBuilder::default()
    .delete_consecutive_new_lines(false)
    .build();
  let source_code = "class Test {
    public void foobar() {

      System.out.println(\"Hello World!\");


      System.out.println();
    }
  }";
  fn check(temp_dir: &TempDir) -> Result<bool, io::Error> {
    let paths = fs::read_dir(temp_dir)?;
    let path = paths.find_or_first(|_| true).unwrap()?;
    let actual_content = fs::read_to_string(path.path().as_path())?;
    Ok(actual_content.eq(&actual_content))
  }
  assert!(execute_persist_in_temp_folder(source_code, &args, &check)?);
  Ok(())
}

#[test]
fn test_satisfies_constraints_positive() {
  let _rule = Rule::new(
    "test",
    "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          value: [(true) (false)] @init)) @variable_declaration)
      )",
    "variable_declaration",
    "",
    HashSet::new(),
    HashSet::from([Constraint::new(
      String::from("(method_declaration) @md"),
      vec![String::from(
        "(
         ((assignment_expression
                         left: (_) @a.lhs
                         right: (_) @a.rhs) @assignment)
         (#eq? @a.lhs \"@variable_name\")
         (#not-eq? @a.rhs \"@init\")
       )",
      )],
    )]),
  );
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
    .language(java.name().to_string())
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
  let _rule = Rule::new(
    "test",
    "(
      ((local_variable_declaration
                      declarator: (variable_declarator
                                          name: (_) @variable_name
                                          value: [(true) (false)] @init)) @variable_declaration)
      )",
    "variable_declaration",
    "",
    HashSet::new(),
    HashSet::from([Constraint::new(
      String::from("(method_declaration) @md"),
      vec![String::from(
        "(
         ((assignment_expression
                         left: (_) @a.lhs
                         right: (_) @a.rhs) @assignment)
         (#eq? @a.lhs \"@variable_name\")
         (#not-eq? @a.rhs \"@init\")
       )",
      )],
    )]),
  );
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
    .language(java.name().to_string())
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
