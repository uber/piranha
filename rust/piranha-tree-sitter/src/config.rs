use crate::tree_sitter::get_language;
use std::env;
use std::fs;
use std::path::Path;
use tree_sitter::Query;
use serde_derive::Deserialize;

#[derive(Debug)]
pub struct ResolvedRule {
    pub name: String,
    pub query: Query,
    pub rewrite_template: String,
    pub chained_rule: Option<Rule>,
}

#[derive(Deserialize)]
struct Config {
    rule: Vec<Rule>
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub kind: String,
    pub query: String,
    pub replace: String,
    pub and_then : Option<Vec<Rule>>
}


pub fn read_config() -> (Vec<ResolvedRule>, Vec<ResolvedRule>) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_buf_to_java_toml = &manifest_dir.join("src").join("configurations")
        .join("java_rules.toml");

    let path_to_java_toml = path_buf_to_java_toml.as_path();
    println!("{:?}", &path_to_java_toml);

    let content = fs::read_to_string(&path_to_java_toml)
        .expect("Could not read the configuration file");

    let config: Config = toml::from_str(content.as_str()).unwrap();

    let mut xp_rules: Vec<ResolvedRule> = vec![];
    let mut cleanup_rules: Vec<ResolvedRule> = vec![];

    for r in config.rule {
        let rule = ResolvedRule::java_rule(&r.name,
                                           &r.query,
                                           &r.replace);

        if rule.is_err(){
            panic!("Please check the rule - {}", r.name);
        }
        if r.kind.eq("xp") {
            xp_rules.push(rule.unwrap());
        }else{
            cleanup_rules.push(rule.unwrap());
        }
    }

    return (xp_rules, cleanup_rules);
}



impl ResolvedRule {
    pub fn java_rule(
        name: &str,
        query_str: &str,
        rewrite_template: &str,
    ) -> Result<ResolvedRule, String> {
        let q = Query::new(get_language("Java"),
                           query_str);
        if q.is_err() {
            return Err(format!(
                "CONFIGURATION ERROR: The query is incorrect. Please check - \n{:?} ",
                query_str
            ));
        }
        let query = q.unwrap();
        let rewrite_template = String::from(rewrite_template);
        let r = ResolvedRule {
            name: String::from(name),
            query,
            rewrite_template,
            chained_rule: None,
        };
        Ok(r)
    }
}

// TODO Add all the checks for rules :
// (i) Rewrite rule should contain template variable from the query
// (ii) complete capture variable should be valid
// (iii)
// fn check_rules(rules: Vec<&Result<ResolvedRule, String>>) -> Vec<&ResolvedRule> {
//     let mut errorneous_rules = String::new();
//     let mut ok_rules = vec![];
//     for rule in rules {
//         match rule {
//             Ok(r) => {
//                 ok_rules.push(r);
//             }
//             Err(e) => errorneous_rules = errorneous_rules + e.as_str() + "\n",
//         }
//     }
//     if errorneous_rules.is_empty() {
//         return ok_rules;
//     }
//     panic!("{}", &errorneous_rules);
// }

// pub fn get_xp_api_rules(language: &str) -> Vec<&ResolvedRule> {
//     match language {
//         "Java" => check_rules(vec![&R1, &R2]),
//         _ => panic!("Language not supported"),
//     }
// }
//
// pub fn get_cleanup_rules(language: &str) -> Vec<&ResolvedRule> {
    // match language {
    //     "Java" => check_rules(vec![
    //         &_IF_TRUE,
    //         &_IF_ELSE_FALSE,
    //         &_neg_false,
    //         &_neg_true,
    //         &_resolve_and_true_left,
    //         &_resolve_and_true_right,
    //         &_resolve_or_right_true,
    //         &_resolve_or_left_true,
    //         &_unnecessary_block,
    //         &_resolve_and_false_left,
    //         &_resolve_and_false_right,
    //         &_resolve_or_false_left,
    //         &_resolve_or_false_right,
    //         &_if_false,
    //         &_REMOVE_BRACKETS_AND,
    //         &_REMOVE_BRACKETS_AND_COMPLEMENT,
    //         &_remove_brackets_or,
    //         &_remove_brackets_or_complement,
    //         &_elide_after_return,
    //         &_resolve_ternary_true,
    //         &_resolve_ternary_false,
    //     ]),
    //     _ => panic!("Language not supported"),
    // }
    // vec![]
