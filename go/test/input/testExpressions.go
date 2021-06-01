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
	if ge.flagMthds.treatedBehaviour(staleFlag) {
		fmt.Println("treated behaviour of treatedBehaviour")
	} else {
		fmt.Println("control behaviour of treatedBehaviour")
	}

	//global feature is not in properties right now. So this should not get treated
	if globalFeature(staleFlag) {
		fmt.Println("global treated behaviour")
	} else {
		fmt.Println("global control behaviour")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) {
		fmt.Println("treated behaviour of controlBehaviour")
	} else {
		fmt.Println("control behaviour of controlBehaviour")
	}
	var x, y bool = false, false

	if ge.flagMthds.treatedBehaviour(staleFlag) || x {
		fmt.Println("treated || of treatedBehaviour")
	} else {
		fmt.Println("control || of treatedBehaviour")
	}

	if ge.flagMthds.treatedBehaviour(staleFlag) && x {
		fmt.Println("treated && of treatedBehaviour")
	} else {
		fmt.Println("control && of treatedBehaviour")
	}

	if ge.flagMthds.treatedBehaviour(staleFlag) && (x || y) {
		fmt.Println("treated && of || of treatedBehaviour")
	} else {
		fmt.Println("control && of || of treatedBehaviour")
	}

	if ge.flagMthds.treatedBehaviour(staleFlag) && (x && y) {
		fmt.Println("treated && of && of treatedBehaviour")
	} else {
		fmt.Println("control && of && of treatedBehaviour")
	}

	if ge.flagMthds.treatedBehaviour(staleFlag) && y == x {
		fmt.Println("treated && equals of treatedBehaviour")
	} else {
		fmt.Println("control && equals of treatedBehaviour")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) || y == x {
		fmt.Println("treated || equals of controlBehaviour")
	} else {
		fmt.Println("control || equals of controlBehaviour")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) && y && x {
		fmt.Println("treated && and && of controlBehaviour")
	} else {
		fmt.Println("control && and && of controlBehaviour")
	}

	if ge.flagMthds.controlBehaviour(staleFlag) || y || x {
		fmt.Println("treated || && || of controlBehaviour")
	} else {
		fmt.Println("control || && || of controlBehaviour")
	}

	y = ge.flagMthds.treatedBehaviour(staleFlag)
	y = ge.flagMthds.controlBehaviour(staleFlag) && x
	y = ge.flagMthds.treatedBehaviour(staleFlag) || x

	if y {
		fmt.Println("y cleaned, so treated behaviour")
	} else {
		fmt.Println("y cleaned, so you see control behaviour")
	}

	y = ge.flagMthds.treatedBehaviour(staleFlag) && y == x
	// This is done on purpose to check deep clean work
	y = ge.flagMthds.treatedBehaviour(staleFlag)

	if y {
		fmt.Println("y not cleaned, so treated behaviour")
	} else {
		fmt.Println("y not cleaned, so control behaviour")
	}

}
