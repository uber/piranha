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
	if !exp.BoolValue("false") {
		fmt.Println("not false")
	}
	if !(exp.BoolValue("false")) {
		fmt.Println("not false")
	}

	if !exp.BoolValue("true") {
		fmt.Println("not true")
	}
	if !(exp.BoolValue("true")) {
		fmt.Println("not true")
	}
}

// simplify `!true` and `!false` and also:
// true && something -> true
// something && true -> true
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
// something && false -> true
func simplify_false_and_something(something bool) {
	if exp.BoolValue("false") && something {
		fmt.Println("only false")
	}
	if !exp.BoolValue("true") && something {
		fmt.Println("only false")
	}

	if something && exp.BoolValue("false") {
		fmt.Println("only false")
	}
	if something && !exp.BoolValue("true") {
		fmt.Println("only false")
	}
}

// simplify `!true` and `!false` and also:
// true || something -> true
// something || true -> true
func simplify_true_or_something(something bool) {
	if exp.BoolValue("true") || something {
		fmt.Println("only true")
	}
	if !exp.BoolValue("false") || something {
		fmt.Println("only true")
	}

	if something || exp.BoolValue("true") {
		fmt.Println("only true")
	}
	if something || !exp.BoolValue("false") {
		fmt.Println("only true")
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
