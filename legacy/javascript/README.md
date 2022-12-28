# PiranhaJS
![Node.js CI](https://github.com/uber/piranha/workflows/Node.js%20CI/badge.svg) ![Lint](https://github.com/uber/piranha/workflows/Lint/badge.svg)

Piranha scans source files to delete code related to stale feature flags leading to a cleaner, safer, more performant, and more maintainable code base. This is the ECMAScript variant of Piranha implemented as a Node.js app. 

## Requirements

Check your node version using ``` node -v```. It should be 10.12.0 or above. Instructions about updating NodeJS can be found at this [stack overflow link](
https://stackoverflow.com/questions/10075990/upgrading-node-js-to-latest-version).

## Installation

Clone, and then install node dependencies.

```
git clone https://github.com/uber/piranha.git 
cd javascript
npm i
```

To test your installation, run `npm test` and ensure all tests pass.

## Usage

PiranhaJS can be configured to recognize different flag APIs and flag behaviours by specifying a `properties.json` file and the appropriate command line options. Run `node src/piranha.js -h` to see a list of command line options Piranha accepts. 

```
usage: piranha.js [-h] -s SOURCE -f FLAG -p PROPERTIES [-o OUTPUT] [-t]
                  [-n MAX_CLEANUP_STEPS] [-c]
                  

Optional arguments:
  -h, --help            Show this help message and exit.
  -o OUTPUT, --output OUTPUT
                        Destination of the refactored output. File is 
                        modified in-place by default.
  -t , --treated        If this option is supplied, the flag is treated, 
                        otherwise it is control.
  -n MAX_CLEANUP_STEPS, --max_cleanup_steps MAX_CLEANUP_STEPS
                        The number of times literals should be simplified. 
                        Runs until fixed point by default.
  -c , --keep_comments 
                        To keep all comments

Required arguments:
  -s SOURCE, --source SOURCE
                        Path of input file for refactoring
  -f FLAG, --flag FLAG  Name of the stale flag
  -p PROPERTIES, --properties PROPERTIES
                        Path of configuration file for Piranha
```

### Example 

Consider the following properties file `config/properties.json`.

```json
{
  "methodProperties": [
    {
      "methodName": "isToggleDisabled",
      "flagType": "control",
      "argumentIndex": 0
    },
    {
      "methodName": "isFlagTreated",
      "flagType": "treated",
      "argumentIndex": 0
    },
    {
      "methodName": "isToggleEnabled",
      "flagType": "treated",
      "argumentIndex": 0
    }
  ]
}
```

This specifies a list of flag APIs each of which return a boolean value. It is assumed that a method returns `true` iff the `flagType` specified in the properties file matches the flag behaviour specified at runtime. For example, if the flag behavior is `treated`, `isFlagTreated(flag)` will return `true` and `isToggleDisabled(flag)` will return `false`.

The code in `examples/sample.js` contains the stale flag `featureFlag`

```javascript
// Simple flag cleanup in conditional
if(isFlagTreated(featureFlag)) {
    f1();
 } else {
    f2();
 }
 
 // Assignment cleanup
 var a = isToggleDisabled(featureFlag);
 if(a) {
    f1(); 
 } else {
    f2();
 }
 
 // function cleanup
 function b() {
     return isFlagTreated(featureFlag);
 }
 
 if(b() || f1()) {
    f1()   
 } else {
    f2()   
 }
 
 // Complex cleanup
 var c = isToggleDisabled(featureFlag) ? f1() : isToggleDisabled(featureFlag) ? f2() : isFlagTreated(featureFlag) ? f3() : f4();
```
When this file is run through Piranha with the command 

```
node src/piranha.js -s examples/sample.js -p config/properties.json --treated -f featureFlag
```

`sample.js` is modified in-place to yield

```javascript
// Simple flag cleanup in conditional
f1();

// Assignment cleanup
f2();
f1()

// Complex cleanup
var c = f3();
```

Note that code-style and comments are preserved wherever possible by default.

### Running tests

To run unit and integration tests invoke

```
npm test
```

See the suite of tests in `test/test_refactor.js` and `test/test_integration.js` for more detail on how Piranha works and what kind of refactoring it can handle. 

### Running piranha as a command

It can be convenient to run piranha as a shell command from any location. This can be done by creating an alias.

From the `piranha/javascript` directory, run the following shell commands (for the bash shell).

```
printf "alias piranhajs='node `realpath src/piranha.js`'" >> ~/.bashrc
source ~/.bashrc
```

Other shells have similar commands. Macs don't come preinstalled with the `realpath` utility. One way is to create an alias for `realpath` and run above commands.
```
alias realpath="python -c \"import os,sys; print os.path.realpath(sys.argv[1])\""
```
Another way is to install `coreutils` which contains `realpath` using `brew`. Make sure you have `Homebrew` installed before running below command.
```
brew install coreutils
``` 
