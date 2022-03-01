use crate::tree_sitter::get_language;
use tree_sitter::Query;

#[derive(Debug)]
pub struct Rule {
    pub query: Query,
    // pub complete_capture_var: String,
    pub rewrite_template: String,
    pub chained_rule: Box<Option<Rule>>,
}

impl Rule {
    pub fn java_rule(query_str: &str, rewrite_template: &str) -> Result<Rule, String> {
        let q = Query::new(get_language("Java"), query_str);
        if q.is_err() {
            return Err(format!(
                "CONFIGURATION ERROR: The query is incorrect. Please check - \n{:?} ",
                query_str
            ));
        }
        let query = q.unwrap();
        // String::from(capture_var);
        let rewrite_template = String::from(rewrite_template);

        let r = Rule {
            query,
            rewrite_template,
            chained_rule: Box::from(None),
        };
        Ok(r)
    }
}
// ((method_declaration
// (modifiers ((annotation
// name : (_)@an
// arguments :
// (annotation_argument_list
// ((element_value_pair
// key: (_) @k2
// value: (_) @v3))
// ((element_value_pair
// key: (_) @k1
// value: (_) @v1))
//
// ) @ag ) @a))
// name: (_) @md_name
// ) @m)
// (#eq? @k1 "key")
// (#eq? @v1 "safety_identity_verification_docscan_library_migration")
// (#eq? @k2 "namespace")
// (#eq? @v3 "trusted_identity_mobile")

// TODO Add all the checks for rules :
// (i) Rewrite rule should contain template variable from the query
// (ii) complete capture variable should be valid
// (iii)
fn check_rules(rules: Vec<Result<Rule, String>>) -> Vec<Rule> {
    let mut errorneous_rules = String::new();
    let mut ok_rules = vec![];
    for rule in rules {
        match rule {
            Ok(r) => {
                ok_rules.push(r);
            }
            Err(e) => errorneous_rules = errorneous_rules + e.as_str() + "\n",
        }
    }
    if errorneous_rules.is_empty() {
        return ok_rules;
    }
    panic!("{}", &errorneous_rules);
}

pub mod java_rules {
    use super::Rule;
    use crate::configs::check_rules;

    pub fn get_xp_api_rules() -> Vec<Rule> {
        let r1 = Rule::java_rule(
            "(
    (
        (method_invocation
            object: (method_invocation name: (identifier) @i2)
            name : (identifier) @i) @m)
        (#eq? @i getVal)
        (#eq? @i2 staleFlag)
    )",
            "true",
        );

        let r2 = Rule::java_rule(
            "(
    (
        (method_invocation
            object: (method_invocation name: (identifier) @i2)
            name : (identifier) @i) @m)
        (#eq? @i getValN)
        (#eq? @i2 staleFlag)
    )",
            "false",
        );
        let rules = vec![r1, r2];
        return check_rules(rules);
    }

    pub fn cleanup_rules() -> Vec<Rule> {
        let _if_true = Rule::java_rule(
            "
( (if_statement
    condition : ((parenthesized_expression ((true) @i)))
    consequence : ((statement) @s)
)  @ifstmt)
",
            "@s",
        );

        let _if_else_false = Rule::java_rule(
            "
( (if_statement
    condition : ((parenthesized_expression ((false) @i)))
    consequence : ((statement) @s)
    alternative : ((_) @el)
)  @ifstmt)",
            "@el",
        );

        let _if_false = Rule::java_rule(
            "
((if_statement
    condition : ((parenthesized_expression ((false) @i)))
    consequence : ((statement) @s)
    !alternative
)  @ifstmt)
",
            "",
        );

        let _ternary_op_true = Rule::java_rule(
            "
( (if_statement
    condition : ((parenthesized_expression ((true) @i)))
    consequence : ((statement) @s)
)  @ifstmt)
",
            "@s",
        );

        let _neg_false = Rule::java_rule(
            "
((unary_expression
operator: \"!\"
operand: (false)) @u)
",
            "true",
        );

        let _neg_true = Rule::java_rule(
            "
((unary_expression
operator: \"!\"
operand: (true)) @u)
",
            "false",
        );

        let _unnecessary_block = Rule::java_rule(
            "(
(block
  (
      (
    (_)*@pre
    (block (_)* @b)@n)
    (_)*@post
    )
)@b1
)",
            "{\n@pre\n@b\n@post\n}",
        );

        let _resolve_and_true_left = Rule::java_rule(
            // "((binary_expression left: (true) @l operator:\"&&\" right : (_) @other )) @b",
            "((binary_expression left: (true) operator:\"&&\" right : (_)* @other )) @b",
            "@other",
        );

        let _resolve_and_true_right = Rule::java_rule(
            "
    ((binary_expression
    left : (_)* @other
    operator:\"&&\"
    right: (true)
    ) @b)",
            "@other",
        );

        let _resolve_or_right_true = Rule::java_rule(
            "
    ((binary_expression
        left : (_)* @other
        operator:\"||\"
        right: (true)
    ) @b)",
            "true",
        );

        let _resolve_or_left_true = Rule::java_rule(
            "
    ((binary_expression
        left : (true)
        operator:\"||\"
        right: (_)* @other
    ) @b)",
            "true",
        );

        let _resolve_and_false_left = Rule::java_rule(
            "
    ((binary_expression
        left: (false)
        operator:\"&&\"
        right : (_)* @other
    ) @b)",
            "false",
        );

        let _resolve_and_false_right = Rule::java_rule(
            "
    ((binary_expression
        left : (_)* @other
        operator:\"&&\"
        right: (false)
    ) @b)",
            "false",
        );

        let _resolve_or_false_left = Rule::java_rule(
            "((binary_expression
        left : (_)* @other
        operator:\"||\"
        right: (false)
    ) @bo)
    ",
            "@other",
        );

