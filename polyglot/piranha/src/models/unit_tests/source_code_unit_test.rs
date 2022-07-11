use std::{
  fs::{self},
  io,
};

use itertools::Itertools;
use tempdir::TempDir;

use crate::models::piranha_arguments::PiranhaArguments;

use {
  super::SourceCodeUnit,
  crate::{
    models::edit::Edit, models::rule::Rule, utilities::eq_without_whitespace,
    utilities::tree_sitter_utilities::get_parser,
  },
  std::{collections::HashMap, path::PathBuf},
  tree_sitter::Range,
};

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

  let mut parser = get_parser(String::from("java"));

  let mut source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
  );

  let _ = source_code_unit.apply_edit(
    &Edit::new(
      Range {
        start_byte: 49,
        end_byte: 78,
        start_point: tree_sitter::Point { row: 3, column: 9 },
        end_point: tree_sitter::Point { row: 3, column: 38 },
      },
      String::new(),
      Rule::dummy(),
      HashMap::new(),
    ),
    &mut parser,
  );
  assert!(eq_without_whitespace(
    r#"class Test {
      public void foobar(){
        
        isFlagTreated = true;
        if (isFlagTreated) {
          // Do something;
        }
      }
    }"#,
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

  let mut parser = get_parser(String::from("java"));

  let mut source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    PathBuf::new().as_path(),
  );

  let _ = source_code_unit.apply_edit(
    &Edit::new(
      Range {
        start_byte: 1000,
        end_byte: 10075,
        start_point: tree_sitter::Point { row: 3, column: 9 },
        end_point: tree_sitter::Point { row: 3, column: 38 },
      },
      String::new(),
      Rule::dummy(),
      HashMap::new(),
    ),
    &mut parser,
  );
}

fn execute_persist_in_temp_folder(
  source_code: &str, args: &PiranhaArguments,
  check_predicate: &dyn Fn(&TempDir) -> Result<bool, io::Error>,
) -> Result<bool, io::Error> {
  let mut parser = get_parser(String::from("java"));
  let tmp_dir = TempDir::new("example")?;
  let file_path = &tmp_dir.path().join("Sample1.java");
  _ = fs::write(&file_path.as_path(), source_code.to_string());
  let source_code_unit = SourceCodeUnit::new(
    &mut parser,
    source_code.to_string(),
    &HashMap::new(),
    &file_path.as_path(),
  );
  source_code_unit.persist(args);
  check_predicate(&tmp_dir)
}

#[test]
fn test_persist_delete_file_when_empty() -> Result<(), io::Error> {
  let args = PiranhaArguments::dummy_with_user_opt(true, true);
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
  let args = PiranhaArguments::dummy_with_user_opt(false, true);
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
  let args = PiranhaArguments::dummy_with_user_opt(true, true);
  let source_code = "class Test {
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
    return Ok(actual_content.eq(&expected_str));
  }
  assert!(execute_persist_in_temp_folder(source_code, &args, &check)?);
  Ok(())
}

#[test]
fn test_persist_do_not_delete_consecutive_lines() -> Result<(), io::Error> {
  let args = PiranhaArguments::dummy_with_user_opt(true, false);
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
    return Ok(actual_content.eq(&actual_content));
  }
  assert!(execute_persist_in_temp_folder(source_code, &args, &check)?);
  Ok(())
}
