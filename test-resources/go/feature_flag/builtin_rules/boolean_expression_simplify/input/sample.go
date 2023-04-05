/*
Copyright (c) 2023 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

package main

import "fmt"

// Simplifying `!true` and `!false`.
// It will eventually be a part of larger cleanup in upcoming tests.
func simplify_not() {
    if !exp.BoolValue("false") {
        fmt.Println("not false 1")
    } else {
        fmt.Println("cleaned up else")
    }
    if !(exp.BoolValue("false")) {
        fmt.Println("not false 2")
    } else {
        fmt.Println("cleaned up else")
    }

    if !exp.BoolValue("true") {
        fmt.Println("not true")
    } else {
        fmt.Println("else staying 1")
    }
    if !(exp.BoolValue("true")) {
        fmt.Println("not true")
    } else {
        fmt.Println("else staying 2")
    }
}

// simplify `!true` and `!false` and also:
// true && something -> something
// something && true -> something
func simplify_true_and_something(something bool) {
    if exp.BoolValue("true") && something {
        fmt.Println("only something")
    }
    if !exp.BoolValue("false") && something {
        fmt.Println("only something")
    }

    if something && exp.BoolValue("true") {
        fmt.Println("only something")
    }
    if something && !exp.BoolValue("false") {
        fmt.Println("only something")
    }
}

// simplify `!true` and `!false` and also:
// false && something -> false
// something && false -> false
func simplify_false_and_something(something bool) {
    if exp.BoolValue("false") && something {
        fmt.Println("only false")
    } else {
        fmt.Println("else 1")
    }
    if !exp.BoolValue("true") && something {
        fmt.Println("only false")
    } else {
        fmt.Println("else 2")
    }

    if something && exp.BoolValue("false") {
        fmt.Println("only false")
    } else {
        fmt.Println("else 3")
    }
    if something && !exp.BoolValue("true") {
        fmt.Println("only false")
    } else {
        fmt.Println("else 4")
    }
    // selector_expression: simplify
    if exp.BoolConst && exp.BoolValue("false") {
        fmt.Println("nope")
    } else {
        fmt.Println("else 5")
    }
    // does not simplify binary_expression; left call may contain side-effects
    if exp.BoolValue("random") && exp.BoolValue("false") {
        fmt.Println("keep 1")
    } else {
        fmt.Println("keep 2")
    }
    // function call && false
    if f1() && exp.BoolValue("false") {
        fmt.Println("keep as it is")
    }

    // function call || true
    if f1() || exp.BoolValue("true") {
        fmt.Println("keep as it is")
    }
}

// simplify `!true` and `!false` and also:
// true || something -> true
// something || true -> true
func simplify_true_or_something(something bool) {
    if exp.BoolValue("true") || something {
        fmt.Println("only true 1")
    }
    if !exp.BoolValue("false") || something {
        fmt.Println("only true 2")
    }

    if something || exp.BoolValue("true") {
        fmt.Println("only true 3")
    }
    if something || !exp.BoolValue("false") {
        fmt.Println("only true 4")
    }
    // selector_expression: simplify
    if exp.BoolConst || exp.BoolValue("true") {
        fmt.Println("only true 5")
    }

    // does not simplify binary_expression; left call may contain side-effects
    if exp.BoolValue("random") || exp.BoolValue("true") {
        fmt.Println("keep")
    }
}

// simplify `!true` and `!false` and also:
// false || something -> something
// something || false -> something
func simplify_false_or_something(something bool) {
    if exp.BoolValue("false") || something {
        fmt.Println("only something")
    }
    if !exp.BoolValue("true") || something {
        fmt.Println("only something")
    }

    if something || exp.BoolValue("false") {
        fmt.Println("only something")
    }
    if something || !exp.BoolValue("true") {
        fmt.Println("only something")
    }
}

// simplify `!true` and `!false` and also:
// if true { something } else { somethingElse } -> something
func simplify_if_statement_true() {
    if exp.BoolValue("true") {
        fmt.Println("true 1")
    }

    if exp.BoolValue("true") {
        fmt.Println("true 2")
    } else {
        fmt.Println("false")
    }

    if !exp.BoolValue("false") {
        fmt.Println("true 3")
    } else {
        fmt.Println("false 2")
    }
}

func simplify_if_statement_false() {
    if exp.BoolValue("false") {
        fmt.Println("to be removed")
    } else {
        fmt.Println("remain")
    }
    // no alternative, should remove the whole `if_statement`
    if exp.BoolValue("false") {
        fmt.Println("to be removed 2")
    }
}

func simplify_identity_eq() {
    if exp.BoolValue("true") == exp.BoolValue("true") {
        fmt.Println("keep 1")
    } else {
        fmt.Println("remove 1")
    }
    if exp.BoolValue("false") == exp.BoolValue("false") {
        fmt.Println("keep 2")
    } else {
        fmt.Println("remove 2")
    }
}

func simplify_identity_neq() {
    if exp.BoolValue("true") != exp.BoolValue("true") {
        fmt.Println("remove 1")
    } else {
        fmt.Println("keep 1")
    }
    if exp.BoolValue("false") != exp.BoolValue("false") {
        fmt.Println("remove 2")
    } else {
        fmt.Println("keep 2")
    }
}

// `nil == nil` not compilable in go
func simplify_identity_eq_nil() {
    if exp.Value("nil") == nil {
        fmt.Println("keep")
    } else {
        fmt.Println("remove")
    }
}

// `nil != nil` not compilable in go
func simplify_identity_neq_nil() {
    if exp.Value("nil") != nil {
        fmt.Println("remove")
    } else {
        fmt.Println("keep")
    }
}
