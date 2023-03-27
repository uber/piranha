/*
Copyright (c) 2022 Uber Technologies, Inc.

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
    fmt.Println("not false 1")
    fmt.Println("not false 2")
    fmt.Println("else staying 1")
    fmt.Println("else staying 2")
}

// simplify `!true` and `!false` and also:
// true && something -> something
// something && true -> something
func simplify_true_and_something(something bool) {
    if something {
        fmt.Println("only something")
    }
    if something {
        fmt.Println("only something")
    }

    if something {
        fmt.Println("only something")
    }
    if something {
        fmt.Println("only something")
    }
}

// simplify `!true` and `!false` and also:
// false && something -> false
// something && false -> false
func simplify_false_and_something(something bool) {
    fmt.Println("else 1")
    fmt.Println("else 2")
    fmt.Println("else 3")
    fmt.Println("else 4")
    // selector_expression: simplify
    fmt.Println("else 5")
    // does not simplify binary_expression; left call may contain side-effects
    if exp.BoolValue("random") && false {
        fmt.Println("keep 1")
    } else {
        fmt.Println("keep 2")
    }

    // function call && false
    if f1() && false {
        fmt.Println("keep as it is")
    }

    // function call || true
    if f1() || true {
        fmt.Println("keep as it is")
    }
}

// simplify `!true` and `!false` and also:
// true || something -> true
// something || true -> true
func simplify_true_or_something(something bool) {
    fmt.Println("only true 1")
    fmt.Println("only true 2")
    fmt.Println("only true 3")
    fmt.Println("only true 4")
    // selector_expression: simplify
    fmt.Println("only true 5")
    // does not simplify binary_expression; left call may contain side-effects
    if exp.BoolValue("random") || true {
        fmt.Println("keep")
    }
}

// simplify `!true` and `!false` and also:
// false || something -> something
// something || false -> something
func simplify_false_or_something(something bool) {
    if something {
        fmt.Println("only something")
    }
    if something {
        fmt.Println("only something")
    }

    if something {
        fmt.Println("only something")
    }
    if something {
        fmt.Println("only something")
    }
}

// simplify `!true` and `!false` and also:
// if true { something } else { somethingElse } -> something
func simplify_if_statement_true() {
    fmt.Println("true 1")
    fmt.Println("true 2")
    fmt.Println("true 3")
}

func simplify_if_statement_false() {
    fmt.Println("remain")
    // no alternative, should remove the whole `if_statement`
}

func simplify_identity_eq() {
    fmt.Println("keep 1")
    fmt.Println("keep 2")
}

func simplify_identity_neq() {
    fmt.Println("keep 1")
    fmt.Println("keep 2")
}

// `nil == nil` not compilable in go
func simplify_identity_eq_nil() {
    fmt.Println("keep")
}

// `nil != nil` not compilable in go
func simplify_identity_neq_nil() {
    fmt.Println("keep")
}
