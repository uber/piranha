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

func testIfConditionalAndInitExpressions(ge GoExamples) {
	fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) && ge.flagMthds.treatedBehaviour(localFlag) && globalFeature(globalFlag)`")
	fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) && globalFeature(globalFlag) && ge.flagMthds.treatedBehaviour(localFlag)`")
	fmt.Println("else-branch of `globalFeature(globalFlag) && ge.flagMthds.treatedBehaviour(staleFlag) && ge.flagMthds.treatedBehaviour(localFlag)`")

	if ge.flagMthds.treatedBehaviour(localFlag) || globalFeature(globalFlag) {
		fmt.Println("then-branch of `ge.flagMthds.treatedBehaviour(staleFlag) || ge.flagMthds.treatedBehaviour(localFlag) || globalFeature(globalFlag)`")
	} else {
		fmt.Println("else-branch of `ge.flagMthds.treatedBehaviour(staleFlag) || ge.flagMthds.treatedBehaviour(localFlag) || globalFeature(globalFlag)`")
	}

	fmt.Println("then-branch of `ge.flagMthds.controlBehaviour(staleFlag) || ge.flagMthds.controlBehaviour(localFlag) || globalFeature(globalFlag)`")
	fmt.Println("then-branch of `ge.flagMthds.controlBehaviour(staleFlag) || globalFeature(globalFlag) || ge.flagMthds.controlBehaviour(localFlag)`")
	fmt.Println("then-branch of `globalFeature(globalFlag) || ge.flagMthds.controlBehaviour(staleFlag) || ge.flagMthds.controlBehaviour(localFlag)`")

	if ge.flagMthds.controlBehaviour(localFlag) && globalFeature(globalFlag) {
		fmt.Println("then-branch of `ge.flagMthds.controlBehaviour(staleFlag) && ge.flagMthds.controlBehaviour(localFlag) && globalFeature(globalFlag)`")
	} else {
		fmt.Println("else-branch of `ge.flagMthds.controlBehaviour(staleFlag) && ge.flagMthds.controlBehaviour(localFlag) && globalFeature(globalFlag)`")
	}

	//comment1
	a := false //comment2
	//comment3
	b := ge.flagMthds.treatedBehaviour(newFlag) //comment4
	fmt.Println("else-branch of `a && b`")

	if b {
		fmt.Println("then-branch of `a || b`")
	} else {
		fmt.Println("else-branch of `a || b`")
	}

	fmt.Println("else-branch of `v := ge.flagMthds.treatedBehaviour(staleFlag); v`")
	fmt.Println("else-branch of `v := ge.flagMthds.treatedBehaviour(staleFlag); v == true`")
	fmt.Println("then-branch of `v := ge.flagMthds.treatedBehaviour(staleFlag); v != true`")
}
