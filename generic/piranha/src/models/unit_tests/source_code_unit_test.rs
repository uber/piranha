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
      pub void foobar(){
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
        start_byte: 46,
        end_byte: 75,
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
      pub void foobar(){
        
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
      pub void foobar(){
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