        let _resolve_or_false_right = Rule::java_rule(
            "((binary_expression
        left : (false)
        operator:\"||\"
        right: (_)* @other
    ) @b)",
            "@other",
        );

        let _remove_brackets_or = Rule::java_rule(
            "(
        (binary_expression
            left: ((parenthesized_expression (expression !left) @e))
            operator: \"||\"
            right: (_)* @r)
        @bo)",
            "@e || @r",
        );

        let _remove_brackets_or_complement = Rule::java_rule(
            "(binary_expression
            left: (_)* @l
            operator: \"||\"
            right: ((parenthesized_expression (expression !left) @e))
        @bo)",
            "@l || @e",
        );

        let _remove_brackets_and = Rule::java_rule(
            "(
        (binary_expression
            left: ((parenthesized_expression (expression !left) @e))
            operator: \"&&\"
            right: (_)* @r)
        @bo)",
            "@e && @r",
        );

        let _remove_brackets_and_complement = Rule::java_rule(
            "(
        (binary_expression
            left: (_)* @l
            operator: \"&&\"
            right: ((parenthesized_expression (expression !left) @e)))
            @bo
        )",
            "@l && @e",
        );

        let _remove_brackets_and_complement = Rule::java_rule(
            "(
        (binary_expression
            left: (_)* @l
            operator: \"&&\"
            right: ((parenthesized_expression (expression !left) @e)))
            @bo
        )",
            "@l && @e",
        );

        let _remove_brackets_binary_op = Rule::java_rule(
            "(
        (binary_expression
            left: (_)* @l
            operator: \"&&\"
            right: ((parenthesized_expression (expression !left) @e)))
            @bo
        )",
            "@l && @e",
        );

        let _elide_after_return = Rule::java_rule(
            "(
        (block  ((statement)* @pre)
         ((return_statement) @r)
         ((statement)+ @post)) @b)",
            "{\n@pre\n@r\n}",
        );

        let _resolve_ternary_true = Rule::java_rule(
            "(ternary_expression condition: (true)
                consequence: (_)* @then
                alternative: (_)* @else) @t",
            "@then",
        );

        let _resolve_ternary_false = Rule::java_rule(
            "(ternary_expression condition: (true)
                consequence: (_)* @then
                alternative: (_)* @else) @t",
            "@else",
        );

        let rules = vec![
            _if_true,
            _if_else_false,
            _neg_false,
            _neg_true,
            _resolve_and_true_left,
            _resolve_and_true_right,
            _resolve_or_right_true,
            _resolve_or_left_true,
            _unnecessary_block,
            _resolve_and_false_left,
            _resolve_and_false_right,
            _resolve_or_false_left,
            _resolve_or_false_right,
            _if_false,
            _remove_brackets_and,
            _remove_brackets_and_complement,
            _remove_brackets_or,
            _remove_brackets_or_complement,
            _elide_after_return,
            _resolve_ternary_true,
            _resolve_ternary_false,
        ];

        check_rules(rules)
    }
}
