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
package main

import (
	"flag"

	"github.com/PiranhaGo/src"
)

func main() {
	var sourceFile, configFile, flagName, outputFileName, modeName string
	var isTreated = false

	// get configFile
	flag.StringVar(&configFile, "p", "PROPERTIES", "Configuration file (json format) for Piranha.")
	// get sourceFile
	flag.StringVar(&sourceFile, "s", "SOURCE_FILE", "Path of the file to be refactored.")
	// get flagName
	flag.StringVar(&flagName, "f", "STALE_FLAG", "Name of the stale flag.")
	// check treatedMode
	flag.StringVar(&modeName, "mode", "MODE_NAME", "If MODE_NAME=treated, then flag is treated, otherwise MODE_NAME=control, it is control.")
	if modeName == "treated" {
		isTreated = true
	}
	// get outputFileName
	flag.StringVar(&outputFileName, "o", "OUTPUT", "Destination of the refactored output from piranha. If -o is not provided, then the source file is updated in place.")

	flag.Parse()
	
	
	src.RunPiranha(sourceFile, configFile, flagName, outputFileName, isTreated)
}
