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

// A defer statement defers the execution of a function until the surrounding function returns.
func (ge GoExamples) useOfDefer() bool {
	defer ge.storeuseInit()
	return true
}

func (ge GoExamples) storeuseBefore() {
	/*
		TODO: There is work going on to add the deep clean feature.
		In this case the first if statment should be treated because
		ge.basicFeature value may get affected by stale feature flag.
	*/
	if ge.basicFeature {
		fmt.Println("Hi basicFeature")
	}
	if ge.localService {
		fmt.Println("hi localService")
	}
	if ge.newFeatures {
		fmt.Println("hi newFeatures")
	}

	if !ge.localService {
		fmt.Println("not localService")
	} else {
		fmt.Println("localService")
	}

	if ge.localService && ge.newFeatures {
		fmt.Println("localService and newFeatures")
	}

	if ge.newFeatures {
		fmt.Println("newFeatures")
	}
}

// This function is also acting as a pointer reciever
// Methods with pointer receivers can modify the value to which the receiver points
// This also using above two functions
func (ge *GoExamples) storeuseInit() {
	ge.basicFeature = false
	ge.localService = ge.flagMthds.controlBehaviour(localFlag)
	ge.newFeatures = ge.flagMthds.treatedBehaviour(newFlag)

	pointerfieldX := false
	pointerfieldY := &ge.localService

	if ge.localService {
		fmt.Println("I initialised localService")
	}

	//testing pointer. I think need more test cases like this
	if !*pointerfieldY {
		fmt.Println("pointers to be treated/not treated")
	}
}

func (ge GoExamples) storeuseAfter() {
	if ge.localService {
		fmt.Println("hi localService again")
	}
	if ge.newFeatures {
		fmt.Println("hi newFeatures again")
	}

	if !ge.localService {
		fmt.Println("not localService again")
	} else {
		fmt.Println("localService again")
	}

	if ge.localService && ge.newFeatures {
		fmt.Println("localService and newFeatures again")
	}

	if ge.newFeatures {
		fmt.Println("newFeatures again")
	}
}
