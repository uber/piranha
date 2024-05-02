use crate::models::default_configs::RUBY;

use super::create_rewrite_tests;

create_rewrite_tests! {
  RUBY,
  test_replace_empty_if_unless_statement: "replace_empty_if_or_unless_statement", 1;
  test_replace_ternary_operator: "replace_ternary_operator", 1;
  test_replace_if_statement: "replace_if_statement",  3;
  test_replace_unless_statement: "replace_unless_statement", 3;
  test_boolean_cleanup: "simplify_boolean_expressions", 1;
  test_simplify_rspec_block_expressions: "simplify_rspec_block_expressions", 1;
  test_simplify_if_lambda_conditional_statements: "simplify_if_lambda_conditional_statements", 1;
  test_simplify_unless_lambda_conditional_statements: "simplify_unless_lambda_conditional_statements", 1;
  test_delete_lines_after_return: "delete_lines_after_return", 1;
  test_simplify_local_variable_assigned_flag_check: "simplify_local_variable_assigned_flag_check", 1;
}
