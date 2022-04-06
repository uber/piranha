#[cfg(test)]
use std::fs::{self, DirEntry};
use std::path::Path;

use colored::Colorize;

use crate::config::PiranhaArguments;
use crate::piranha::get_cleanups_for_code_base_new;
use crate::utilities::read_file;

#[test]
fn test_java_scenarios_treated() {
    let language = "Java";
    let path_to_test_resource = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test-resources")
        .join("java");

    let c = get_cleanups_for_code_base_new(
        PiranhaArguments::new(
        path_to_test_resource.join("input").to_str().unwrap(),
        language,
        "STALE_FLAG",
        "some_long_name",   
        "true",
        "./../../configurations/",));
        
    let path_to_expected = path_to_test_resource.join("expected_treated");

    assert_eq!(c.len(), 4);

    for e in c {
        let file_name = e.0.file_name().unwrap();
        let f = get_file_with_name(
            path_to_expected.as_path().to_str().unwrap(),
            file_name.to_str().unwrap(),
        )
        .unwrap()
        .path();
        let expected_content = read_file(&f);
        let output = &e.1;
        let result = output
            .replace("\n", "")
            .eq(&expected_content.replace("\n", ""));
        if !result {
            println!("{:?}\n{}",file_name, output);
        }
        assert!(result);
        println!(
            "{}",
            format!("Test Result for {:?} is successful!!!", f.file_name()).bright_blue()
        );
    }

    pub fn has_name(dir_entry: &DirEntry, extension: &str) -> bool {
        dir_entry
            .path()
            .file_name()
            .map(|e| e.eq(extension))
            .unwrap_or(false)
    }

    pub fn get_file_with_name(input_dir: &str, name: &str) -> Option<DirEntry> {
        fs::read_dir(input_dir)
            .unwrap()
            .filter_map(|d| d.ok())
            .filter(|de| has_name(de, name))
            .next()
    }

    // assert_eq!(expected_content, e.1);
}

// pub fn _parse_code(language: Language, source_code: &String) -> (Parser, Tree) {
//     let mut parser = Parser::new();
//     parser
//         .set_language(language)
//         .expect("Could not set language");
//     let tree = parser
//         .parse(&source_code, None)
//         .expect("Could not parse code");
//     (parser, tree)
// }

