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

func testExpressions(ge GoExamples) {
	var x, y bool = false, false
	if x {
		fmt.Println("then-branch of `ge.flagMthds.treatedBehaviour(staleFlag) || x`")
	} else {
		fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) || x`")
	}
	fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) && x`")
	fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) && (x || y)`")
	fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) && (x && y)`")
	fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) && y == x`")
	fmt.Println("then-braanch of `ge.flagMthds.controlBehaviour(staleFlag) || y == x`")
	if y && x {
		fmt.Println("then-branch of `ge.flagMthds.controlBehaviour(staleFlag) && y && x`")
	} else {
		fmt.Println("else-branch of `ge.flagMthds.controlBehaviour(staleFlag) && y && x`")
	}
	fmt.Println("then-braanch of `ge.flagMthds.controlBehaviour(staleFlag) || y || x`")
}
