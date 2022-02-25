pub struct Rule {
    pub query: String,
    pub complete_capture_var: String,
    pub rewrite_template: String,
}


pub mod java_rules {
    use super::Rule;

    pub fn piranha_rules() -> Vec<Rule> {
        let r1 = Rule {
            query: String::from(
                "(
        (
            (method_invocation
                object: (method_invocation name: (identifier) @i2)
                name : (identifier) @i) @m)
            (#eq? @i getVal)
            (#eq? @i2 staleFlag)
        )",
            ),
            complete_capture_var: String::from("m"),
            rewrite_template: String::from("true"),
        };

        let r2 = Rule {
            query: String::from(
                "(
        (
            (method_invocation
                object: (method_invocation name: (identifier) @i2)
                name : (identifier) @i) @m)
            (#eq? @i getValN)
            (#eq? @i2 staleFlag)
        )",
            ),
            complete_capture_var: String::from("m"),
            rewrite_template: String::from("false"),
        };
        return vec![r1, r2];
    }

    pub fn cleanup_rules() -> Vec<Rule> {

        let _if_true = Rule {
            query: String::from(
                "
  ( (if_statement
		condition : ((parenthesized_expression ((true) @i)))
        consequence : ((statement) @s)
	)  @ifstmt)
    ",
            ),
            complete_capture_var: String::from("ifstmt"),
            rewrite_template: String::from("@s"),
        };

        let _if_else_false = Rule {
            query: String::from(
                "
   ( (if_statement
		condition : ((parenthesized_expression ((false) @i)))
        consequence : ((statement) @s)
        alternative : ((_) @el)
	)  @ifstmt)
    ",
            ),
            complete_capture_var: String::from("ifstmt"),
            rewrite_template: String::from("@el"),
        };

        let _if_false = Rule {
            query: String::from(
                "
    ((if_statement
		condition : ((parenthesized_expression ((false) @i)))
        consequence : ((statement) @s)
        !alternative
	)  @ifstmt)
    ",
            ),
            complete_capture_var: String::from("ifstmt"),
            rewrite_template: String::from(""),
        };

        let _ternary_op_true = Rule {
            query: String::from(
                "
  ( (if_statement
		condition : ((parenthesized_expression ((true) @i)))
        consequence : ((statement) @s)
	)  @ifstmt)
    ",
            ),
            complete_capture_var: String::from("ifstmt"),
            rewrite_template: String::from("@s"),
        };

        let _neg_false = Rule {
            query: String::from(
                "
  ((unary_expression
	operator: \"!\"
    operand: (false)) @u)
    ",
            ),
            complete_capture_var: String::from("u"),
            rewrite_template: String::from("true"),
        };

        let _neg_true = Rule {
            query: String::from(
                "
  ((unary_expression
	operator: \"!\"
    operand: (true)) @u)
    ",
            ),
            complete_capture_var: String::from("u"),
            rewrite_template: String::from("false"),
        };

        let _unnecessary_block = Rule {
            query: String::from(
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
            ),
            complete_capture_var: String::from("b1"),
            rewrite_template: String::from("{\n@pre\n@b\n@post\n}"),
        };

        let _resolve_and_true_left = Rule {
            query: String::from(
                // "((binary_expression left: (true) @l operator:\"&&\" right : (_) @other )) @b",
                "((binary_expression left: (true) operator:\"&&\" right : (_)* @other )) @b"
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("@other"),
        };

        let _resolve_and_true_right = Rule {
            query: String::from(
                "
        ((binary_expression
        left : (_)* @other
        operator:\"&&\"
        right: (true)
        ) @b)",
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("@other"),
        };

        let _resolve_or_right_true = Rule {
            query: String::from(
                "
        ((binary_expression
            left : (_)* @other
            operator:\"||\"
            right: (true)
        ) @b)",
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("true"),
        };

        let _resolve_or_left_true = Rule {
            query: String::from(
                "
        ((binary_expression
            left : (true)
            operator:\"||\"
            right: (_)* @other
        ) @b)",
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("true"),
        };

        let _resolve_and_false_left = Rule {
            query: String::from(
                "
        ((binary_expression
            left: (false)
            operator:\"&&\"
            right : (_)* @other
        ) @b)",
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("false"),
        };

        let _resolve_and_false_right = Rule {
            query: String::from(
                "
        ((binary_expression
            left : (_)* @other
            operator:\"&&\"
            right: (false)
        ) @b)",
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("false"),
        };

        let _resolve_or_false_left = Rule {
            query: String::from(
                "((binary_expression
            left : (_)* @other
            operator:\"||\"
            right: (false)
        ) @bo)
        ",
            ),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@other"),
        };

        let _resolve_or_false_right = Rule {
            query: String::from(
                "((binary_expression
            left : (false)
            operator:\"||\"
            right: (_)* @other
        ) @b)",
            ),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("@other"),
        };

        let _remove_brackets_or = Rule {
            query: String::from("(
(binary_expression
	left: ((parenthesized_expression (expression !left) @e))
    operator: \"||\"
    right: (_)* @r)
@bo)"),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@e || @r")
        };

        let _remove_brackets_or_complement = Rule {
            query: String::from("(binary_expression
    left: (_)* @l
    operator: \"||\"
	right: ((parenthesized_expression (expression !left) @e))
@bo)"),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@l || @e")
        };

        let _remove_brackets_and = Rule {
            query: String::from("(
(binary_expression
	left: ((parenthesized_expression (expression !left) @e))
    operator: \"&&\"
    right: (_)* @r)
@bo)"),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@e && @r")
        };

        let _remove_brackets_and_complement = Rule {
            query: String::from("(
(binary_expression
    left: (_)* @l
    operator: \"&&\"
	right: ((parenthesized_expression (expression !left) @e)))
    @bo
)"),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@l && @e")
        };

        let _remove_brackets_and_complement = Rule {
            query: String::from("(
(binary_expression
    left: (_)* @l
    operator: \"&&\"
	right: ((parenthesized_expression (expression !left) @e)))
    @bo
)"),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@l && @e")
        };

        let _remove_brackets_binary_op = Rule {
            query: String::from("(
(binary_expression
    left: (_)* @l
    operator: \"&&\"
	right: ((parenthesized_expression (expression !left) @e)))
    @bo
)"),
            complete_capture_var: String::from("bo"),
            rewrite_template: String::from("@l && @e")
        };

        let _elide_after_return = Rule {
            query : String::from("(
(block  ((statement)* @pre)
 ((return_statement) @r)
 ((statement)+ @post)) @b)"),
            complete_capture_var: String::from("b"),
            rewrite_template: String::from("{\n@pre\n@r\n}")
        };

        let _resolve_ternary_true = Rule {
            query : String::from("(ternary_expression condition: (true)
        consequence: (_)* @then
        alternative: (_)* @else) @t"),
            complete_capture_var: String::from("t"),
            rewrite_template: String::from("@then")
        };

        let _resolve_ternary_false = Rule {
            query : String::from("(ternary_expression condition: (true)
        consequence: (_)* @then
        alternative: (_)* @else) @t"),
            complete_capture_var: String::from("t"),
            rewrite_template: String::from("@else")
        };



        vec![
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
        ]
    }
}