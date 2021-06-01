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

// TestFiles :This will test the output from the correct output
func TestFiles(t *testing.T) {
	tables := []struct {
		input  string
		output string
	}{
		{"./input/init.go", "./output/control/init.go"},
		{"./input/testExpressions.go", "./output/control/testExpressions.go"},
		{"./input/testConditional.go", "./output/control/testConditional.go"},
		{"./input/testSwitch.go", "./output/control/testSwitch.go"},
		{"./input/deepClean.go", "./output/control/deepClean.go"},

		{"./input/init.go", "./output/treated/init.go"},
		{"./input/testExpressions.go", "./output/treated/testExpressions.go"},
		{"./input/testConditional.go", "./output/treated/testConditional.go"},
		{"./input/testSwitch.go", "./output/treated/testSwitch.go"},
		{"./input/deepClean.go", "./output/treated/deepClean.go"},
	}
	// running each file as if they are running with this command
	// For one Pass
	// ../piranha -p ../properties.json -s "{filename}" -f staleFlag -o ./treatedFiles/$(basename {filename}) -treated
	var argsOnePass []string
	argsOnePass = append(argsOnePass, "-p")
	argsOnePass = append(argsOnePass, "../properties.json")
	argsOnePass = append(argsOnePass, "-s")
	argsOnePass = append(argsOnePass, "{filename}")
	argsOnePass = append(argsOnePass, "-f")
	argsOnePass = append(argsOnePass, "staleFlag")
	argsOnePass = append(argsOnePass, "-o")
	argsOnePass = append(argsOnePass, "{filename}")

	fmt.Print("Output format: \n")
	fmt.Print("Line <Line number>: <output from correct file>\n")
	fmt.Print("Line <Line number>: <output from treated file>\n")
	fmt.Print("\n")
	fmt.Print("Starting Tests \n")
	for ind, table := range tables {
		if ind == 5 {
			argsOnePass = append(argsOnePass, "-treated")
		}
		argsOnePass[3] = table.input
		argsOnePass[7] = "temp.go"
		src.RunPiranha(argsOnePass)

		fmt.Print("Testing ")
		if ind < 5 {
			fmt.Print("without -treated\n")
		} else {
			fmt.Print("with -treated\n")
		}
		fmt.Print("Matching with: ")
		fmt.Println(table.output)
		FileNotMatched := false
		corrFile, err := os.Open(table.output)
		if err != nil {
			fmt.Println("File reading error of correct file", err)
			return
		}
		treatFile, err := os.Open("temp.go")
		if err != nil {
			fmt.Println("File reading error of treat file", err)
			return
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