// // TODO using sexp is not the most ideal way to test the expected and produced output
// #[test]
// fn test_simple_if() {
//     let piranha_args = get_piranha_arg();
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static void main(String a[]){
//     ab12.ba21();
//     if(exp.staleFlag().getVal()){
//         System.out.println(\"Hello World!\");
//     }
//     cp.put(exp.staleFlag(), true);
// }
// public void foo() {}
// }",
//         "Java", &piranha_args
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static void main(String a[]){
// ab12.ba21();

// System.out.println(\"Hello World!\");

// }
//     public void foo() {}
// }",
//         "Java"
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_simple_if_false() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static void main(String a[]){
//     ab12.ba21();
//     if(!exp.staleFlag().getVal()){
//         System.out.println(\"Hello World!\");
//     }else{
//         cp.put(exp.staleFlag(), false);
//         System.out.println(\"Hi World!\");
//     }
// }
// public void foo() {}
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static void main(String a[]){
// ab12.ba21();

// System.out.println(\"Hi World!\");

// }
//     public void foo() {}
// }",
//         "Java"
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_simple_if_or() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static void main(String a[]){
//     ab12.ba21();
//     if(exp.staleFlag().getVal() || someCondition()){
//         System.out.println(\"Hello World!\");
//     }
//     foobar();
//     if(exp.staleFlag().getVal() || someCondition()){
//         System.out.println(\"Hello World!\");
//     }

//     cp.put(exp.staleFlag(), \"abc\");

// }
// public void foo() {}
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static void main(String a[]){
// ab12.ba21();

// System.out.println(\"Hello World!\");

// foobar();

// System.out.println(\"Hello World!\");
// cp.put(exp.staleFlag(), \"abc\");

// }
//     public void foo() {}
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_simple_if_and() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static void main(String a[]){
//     ab12.ba21();
//     if(exp.staleFlag().getVal() || someCondition()){
//         System.out.println(\"Hello World!\");
//     }
//     foobar();
//     if(exp.staleFlag().getVal() && someCondition()){
//         System.out.println(\"Hello World!\");
//     }

// }
// public void foo() {}
// }",
//         "Java", &get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static void main(String a[]){
// ab12.ba21();

// System.out.println(\"Hello World!\");

// foobar();
// if(someCondition()){
//     System.out.println(\"Hello World!\");
// }
// }
//     public void foo() {}
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_complex_if_and() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static void main(String a[]){
//     ab12.ba21();
//     if(exp.staleFlag().getVal() || someCondition()){
//         System.out.println(\"Hello World!\");
//     }
//     foobar();
//     if(something() && (exp.staleFlag().getVal() && someCondition())){
//         System.out.println(\"Hello World!\");
//     }

// }
// public void foo() {}
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static void main(String a[]){
// ab12.ba21();

// System.out.println(\"Hello World!\");

// foobar();
// if(something() && someCondition()){
//     System.out.println(\"Hello World!\");
// }
// }
//     public void foo() {}
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_complex_if_and_or() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static void main(String a[]){
//     ab12.ba21();
//     if(exp.staleFlag().getVal() || someCondition()){
//         System.out.println(\"Hello World!\");
//     }
//     foobar();
//     if(something() && ((exp.staleFlag().getVal() || someCondition()) || abc()) ){
//         System.out.println(\"Hello World!\");
//     }

// }
// public void foo() {}
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static void main(String a[]){
// ab12.ba21();

// System.out.println(\"Hello World!\");

// foobar();
// if(something()){
//     System.out.println(\"Hello World!\");
// }
// }
//     public void foo() {}
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_simple_elide_after_return() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static int foobar(){
//     ab12.ba21();
//     if(exp.staleFlag().getVal()){
//         return 10;
//     }
//     int x = 0;
//     while(x < 5){
//         doSomething(x);
//         x ++;
//     }
// }
// public void foo() {}
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static int foobar(){
// ab12.ba21();
// return 10;
// }
//     public void foo() {}
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_simple_else_if_piranha_test_case() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static int remove_else_if(){
//        if (extra_toggle) {
//       return 0;
//     } else if (exp.staleFlag().getVal()) {
//       return 1;
//     } else {
//       return 2;
//     }
// }
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static int foobar(){
// if (extra_toggle) {
//       return 0;
//     } else {
//       return 1;
//     }
// }
// }",
//         "Java"
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_simple_else_if_no_block_piranha_test_case() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static int foobar(){
//        if (extra_toggle)
//       return 0;
//      else if (exp.staleFlag().getVal())
//       return 1;
//      else
//       return 2;

// }
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static int foobar(){
// if (extra_toggle)
//       return 0;
//      else
//       return 2;

// }
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_return_within_if_additional_piranha_test_case() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
// public static int foobar(){
//        if (x == 0) {
//       if (exp.staleFlag().getVal()) {
//         return 0;
//       }
//       return 75;
//     }

//     if (x == 1)
//       if (exp.staleFlag().getVal()) {
//         return 1;
//       } else {
//         return 76;
//       }

//     if (x == 2) {
//       int y = 3;
//       if (exp.staleFlag().getVal()) {
//         y++;
//         return y;
//       }
//       return y + 10;
//     }

//     if (x == 3) {
//       int z = 4;
//       if (exp.staleFlag().getVal()) {
//         z++;
//       } else {
//         z = z * 5;
//         return z + 10;
//       }
//       return z;
//     }
//     return 100;
// }
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public static int foobar(){
// if (x == 0) {
//       return 0;
//     }
//     if (x == 1) {
//         return 1;
//     }
//     if (x == 2) {
//       int y = 3;
//       y++;
//       return y;
//     }

//     if (x == 3) {
//       int z = 4;
//       z++;
//       return z;
//     }
//     return 100;

// }
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_if_piranha_test_case() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//  public void conditional_contains_stale_flag() {

//     if (exp.staleFlag().getVal()) {
//       System.out.println(\"Hello World\");
//     }
//   }

//   public void conditional_with_else_contains_stale_flag() {

//     if (exp.staleFlag().getVal()) {
//       System.out.println(\"Hello World\");
//     } else {
//       System.out.println(\"Hi world\");
//     }
//   }

//   public void complex_conditional_contains_stale_flag() {

//     if (true || (tBool && exp.staleFlag().getVal())) {
//       System.out.println(\"Hello World\");
//     } else {
//       System.out.println(\"Hi world\");
//     }
//   }

//   public void other_api_stale_flag() {

//     if (exp.staleFlag().getVal()) {
//       System.out.println(\"Hello World\");
//     } else {
//       System.out.println(\"Hi world\");
//     }
//   }
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public void conditional_contains_stale_flag() {
//     System.out.println(\"Hello World\");
//   }

//   public void conditional_with_else_contains_stale_flag() {
//     System.out.println(\"Hello World\");
//   }

//   public void complex_conditional_contains_stale_flag() {
//     System.out.println(\"Hello World\");
//   }

//   public void other_api_stale_flag() {
//     System.out.println(\"Hello World\");
//   }
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// //  TODO Discuss:
// // Original test case has last statement as
// // `tBool = true;` which is something we should not do unless
// // we are propagating value of `tBool
// #[test]
// fn test_assignments_containing_stale_flag_piranha_test_case() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//    public void assignments_containing_stale_flag() {

//     tBool = exp.staleFlag().getVal();

//     tBool = exp.staleFlag().getVal() && true;

//     tBool = exp.staleFlag().getVal() || true;

//     tBool = exp.staleFlag().getVal() || tBool;

//     tBool = exp.staleFlag().getVal() && (tBool || true);
//   }
// }",
//         "Java",&get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public void assignments_containing_stale_flag() {
//     tBool = true;

//     tBool = true;

//     tBool = true;

//     tBool = true;

//     tBool = (tBool || true);
//   }
// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_condexp_contains_stale_flag_piranha_test_case() {
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//    public void condexp_contains_stale_flag() {
//     // BUG: Diagnostic contains: Cleans stale XP flags
//     tBool = exp.staleFlag().getVal() ? true : false;
//   }
// }",
//         "Java", &get_piranha_arg()
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public void condexp_contains_stale_flag() {
//     // BUG: Diagnostic contains: Cleans stale XP flags
//     tBool = true;
//   }

// }",
//         "Java",
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_or_compounded_with_not_piranha_test_case() {
//     let p = PiranhaArguments {
//             flag_name: String::from("staleFlag"),
//             flag_value: false,
//             flag_namespace: String::from("ns"),
//         };
//     let transform_sexp = _get_transform_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//    public int or_compounded_with_not(int x, boolean extra_toggle) {
//     if (extra_toggle || !e.staleFlag().getVal()) {
//       return 0;
//     } else {
//       return 1;
//     }
//   }
// }",
//         "Java", &p
//     );

//     let expected_sexp = _get_expected_sexp(
//         "package com.uber.piranha;
// public class Foobar {
//     public int or_compounded_with_not(int x, boolean extra_toggle) {
//     return 0;
//   }

// }",
//         "Java"
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }

// #[test]
// fn test_method_declaration_delete(){
//     let p = PiranhaArguments { flag_name: String::from("safety_identity_verification_docscan_library_migration"), flag_value: true, flag_namespace: String::from("ns") };
//     let transform_sexp = _get_transform_sexp(
//         " @ParameterDefinitions(namespace = \"trusted_identity_mobile\")
//         public interface IdentityVerificationFlowDocScanParameters {

//             @BoolParam(key = \"safety_identity_verification_docscan_library_migration\", namespace=\"ns\")
//             BoolParameter safetyIdentityVerificationDocscanLibraryMigration();

//             @BoolParam(key = \"minors_docscan_support\", namespace=\"ns\")
//             BoolParameter isMinorsDocScanSupportOn();

//           static IdentityVerificationFlowDocScanParameters create(CachedParameters cachedParameters) {
//             return ParameterUtils.create(IdentityVerificationFlowDocScanParameters.class, cachedParameters);
//           }
//         }",
//         "Java", &p

//     );

//     let expected_sexp = _get_expected_sexp(
//         "@ParameterDefinitions(namespace = \"trusted_identity_mobile\")
//         public interface IdentityVerificationFlowDocScanParameters {

//           @BoolParam(key = \"minors_docscan_support\", namespace=\"ns\")
//           BoolParameter isMinorsDocScanSupportOn();

//           static IdentityVerificationFlowDocScanParameters create(CachedParameters cachedParameters) {
//             return ParameterUtils.create(IdentityVerificationFlowDocScanParameters.class, cachedParameters);
//           }
//         }",
//         "Java"
//     );
//     assert_eq!(expected_sexp, transform_sexp)
// }
// //PiranhaArguments { flag_name: String::from("safety_identity_verification_docscan_library_migration"), flag_value: true, flag_namespace: String::from("ns") }
// fn _get_transform_sexp(input_src_code: &str, input_language: &str, piranha_args : &PiranhaArguments) -> String {
//     let input = String::from(input_src_code);
//     let (tree, _output) = get_cleanups_for_file_content(&input, input_language, piranha_args);
//     let transform_sexp = tree.root_node().to_sexp();
//     println!("{}", _output.as_str());
//     transform_sexp
// }

// fn _get_expected_sexp(expected_source_code: &str, language: &str) -> String {
//     let l = get_language(language);
//     let expected_code = String::from(expected_source_code);
//     let (_p, t) = parse_code(l, &expected_code);
//     let expected_sexp = t.root_node().to_sexp();
//     expected_sexp
// }
