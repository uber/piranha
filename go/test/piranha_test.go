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
	"bufio"
	"fmt"
	"log"
	"os"
	"testing"

	"github.com/PiranhaGo/src"
)

func compFiles(file1 string, file2 string) bool {
	FileNotMatched := false
	corrFile, err := os.Open(file2)
	if err != nil {
		fmt.Println("File reading error of correct file", err)
		return false
	}
	treatFile, err := os.Open(file1)
	if err != nil {
		fmt.Println("File reading error of treat file", err)
		return false
	}

	corrScan := bufio.NewScanner(corrFile)
	treatScan := bufio.NewScanner(treatFile)

	lineNumcorrFile := 0
	lineNumtreatFile := 0

	endOfcorrFile := false
	endOftreatFile := false
	for corrScan.Scan() {
		// Scanning
		treatScan.Scan()
		lineNumcorrFile++
		lineNumtreatFile++
		for corrScan.Text() == "" {
			if !corrScan.Scan() {
				endOfcorrFile = true
				break
			}
			lineNumcorrFile++
		}
		for treatScan.Text() == "" {
			if !treatScan.Scan() {
				endOftreatFile = true
				break
			}
			lineNumtreatFile++
		}
		if endOfcorrFile || endOftreatFile {
			break
		}

		// Matching
		if corrScan.Text() != treatScan.Text() {
			fmt.Print("\n")
			fmt.Print("Line ", lineNumcorrFile, ": ", corrScan.Text())
			fmt.Print("\n")
			fmt.Print("Line ", lineNumtreatFile, ": ", treatScan.Text())
			fmt.Print("\n")
			FileNotMatched = true
			break
		}
		if FileNotMatched {
			break
		}
	}
	corrFile.Close()
	treatFile.Close()
	return FileNotMatched
}

// TestFiles :This will test the output from the correct output
func TestFiles(t *testing.T) {
	tables := []struct {
		input         string
		outputControl string
		outputTreated string
	}{
		{"./input/init.go", "./output/control/init.go", "./output/treated/init.go"},
		{"./input/testExpressions.go", "./output/control/testExpressions.go", "./output/treated/testExpressions.go"},
		{"./input/testConditional.go", "./output/control/testConditional.go", "./output/treated/testConditional.go"},
		{"./input/testSwitch.go", "./output/control/testSwitch.go", "./output/treated/testSwitch.go"},
		{"./input/deepClean.go", "./output/control/deepClean.go", "./output/treated/deepClean.go"},
	}
	// running each file as if they are running with this command
	// For one Pass
	// ../piranha -p ../properties.json -s "{filename}" -f staleFlag -o ./treatedFiles/$(basename {filename}) -treated
	var configFile = "../properties.json"
	var flagName = "staleFlag"
	var isTreated bool

	fmt.Print("Output format: \n")
	fmt.Print("Line <Line number>: <output from correct file>\n")
	fmt.Print("Line <Line number>: <output from treated file>\n")
	fmt.Print("\n")
	fmt.Print("Starting Tests \n")

	fmt.Print("Testing with -mode treated\n\n")
	isTreated = true
	var FileNotMatched bool
	for _, table := range tables {
		fmt.Print("Matching with: ")
		fmt.Println(table.outputTreated)
		src.RunPiranha(table.input, configFile, flagName, "temp.go", isTreated)

		FileNotMatched = compFiles("temp.go", table.outputTreated)
		if FileNotMatched {
			t.Errorf("Files didn't match, see above output")
		} else {
			fmt.Println("File Matched Sucessfully")
		}
		fmt.Println()
		del := os.Remove("temp.go")
		if del != nil {
			log.Fatal(del)
		}
	}
	fmt.Print("Testing with -mode control\n")
	isTreated=false
	for _, table := range tables {
		fmt.Print("Matching with: ")
		fmt.Println(table.outputControl)
		src.RunPiranha(table.input, configFile, flagName, "temp.go", isTreated)

		FileNotMatched = compFiles("temp.go", table.outputControl)
		if FileNotMatched {
			t.Errorf("Files didn't match, see above output")
		} else {
			fmt.Println("File Matched Sucessfully")
		}
		fmt.Println()
		del := os.Remove("temp.go")
		if del != nil {
			log.Fatal(del)
		}
	}
}
