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
package src

import (
	"fmt"
	"go/parser"
	"go/token"
	"log"
	"os"
	"strings"

	"github.com/dave/dst"
	"github.com/dave/dst/decorator"
)

/*
Inputs from args
Usage: ./piranha [-h] -p PROPERTIES -s SOURCE_FILE -f STALE_FLAG -mode MODE_NAME [-o OUTPUT]
Required arguments:
		-s SOURCE_FILE: Path of the file to be refactored.
		-p PROPERTIES: Configuration file (json format) for Piranha.
		-f STALE_FLAG: Name of the stale flag.
		-mode MODE_NAME: If MODE_NAME=treated, then flag is treated,
			otherwise MODE_NAME=control, it is control.
Optional arguments:
		-h: Show the options and exit.
		-o OUTPUT: Destination of the refactored output from piranha. If -o is not provided, then the source file is updated in place.
*/
func reportArgumentError(arg string) {
	if arg == "configFile" {
		fmt.Println("Please provide configuration file of json format.")
	} else if arg == "sourceFile" {
		fmt.Println("Please provide source file of go format.")
	}
	fmt.Println("For more info, run ./piranha -h.")
}

// RunPiranha : the main function for the piranha tool
func RunPiranha(sourceFile string, configFile string, flagName string, outputFileName string, isTreated bool) {
	if flagName == "STALE_FLAG" {
		fmt.Println("Please provide a flag.")
	}
	if sourceFile == "SOURCE_FILE" {
		fmt.Println("Please provide a source file that is to be refactored.")
	}
	if configFile == "PROPERTIES" {
		fmt.Println("Please provide a config file. See README for more instructions.")
	}
	if flagName == "STALE_FLAG" || sourceFile == "SOURCE_FILE" || configFile == "PROPERTIES" {
		fmt.Println("For more info, run ./piranha -h.")
		return
	}

	if !strings.HasSuffix(configFile, ".json") {
		reportArgumentError("configFile")
		return
	}

	if !strings.HasSuffix(sourceFile, ".go") {
		reportArgumentError("sourceFile")
		return
	}

	fs := token.NewFileSet()
	parsed, err := decorator.ParseFile(fs, sourceFile, nil, parser.ParseComments)
	if err != nil {
		log.Fatal(err)
	}

	var cleaner staleFlagCleaner
	cleaner.init(configFile, flagName, isTreated)
	newRoot := cleaner.run(parsed)
	////////////////////////
	// For debugging purpose. It prints out the ast.
	// spew.Dump(newRoot)
	///////////////////////

	if outputFileName == "" {
		outputFileName = sourceFile
	}
	outputFile, err := os.Create(outputFileName)
	/*
		Here we are typecasting newRoot to dst.File.It is safe because the root of AST
		always starts with the dst.File type.
	*/
	decorator.Fprint(outputFile, newRoot.(*dst.File))
}
