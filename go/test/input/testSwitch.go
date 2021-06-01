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
	switch os := ge.flagMthds.treatedBehaviour(staleFlag); os {
	case true:
		fmt.Println("mike treat 1")
	default:
		fmt.Println("defaulter 1")
	}

	switch os := ge.flagMthds.treatedBehaviour(newFlag); os {
	case true:
		fmt.Println("mike treat 2")
	default:
		fmt.Println("defaulter 2")
	}

	switch !ge.flagMthds.treatedBehaviour(staleFlag) {
	case true:
		fmt.Println("mike treat 3")
	default:
		fmt.Println("defaulter 3")
	}

	switch os := ge.flagMthds.controlBehaviour(staleFlag); os {
	case true:
		fmt.Println("mike treat 4")
	default:
		fmt.Println("defaulter 4")
	}

	x := true
	y := false

	switch os := ge.flagMthds.treatedBehaviour(staleFlag) && (x || y); os {
	case true:
		fmt.Println("mike treat 5")
	default:
		fmt.Println("defaulter 5")
	}

	switch os := ge.flagMthds.treatedBehaviour(staleFlag); os && (x || y) {
	case true:
		fmt.Println("mike treat 6")
	default:
		fmt.Println("mike control 6")
	}

	switch os := ge.flagMthds.controlBehaviour(staleFlag); os && (x || y) {
	case true:
		fmt.Println("mike treat 7")
	default:
		fmt.Println("mike control 7")
	}

	switch ge.flagMthds.treatedBehaviour(staleFlag) || x || y {
	case true:
		print("treated")
		x = !ge.flagMthds.treatedBehaviour(staleFlag) || y
	case false:
		print("control")
		y = !ge.flagMthds.treatedBehaviour(staleFlag) || x
	}

	switch ge.flagMthds.controlBehaviour(staleFlag) && x && y {
	case true:
		fmt.Println("all are true")
	case false:
		fmt.Println("someone is false")
	}

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
	case ge.flagMthds.treatedBehaviour(staleFlag) || x:
		fmt.Println("switch test 1")
	case ge.flagMthds.treatedBehaviour(staleFlag) && x:
		fmt.Println("switch test 2")
	case ge.flagMthds.treatedBehaviour(staleFlag) && (x || y):
		fmt.Println("switch test 3")
	case ge.flagMthds.treatedBehaviour(staleFlag) && (x && y):
		fmt.Println("switch test 4")
	case ge.flagMthds.treatedBehaviour(staleFlag) && y == x:
		fmt.Println("switch test 5")
		x = y || ge.flagMthds.treatedBehaviour(newFlag)
	case ge.flagMthds.controlBehaviour(staleFlag) || y == x:
		fmt.Println("switch test 6")
	case ge.flagMthds.controlBehaviour(staleFlag) && y && x:
		fmt.Println("switch test 7")
	case ge.flagMthds.controlBehaviour(staleFlag) || y || x:
		fmt.Println("switch test 8")
	}

	// Switch 2
	switch {
	case ge.flagMthds.treatedBehaviour(staleFlag) && x:
		fmt.Println("switch test 9")
	case ge.flagMthds.treatedBehaviour(staleFlag) && (x || y):
		fmt.Println("switch test 10")
	case ge.flagMthds.treatedBehaviour(staleFlag) && (x && y):
		fmt.Println("switch test 11")
	case ge.flagMthds.treatedBehaviour(staleFlag) && y == x:
		fmt.Println("switch test 12")
		x = y || ge.flagMthds.treatedBehaviour(newFlag)
	case ge.flagMthds.controlBehaviour(staleFlag) || y == x:
		fmt.Println("switch test 13")
	case ge.flagMthds.controlBehaviour(staleFlag) && y && x:
		fmt.Println("switch test 14")
	case ge.flagMthds.controlBehaviour(staleFlag) || y || x:
		fmt.Println("switch test 15")
	}

	// Switch 3
	switch {
	case ge.flagMthds.controlBehaviour(staleFlag) && y && x:
		fmt.Println("switch test 16")
	}
}
