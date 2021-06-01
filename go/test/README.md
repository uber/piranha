# Testing of PiranhaGo
1. Run `go test` in this directory to test the working of Piranha.
2. `input` directory contains test cases that are used for testing.
3. `output` directory contains two subdirectories `treated` and `control`. When `piranha` is run **with** flag `-treated` on a test file in `input`, it generates refactored code that will match with the corresponding file in `output/treated`. Similarly, if `piranha` is run **without** flag `-treated`, then it will generate output that matches the corresponding file in `output/control`.
