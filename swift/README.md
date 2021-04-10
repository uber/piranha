# Piranha Swift
Piranha scans source files to delete code related to stale feature flags leading to a cleaner, safer, more performant, and more maintainable code base.

# Development

- Run `swift build && swift package generate-xcodeproj`
- Open Xcode project
- Make required changes and do local testing
- Run `sh package.sh`
- Run `sh test.sh` (should print a successful message without a diff)

## Usage

Piranha Swift can be configured to recognize different flag APIs and flag behaviours by specifying a `.properties` file and the appropriate command line options. 

```
USAGE: <piranha_exe> cleanup-stale-flags [--source-file <source-file>] [--config-file <config-file>] --flag <flag> [--group-name <group-name>] [--treated]

OPTIONS:
  -s, --source-file <source-file>
                          Input Source File that needs to run piranha 
  -c, --config-file <config-file>
                          Path of configuration file for Piranha 
  -f, --flag <flag>       Name of the stale flag 
  --group-name <group-name>
  -t, --treated           If this option is supplied, the flag is treated, otherwise it is control. 
  -h, --help              Show help information.
```



