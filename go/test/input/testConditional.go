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
	// Treated Control behaviour will be governed by three or more flags
	if ge.flagMthds.treatedBehaviour(staleFlag) && ge.flagMthds.treatedBehaviour(localFlag) && globalFeature(globalFlag) {
		fmt.Println("treat1")
	} else {
		fmt.Println("control1")
	}

	if ge.flagMthds.treatedBehaviour(staleFlag) && globalFeature(globalFlag) && ge.flagMthds.treatedBehaviour(localFlag) {
		fmt.Println("treat2")
	} else {
		fmt.Println("control2")
	}

	if globalFeature(globalFlag) && ge.flagMthds.treatedBehaviour(staleFlag) && ge.flagMthds.treatedBehaviour(localFlag) {
		fmt.Println("treat3")
	} else {
		fmt.Println("control3")
	}

	if ge.flagMthds.treatedBehaviour(staleFlag) || ge.flagMthds.treatedBehaviour(localFlag) || globalFeature(globalFlag) {
		fmt.Println("treat1 with ||")
	} else {
		fmt.Println("control1 with ||")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) || ge.flagMthds.controlBehaviour(localFlag) || globalFeature(globalFlag) {
		fmt.Println("treat4")
	} else {
		fmt.Println("control4")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) || globalFeature(globalFlag) || ge.flagMthds.controlBehaviour(localFlag) {
		fmt.Println("treat5")
	} else {
		fmt.Println("control5")
	}

	if globalFeature(globalFlag) || ge.flagMthds.controlBehaviour(staleFlag) || ge.flagMthds.controlBehaviour(localFlag) {
		fmt.Println("treat6")
	} else {
		fmt.Println("control6")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) && ge.flagMthds.controlBehaviour(localFlag) && globalFeature(globalFlag) {
		fmt.Println("treat4 with &&")
	} else {
		fmt.Println("control4 with &&")
	}

	//comment1
	a := ge.flagMthds.commonBehaviour("user", staleFlag) //comment2
	//comment3
	b := ge.flagMthds.treatedBehaviour(newFlag) //comment4
	//comment5
	if a && b {
		fmt.Println("treat4 on a && b")
	} else {
		fmt.Println("control4 on a && b")
	}

	if a || b {
		fmt.Println("treat5 on a || b")
	} else {
		fmt.Println("control5 on a || b")
	}

	if v := ge.flagMthds.treatedBehaviour(staleFlag); v {
		fmt.Println("treat6 with v")
	} else {
		fmt.Println("control6 with v")
	}

	if v := ge.flagMthds.treatedBehaviour(staleFlag); v == true {
		b = a && b && !b
		fmt.Println("treat7 with v")
	} else {
		fmt.Println("control7 with v")
	}

	if v := ge.flagMthds.treatedBehaviour(staleFlag); v != true {
		fmt.Println("treat8 with v")
	} else {
		fmt.Println("control8 with v")
	}
}
