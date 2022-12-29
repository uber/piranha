# Piranha Swift
Piranha scans source files to delete code related to stale feature flags leading to a cleaner, safer, more performant, and more maintainable code base.

# Development

- Run `swift build && swift package generate-xcodeproj`
- Open Xcode project
- Make required changes and do local testing
- Run `sh package.sh`
- Run `sh test.sh` (should print a successful message without a diff)

## Usage

Piranha Swift can be configured to recognize different flag APIs and flag behaviours by specifying a `properties.json` file and the appropriate command line options. 

```
USAGE: <piranha_exe> cleanup-stale-flags [--source-file <source-file>] [--config-file <config-file>] --flag <flag> [--group-name <group-name>] [--treated]

OPTIONS:
  -s, --source-file <source-file>
                          Input source file for Piranha
  -c, --config-file <config-file>
                          Path of configuration file for Piranha 
  -f, --flag <flag>       Name of the stale flag 
  --group-name <group-name>
  -t, --treated           If this option is supplied, the flag is treated, otherwise it is control. 
  -h, --help              Show help information.
```

### Example 

Consider the following properties file `properties.json`.

```json
{
  "methodProperties": [
    {
      "methodName": "isToggleDisabled",
      "flagType": "control",
      "flagIndex": 0
    },
    {
      "methodName": "isFlagTreated",
      "flagType": "treated",
      "flagIndex": 0
    },
    {
      "methodName": "isToggleEnabled",
      "flagType": "treated",
      "flagIndex": 0
    }
  ]
}
```

This specifies a list of flag APIs each of which return a boolean value. It is assumed that a method returns `true` iff the `flagType` specified in the properties file matches the flag behaviour specified at runtime. For example, if the flag behavior is `treated`, `isFlagTreated(flag)` will return `true` and `isToggleDisabled(flag)` will return `false`.

The following code in `Examples/sample.swift` contains the stale flag `featureFlag`

```swift
// Simple flag cleanup in conditional
if cachedExperiments.isTreated(forExperiment: ExperimentNamesSwift.featureFlag) {
    f1();
 } else {
    f2();
 }

// Assignment cleanup
var a = isToggleDisabled(ExperimentNamesSwift.featureFlag)
if(a) {
   f1()
} else {
   f2()
}

```
When this file is run through Piranha with the command 

```
<piranha_exe> cleanup-stale-flags -c properties.json -s Examples/sample.swift -f featureFlag --treated
```

It yields the following output:

```swift
f1()
f2()
```

Note that code-style and comments are preserved wherever possible by default.



