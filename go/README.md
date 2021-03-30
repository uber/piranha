# PiranhaGo
PiranhaGo is runnable now. 
Instructions:-
1. To build the package run `go build -o piranha`. Dependencies will install automatically and they are given in `go.mod` file.
2. Below are the instructions for running for the single file.
```
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
```
3. To do a test run, run piranha on `example/testExample.go`. Run `./piranha -p properties.json -s ./example/testExample.go -o ./example/treatedExample.go -f staleFlag` command in root directory. You will get your refracted file as `/example/treatedExample.go`.