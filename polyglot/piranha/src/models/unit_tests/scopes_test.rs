use {
  super::{ScopeGenerator, ScopeQueryGenerator},
  crate::{
    models::{rule_store::RuleStore, source_code_unit::SourceCodeUnit},
    utilities::eq_without_whitespace,
    utilities::tree_sitter_utilities::get_parser,
  },
  std::{collections::HashMap, path::PathBuf},
};

/// Positive test for the generated scope query, given scope generators, source code and position of pervious edit.
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
    ScopeGenerator::get_scope_query(source_code_unit, "Class", 133, 134, &mut rule_store);
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

  let _ = ScopeGenerator::get_scope_query(source_code_unit, "Method", 133, 134, &mut rule_store);
}
