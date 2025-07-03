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

func a() bool {
    enabled := exp.BoolValue("true")

    return enabled
}

func b() string {
    enabled := exp.BoolValue("true")

    s, err := exp.StrValue("str")
    if err != nil {
        fmt.Println(err)
    }

    if enabled {
        return s
    } else {
        return "prefix_" + s
    }
}

func c() string {
    enabled := exp.BoolValue("false")

    s, err := exp.StrValue("str")
    if err != nil {
        fmt.Println(err)
    }

    if enabled {
        return s
    } else {
        fmt.Println("not enabled")
        return "prefix_" + s
    }
}

func after_return1() string {
    enabled := exp.BoolValue("false")
    if !enabled {
        return "not enabled"
    }

    return "enabled"
}

func after_return_nil_1() string {
    enabled, err := exp.BoolValue("false"), nil
    if enabled {
        return "enabled"
    }

    if err == nil {
        fmt.Println("Retain")
    }

    return "not enabled"
}


func after_return_nil_2() string {
    enabled, err := exp.BoolValue("true"), nil
    if !enabled {
        return "enabled"
    }

    if err != nil {
        fmt.Println("Retain")
    }

    return "not enabled"
}

func after_return2(a bool) string {
    if a {
        enabled := exp.BoolValue("false")
        if !enabled {
            fmt.Println("not enabled")
            return "not enabled"
        }
        fmt.Println("should be removed")
    }
    // delete after return needs to consider blocks
    fmt.Println("should not be removed")
    return "keep"
}

// should remove multiple statements after return
func after_return3() string {
    enabled := exp.BoolValue("false")
    if !enabled {
        return "not enabled"
    }

    fmt.Println("1")
    fmt.Println("2")
    fmt.Println("3")
    fmt.Println("4")
    fmt.Println("5")
    return "enabled"
}

func after_return4() string {
    fmt.Println("before 1")
    fmt.Println("before 2")

    enabled := exp.BoolValue("false")
    if !enabled {
        return "not enabled"
    }

    fmt.Println("1")
    fmt.Println("2")
    fmt.Println("3")
    fmt.Println("4")
    fmt.Println("5")
    return "enabled"
}


func simplify_if_statement_false_comment_demo_single_comment() {
    if exp.BoolValue("false") {
        fmt.Println("to be removed")
    } else {
        fmt.Println("remain")
    }
    // this comment will be deleted
    if exp.BoolValue("false") {
        fmt.Println("to be removed 2")
    }
}


func simplify_if_statement_false_comment_demo_double_comment() {
    if exp.BoolValue("false") {
        fmt.Println("to be removed")
    } else {
        fmt.Println("remain")
    }
    // these comments
    // will be deleted
    if exp.BoolValue("false") {
        fmt.Println("to be removed 2")
    }
}

func simplify_if_statement_false_comment_demo_multiline_comment() {
    if exp.BoolValue("false") {
        fmt.Println("to be removed")
    } else {
        fmt.Println("remain")
    }
    /* this comment does get removed
    with all the lines
    in it 
    */
    if exp.BoolValue("false") {
        fmt.Println("to be removed 2")
    }
}

func simplify_if_statement_false_comment_demo_multiline_comment_one_line() {
    if exp.BoolValue("false") {
        fmt.Println("to be removed")
    } else {
        fmt.Println("remain")
    }
    /* this comment will be deleted */
    if exp.BoolValue("false") {
        fmt.Println("to be removed 2")
    }
}
