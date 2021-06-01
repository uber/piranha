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
	fmt.Println("control1")
	fmt.Println("control2")
	fmt.Println("control3")

	if ge.flagMthds.treatedBehaviour(localFlag) || globalFeature(globalFlag) {
		fmt.Println("treat1 with ||")
	} else {
		fmt.Println("control1 with ||")
	}
	fmt.Println("treat4")
	fmt.Println("treat5")
	fmt.Println("treat6")

	if ge.flagMthds.controlBehaviour(localFlag) && globalFeature(globalFlag) {
		fmt.Println("treat4 with &&")
	} else {
		fmt.Println("control4 with &&")
	}

	//comment1
	a := false //comment2
	//comment3
	b := ge.flagMthds.treatedBehaviour(newFlag) //comment4
	fmt.Println("control4 on a && b")

	if b {
		fmt.Println("treat5 on a || b")
	} else {
		fmt.Println("control5 on a || b")
	}

	fmt.Println("control6 with v")
	fmt.Println("control7 with v")
	fmt.Println("treat8 with v")
}
