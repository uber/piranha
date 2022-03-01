use crate::piranha::transform;
use crate::tree_sitter::{get_language, parse_code};

// TODO using sexp is not the most ideal way to test the expected and produced output
#[test]
fn test_simple_if() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    print!("Manifest Dir {}", manifest_dir);

    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static void main(String a[]){
    ab12.ba21();
    if(exp.staleFlag().getVal()){
        System.out.println(\"Hello World!\");
    }
}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static void main(String a[]){
ab12.ba21();

System.out.println(\"Hello World!\");


}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_simple_if_false() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static void main(String a[]){
    ab12.ba21();
    if(!exp.staleFlag().getVal()){
        System.out.println(\"Hello World!\");
    }else{
        System.out.println(\"Hi World!\");
    }
}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static void main(String a[]){
ab12.ba21();

System.out.println(\"Hi World!\");


}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_simple_if_or() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static void main(String a[]){
    ab12.ba21();
    if(exp.staleFlag().getVal() || someCondition()){
        System.out.println(\"Hello World!\");
    }
    foobar();
    if(exp.staleFlag().getVal() || someCondition()){
        System.out.println(\"Hello World!\");
    }

}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static void main(String a[]){
ab12.ba21();

System.out.println(\"Hello World!\");

foobar();

System.out.println(\"Hello World!\");

}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_simple_if_and() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static void main(String a[]){
    ab12.ba21();
    if(exp.staleFlag().getVal() || someCondition()){
        System.out.println(\"Hello World!\");
    }
    foobar();
    if(exp.staleFlag().getVal() && someCondition()){
        System.out.println(\"Hello World!\");
    }

}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static void main(String a[]){
ab12.ba21();

System.out.println(\"Hello World!\");

foobar();
if(someCondition()){
    System.out.println(\"Hello World!\");
}
}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_complex_if_and() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static void main(String a[]){
    ab12.ba21();
    if(exp.staleFlag().getVal() || someCondition()){
        System.out.println(\"Hello World!\");
    }
    foobar();
    if(something() && (exp.staleFlag().getVal() && someCondition())){
        System.out.println(\"Hello World!\");
    }

}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static void main(String a[]){
ab12.ba21();

System.out.println(\"Hello World!\");

foobar();
if(something() && someCondition()){
    System.out.println(\"Hello World!\");
}
}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_complex_if_and_or() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static void main(String a[]){
    ab12.ba21();
    if(exp.staleFlag().getVal() || someCondition()){
        System.out.println(\"Hello World!\");
    }
    foobar();
    if(something() && ((exp.staleFlag().getVal() || someCondition()) || abc()) ){
        System.out.println(\"Hello World!\");
    }

}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static void main(String a[]){
ab12.ba21();

System.out.println(\"Hello World!\");

foobar();
if(something()){
    System.out.println(\"Hello World!\");
}
}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_simple_elide_after_return() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static int foobar(){
    ab12.ba21();
    if(exp.staleFlag().getVal()){
        return 10;
    }
    int x = 0;
    while(x < 5){
        doSomething(x);
        x ++;
    }
}
public void foo() {}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static int foobar(){
ab12.ba21();
return 10;
}
    public void foo() {}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_simple_else_if_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static int remove_else_if(){
       if (extra_toggle) {
      return 0;
    } else if (exp.staleFlag().getVal()) {
      return 1;
    } else {
      return 2;
    }
}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static int foobar(){
if (extra_toggle) {
      return 0;
    } else {
      return 2;
    }
}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_simple_else_if_no_block_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static int foobar(){
       if (extra_toggle)
      return 0;
     else if (exp.staleFlag().getVal())
      return 1;
     else
      return 2;

}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static int foobar(){
if (extra_toggle)
      return 0;
     else
      return 2;

}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_return_within_if_additional_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
public static int foobar(){
       if (x == 0) {
      if (exp.staleFlag().getVal()) {
        return 0;
      }
      return 75;
    }

    if (x == 1)
      if (exp.staleFlag().getVal()) {
        return 1;
      } else {
        return 76;
      }

    if (x == 2) {
      int y = 3;
      if (exp.staleFlag().getVal()) {
        y++;
        return y;
      }
      return y + 10;
    }

    if (x == 3) {
      int z = 4;
      if (exp.staleFlag().getVal()) {
        z++;
      } else {
        z = z * 5;
        return z + 10;
      }
      return z;
    }
    return 100;
}
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public static int foobar(){
if (x == 0) {
      return 0;
    }
    if (x == 1) {
        return 1;
    }
    if (x == 2) {
      int y = 3;
      y++;
      return y;
    }

    if (x == 3) {
      int z = 4;
      z++;
      return z;
    }
    return 100;

}
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_if_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
 public void conditional_contains_stale_flag() {

    if (exp.staleFlag().getVal()) {
      System.out.println(\"Hello World\");
    }
  }

  public void conditional_with_else_contains_stale_flag() {

    if (exp.staleFlag().getVal()) {
      System.out.println(\"Hello World\");
    } else {
      System.out.println(\"Hi world\");
    }
  }

  public void complex_conditional_contains_stale_flag() {

    if (true || (tBool && exp.staleFlag().getVal())) {
      System.out.println(\"Hello World\");
    } else {
      System.out.println(\"Hi world\");
    }
  }

  public void other_api_stale_flag() {

    if (exp.staleFlag().getVal()) {
      System.out.println(\"Hello World\");
    } else {
      System.out.println(\"Hi world\");
    }
  }
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public void conditional_contains_stale_flag() {
    System.out.println(\"Hello World\");
  }

  public void conditional_with_else_contains_stale_flag() {
    System.out.println(\"Hello World\");
  }

  public void complex_conditional_contains_stale_flag() {
    System.out.println(\"Hello World\");
  }

  public void other_api_stale_flag() {
    System.out.println(\"Hello World\");
  }
}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

//  TODO temporal propagation of patterns
#[test]
fn test_assignments_containing_stale_flag_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
   public void assignments_containing_stale_flag() {

    tBool = exp.staleFlag().getVal();

    tBool = exp.staleFlag().getVal() && true;

    tBool = exp.staleFlag().getVal() || true;

    tBool = exp.staleFlag().getVal() || tBool;

    tBool = exp.staleFlag().getVal() && (tBool || true);
  }
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public void assignments_containing_stale_flag() {
    tBool = true;

    tBool = true;

    tBool = true;

    tBool = true;

    tBool = true;
  }

}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

#[test]
fn test_condexp_contains_stale_flag_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
   public void condexp_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = exp.staleFlag().getVal() ? true : false;
  }
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public void condexp_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = true;
  }

}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

// getValN is a hack .
// Eventually from the tests we shuld swap staleFlaag().getVal() to false
#[test]
fn test_or_compounded_with_not_piranha_test_case() {
    let transform_sexp = _get_transform_sexp(
        "package com.uber.piranha;
public class Foobar {
   public int or_compounded_with_not(int x, boolean extra_toggle) {
    if (extra_toggle || !e.staleFlag().getValN()) {
      return 0;
    } else {
      return 1;
    }
  }
}",
        "Java",
    );

    let expected_sexp = _get_expected_sexp(
        "package com.uber.piranha;
public class Foobar {
    public int or_compounded_with_not(int x, boolean extra_toggle) {
    return 0;
  }

}",
        "Java",
    );
    assert_eq!(expected_sexp, transform_sexp)
}

fn _get_transform_sexp(input_src_code: &str, language: &str) -> String {
    let input = String::from(input_src_code);
    let (tree, _output) = transform(&input, language);
    let transform_sexp = tree.root_node().to_sexp();
    println!("{}", _output.as_str());
    transform_sexp
}

fn _get_expected_sexp(expected_source_code: &str, language: &str) -> String {
    let l = get_language(language);
    let expected_code = String::from(expected_source_code);
    let (_p, t) = parse_code(l, &expected_code);
    let expected_sexp = t.root_node().to_sexp();
    expected_sexp
}
