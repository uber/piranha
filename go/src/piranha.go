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
Usage: ./piranha [-h] -p PROPERTIES -s SOURCE_FILE -f STALE_FLAG [-treated] [-o OUTPUT]
Required arguments:
		-s SOURCE_FILE: Path of the file to be refactored
		-p PROPERTIES: Configuration file (json format) for Piranha.
		-f STALE_FLAG: Name of the stale flag
Optional arguments:
		-h: Show the options and exit.
		-treated: If this is given, then flag is treated,
			otherwise it is control.
		-o OUTPUT: Destination of the refactored output from piranha. If -o is not provided, then the source file is updated in place.
*/
func help() {
	//Help message
	fmt.Println(
		"Usage: ./piranha [-h] -p PROPERTIES -s SOURCE_FILE -f STALE_FLAG [-treated] [-o OUTPUT]",
		"\nRequired arguments:",
		"\n\t\t\t-s SOURCE_FILE: Path of the file to be refactored.",
		"\n\t\t\t-p PROPERTIES: Configuration file (json format) for Piranha.",
		"\n\t\t\t-f STALE_FLAG: Name of the stale flag.",
		"\nOptional arguments:",
		"\n\t\t\t-h: Show the options and exit.",
		"\n\t\t\t-treated: If this is given, then flag is treated,",
		"\n\t\t\totherwise it is control.",
		"\n\t\t\t-o OUTPUT: Destination of the refactored output from piranha.",
		"\n\t\t\tIf -o is not provided, then the source file is updated in place.")
}

// RunPiranha : the main function for the piranha tool
func RunPiranha(inArgs []string) {
	var sourceFile, configFile, flagName, outputFileName string = "", "", "", ""
	var isTreated = false
	sizeOfArgs := len(inArgs)
	if len(inArgs) < 2 {
		help()
		return
	}
	for index, arg := range inArgs {
		switch arg {
		case "-h":
			help()
			return
		case "-p":
			if index+1 < sizeOfArgs {
				configFile = inArgs[index+1]
				if !strings.HasSuffix(configFile, ".json") {
					return
				}
			}
		case "-s":
			if index+1 < sizeOfArgs {
				sourceFile = inArgs[index+1]
				if !strings.HasSuffix(sourceFile, ".go") {
					return
				}
			}
		case "-f":
			if index+1 < sizeOfArgs {
				flagName = inArgs[index+1]
			}
		case "-treated":
			isTreated = true
		case "-o":
			if index+1 < sizeOfArgs {
				outputFileName = inArgs[index+1]
			}
		default:
			break
		}
	}

	if flagName == "" {
		fmt.Println("Please provide a flag.")
	}
	if sourceFile == "" {
		fmt.Println("Please provide a source file that is to be refactored.")
	}
	if configFile == "" {
		fmt.Println("Please provide a config file. See README for more instructions.")
	}
	if flagName == "" || sourceFile == "" || configFile == "" {
		fmt.Println("For more info, run ./piranha -h.")
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
