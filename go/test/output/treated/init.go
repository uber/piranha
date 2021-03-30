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

// PropFlag :defining a type for flag
type PropFlag int

// This contatins flags
// This works like enum in golang
// If we got any other implementation of flags then we will see on that in future
const (
	//comment0
	globalFlag PropFlag = iota //comment1
	//comment4
	localFlag
	newFlag
)

// FlagMethods : This will contains treatment and control methods
type FlagMethods struct {
}

func (flgMthd FlagMethods) treatedBehaviour(flag PropFlag) bool {
	return true
}
func (flgMthd FlagMethods) controlBehaviour(flag PropFlag) bool {
	return true
}
func (flgMthd FlagMethods) commonBehaviour(str string, flag2 PropFlag) bool {
	return true
}

// GoExamples : This will act as a class
type GoExamples struct {
	flagMthds FlagMethods

	localService, globalService, newFeatures, basicFeature bool
}

func globalFeature(flag PropFlag) bool {
	if flag == globalFlag {
		return true
	}
	return false
}