// }

// pub mod java_rules {
//     use super::ResolvedRule;
//     use crate::configurations::check_rules;
//
// //     pub(crate) static R1: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(
// //     (
// //         (method_invocation
// //             object: (method_invocation name: (identifier) @i2)
// //             name : (identifier) @i) @m)
// //         (#eq? @i getVal)
// //         (#eq? @i2 staleFlag)
// //     )",
// //         "true",
// //     );
// //
// //     pub(crate) static R2: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(
// //     (
// //         (method_invocation
// //             object: (method_invocation name: (identifier) @i2)
// //             name : (identifier) @i) @m)
// //         (#eq? @i getValN)
// //         (#eq? @i2 staleFlag)
// //     )",
// //         "false",
// //     );
// //
// //     // pub(crate) fn get_java_cleanup_rules() -> Vec<Rule> {
// //     pub static _IF_TRUE: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// // ( (if_statement
// //     condition : ((parenthesized_expression ((true) @i)))
// //     consequence : ((statement) @s)
// // )  @ifstmt)
// // ",
// //         "@s",
// //     );
// //
// //     pub static _IF_ELSE_FALSE: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// // ( (if_statement
// //     condition : ((parenthesized_expression ((false) @i)))
// //     consequence : ((statement) @s)
// //     alternative : ((_) @el)
// // )  @ifstmt)",
// //         "@el",
// //     );
// //
// //     pub static _if_false: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// // ((if_statement
// //     condition : ((parenthesized_expression ((false) @i)))
// //     consequence : ((statement) @s)
// //     !alternative
// // )  @ifstmt)
// // ",
// //         "",
// //     );
// //
// //     pub static _ternary_op_true: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// // ( (if_statement
// //     condition : ((parenthesized_expression ((true) @i)))
// //     consequence : ((statement) @s)
// // )  @ifstmt)
// // ",
// //         "@s",
// //     );
// //
// //     pub static _neg_false: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// // ((unary_expression
// // operator: \"!\"
// // operand: (false)) @u)
// // ",
// //         "true",
// //     );
// //
// //     pub static _neg_true: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// // ((unary_expression
// // operator: \"!\"
// // operand: (true)) @u)
// // ",
// //         "false",
// //     );
// //
// //     pub static _unnecessary_block: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(
// // (block
// //   (
// //       (
// //     (_)*@pre
// //     (block (_)* @b)@n)
// //     (_)*@post
// //     )
// // )@b1
// // )",
// //         "{\n@pre\n@b\n@post\n}",
// //     );
// //
// //     pub static _resolve_and_true_left: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         // "((binary_expression left: (true) @l operator:\"&&\" right : (_) @other )) @b",
// //         "((binary_expression left: (true) operator:\"&&\" right : (_)* @other )) @b",
// //         "@other",
// //     );
// //
// //     pub static _resolve_and_true_right: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// //     ((binary_expression
// //     left : (_)* @other
// //     operator:\"&&\"
// //     right: (true)
// //     ) @b)",
// //         "@other",
// //     );
// //
// //     pub static _resolve_or_right_true: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// //     ((binary_expression
// //         left : (_)* @other
// //         operator:\"||\"
// //         right: (true)
// //     ) @b)",
// //         "true",
// //     );
// //
// //     pub static _resolve_or_left_true: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// //     ((binary_expression
// //         left : (true)
// //         operator:\"||\"
// //         right: (_)* @other
// //     ) @b)",
// //         "true",
// //     );
// //
// //     pub static _resolve_and_false_left: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// //     ((binary_expression
// //         left: (false)
// //         operator:\"&&\"
// //         right : (_)* @other
// //     ) @b)",
// //         "false",
// //     );
// //
// //     pub static _resolve_and_false_right: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "
// //     ((binary_expression
// //         left : (_)* @other
// //         operator:\"&&\"
// //         right: (false)
// //     ) @b)",
// //         "false",
// //     );
// //
// //     pub static _resolve_or_false_left: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "((binary_expression
// //         left : (_)* @other
// //         operator:\"||\"
// //         right: (false)
// //     ) @bo)
// //     ",
// //         "@other",
// //     );
// //
// //     pub static _resolve_or_false_right: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "((binary_expression
// //         left : (false)
// //         operator:\"||\"
// //         right: (_)* @other
// //     ) @b)",
// //         "@other",
// //     );
// //
// //     pub static _remove_brackets_or: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(
// //         (binary_expression
// //             left: ((parenthesized_expression (expression !left) @e))
// //             operator: \"||\"
// //             right: (_)* @r)
// //         @bo)",
// //         "@e || @r",
// //     );
// //
// //     pub static _remove_brackets_or_complement: Result<ResolvedRule, String> =
// //         ResolvedRule::java_rule(
// //             "(binary_expression
// //             left: (_)* @l
// //             operator: \"||\"
// //             right: ((parenthesized_expression (expression !left) @e))
// //         @bo)",
// //             "@l || @e",
// //         );
// //
// //     pub static _REMOVE_BRACKETS_AND: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(
// //         (binary_expression
// //             left: ((parenthesized_expression (expression !left) @e))
// //             operator: \"&&\"
// //             right: (_)* @r)
// //         @bo)",
// //         "@e && @r",
// //     );
// //
// //     pub static _REMOVE_BRACKETS_AND_COMPLEMENT: Result<ResolvedRule, String> =
// //         ResolvedRule::java_rule(
// //             "(
// //         (binary_expression
// //             left: (_)* @l
// //             operator: \"&&\"
// //             right: ((parenthesized_expression (expression !left) @e)))
// //             @bo
// //         )",
// //             "@l && @e",
// //         );
// //
// //     // pub static _REMOVE_BRACKETS_AND_COMPLEMENT:Result<Rule, String> = Rule::java_rule(
// //     //     "(
// //     // (binary_expression
// //     //     left: (_)* @l
// //     //     operator: \"&&\"
// //     //     right: ((parenthesized_expression (expression !left) @e)))
// //     //     @bo
// //     // )",
// //     //     "@l && @e",
// //     // );
// //
// //     // pub static _remove_brackets_binary_op: Result<Rule, String> = Rule::java_rule(
// //     //     "(
// //     //     (binary_expression
// //     //         left: (_)* @l
// //     //         operator: \"&&\"
// //     //         right: ((parenthesized_expression (expression !left) @e)))
// //     //         @bo
// //     //     )",
// //     //     "@l && @e",
// //     // );
// //
// //     pub static _elide_after_return: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(
// //         (block  ((statement)* @pre)
// //          ((return_statement) @r)
// //          ((statement)+ @post)) @b)",
// //         "{\n@pre\n@r\n}",
// //     );
// //
// //     pub static _resolve_ternary_true: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(ternary_expression condition: (true)
// //                 consequence: (_)* @then
// //                 alternative: (_)* @else) @t",
// //         "@then",
// //     );
// //
// //     pub static _resolve_ternary_false: Result<ResolvedRule, String> = ResolvedRule::java_rule(
// //         "(ternary_expression condition: (true)
// //                 consequence: (_)* @then
// //                 alternative: (_)* @else) @t",
// //         "@else",
// //     );
//
//     // check_rules(rules)
//     // }
// }
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
