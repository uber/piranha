/*
Copyright (c) 2021 Uber Technologies, Inc.
Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file

except in compliance with the License. You may obtain a copy of the License at
http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the

License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
express or implied. See the License for the specific language governing permissions and
limitations under the License.
*/
package testfiles

import "fmt"

func testSwitch(ge GoExamples) {
	fmt.Println("defaulter 1")

	switch os := ge.flagMthds.treatedBehaviour(newFlag); os {
	case true:
		fmt.Println("mike treat 2")
	default:
		fmt.Println("defaulter 2")
	}

	fmt.Println("mike treat 3")
	fmt.Println("mike treat 4")

	x := true
	y := false

	fmt.Println("defaulter 5")
	fmt.Println("mike control 6")

	switch os := true; x || y {
	case true:
		fmt.Println("mike treat 7")
	default:
		fmt.Println("mike control 7")
	}

	print("treated")
	x = true
	fmt.Println("all are true")

	/*
		If you are familiar with go progamming then you probably know that
		this style of writing switch statments is like writing if-elseif-else
		statements in C, C++, Java or Python.

		So, to refactor this code we will consider three cases:
		(ith case statement means position of the case statement from first case)

		Case1:
		If ith case statement evaluates to true and all case statements
		above this statement evaluates to false. (This also includes if 1st case
		statement evaluates to true).
		Do:
		Then we will remove whole switch statment and replace it with body
		of ith case statment.

		Case2:
		If ith case statement evaluates to true and all case statements
		above this statement evaluates to isBot(it means cannot determine
		whether it is true or false).
		Do:
		Then we will remove all the cases below the ith case statement and
		rewrite the switch statement with ith case statment as default. (check
		corresponding refactored output without -treated of this file in
		'output/control' directory.)

		Case3:
		If ith case statment is false.
		Do:
		Then remove it from the switch statement in refactored output.
	*/
	x = true
	y = false

	// Switch 1
	switch {
	case x:
		fmt.Println("switch test 1")
	default:
		fmt.Println("switch test 6")
	}

	fmt.Println("switch test 13")

	// Switch 3
	switch {
	case y && x:
		fmt.Println("switch test 16")
	}
}
