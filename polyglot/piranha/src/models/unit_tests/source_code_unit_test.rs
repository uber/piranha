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
  fs::{self},
  io,
};

use itertools::Itertools;
use tempdir::TempDir;

use tree_sitter::Parser;

use crate::models::piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder};
use {
  super::SourceCodeUnit,
  crate::{
    models::edit::Edit, utilities::eq_without_whitespace,
    utilities::tree_sitter_utilities::get_parser,
  },
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
        .language_name(language_name)
        .build()
        .unwrap(),
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

  let language_name = String::from("java");
  let mut parser = get_parser(language_name.to_string());

  let mut source_code_unit = SourceCodeUnit::default(source_code, &mut parser, language_name);

  let _ = source_code_unit.apply_edit(
    &Edit::dummy_edit(range(49, 78, 3, 9, 3, 38), String::new()),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("boolean isFlagTreated = true;", ""),
    &source_code_unit.code()
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

  let language_name = String::from("java");
  let mut parser = get_parser(language_name.to_string());
  let mut source_code_unit = SourceCodeUnit::default(source_code, &mut parser, language_name);

  let _ = source_code_unit.apply_edit(
    &Edit::dummy_edit(range(1000, 2000, 0, 0, 0, 0), String::new()),
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

  let language_name = String::from("java");
  let mut parser = get_parser(language_name.to_string());

  let mut source_code_unit = SourceCodeUnit::default(source_code, &mut parser, language_name);

  let _ = source_code_unit.apply_edit(
    &Edit::dummy_edit(range(37, 47, 2, 26, 2, 36), String::new()),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("\"NullAway\",", ""),
    &source_code_unit.code()
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

  let language_name = String::from("swift");

  let mut parser = get_parser(language_name.to_string());

  let mut source_code_unit = SourceCodeUnit::default(source_code, &mut parser, language_name);

  let _ = source_code_unit.apply_edit(
    &Edit::dummy_edit(range(59, 75, 3, 23, 3, 41), String::new()),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    &source_code.replace("name: \"BMX Bike\",", ""),
    &source_code_unit.code()
  ));
}
fn execute_persist_in_temp_folder(
  source_code: &str, args: &PiranhaArguments,
  check_predicate: &dyn Fn(&TempDir) -> Result<bool, io::Error>,
) -> Result<bool, io::Error> {
  let language_name = String::from("java");
  let mut parser = get_parser(language_name.to_string());
  let tmp_dir = TempDir::new("example")?;
  let file_path = &tmp_dir.path().join("Sample1.java");
  _ = fs::write(&file_path.as_path(), source_code);
  let piranha_args = PiranhaArgumentsBuilder::default()
    .language_name(language_name)
    .build()
    .unwrap();
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    file_path.as_path(),
    &piranha_args,
  );
  source_code_unit.persist(args);
  check_predicate(&tmp_dir)
}

#[test]
fn test_persist_delete_file_when_empty() -> Result<(), io::Error> {
  let args = PiranhaArgumentsBuilder::default()
    .delete_consecutive_new_lines(true)
    .delete_file_if_empty(true)
    .build()
    .unwrap();
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
    .build()
    .unwrap();
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
    .delete_file_if_empty(true)
    .build()
    .unwrap();
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
    .delete_file_if_empty(true)
    .build()
    .unwrap();
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
