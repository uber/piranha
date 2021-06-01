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
		fmt.Println("treated || of treatedBehaviour")
	} else {
		fmt.Println("control || of treatedBehaviour")
	}

	fmt.Println("control && of treatedBehaviour")
	fmt.Println("control && of || of treatedBehaviour")
	fmt.Println("control && of && of treatedBehaviour")
	fmt.Println("control && equals of treatedBehaviour")
	fmt.Println("treated || equals of controlBehaviour")

	if y && x {
		fmt.Println("treated && and && of controlBehaviour")
	} else {
		fmt.Println("control && and && of controlBehaviour")
	}

	fmt.Println("treated || && || of controlBehaviour")
}
