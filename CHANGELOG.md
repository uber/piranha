Version 0.3.16
-------------
* Added concrete syntax as matching language

Version 0.3.15
-------------
* Bug-fix in query validation

Version 0.3.14
-------------
* Bug-fix in graph validation

Version 0.3.13
-------------
* Introduce graph validation 
* Improved swift syntax support

Version 0.3.12
-------------
* Capture interface as "Class" scope in Java

Version 0.3.11
-------------
* Refactor to support other matching languages
* Introduce regex syntax for rules

Version 0.3.10
-------------
* Add support for replace node index
* Bug fix for iOS cleanup 
  
Version 0.3.9
-------------
* Rule graph validation #493
* Bug fix #497 #499
* Kotlin dependency update

Version 0.3.8
-------------
* Bug Fix related to code snippet mode #489
* Added support for iOS string resource file format #490

Version 0.3.7
-------------
* Support `enabled, err := foobar(), nil` scenario

Version 0.3.6
-------------
* Support richer constraints with `not_enclosing_node`
* Added checks to make sure filter arguments are consistent

Version 0.3.5
-------------
* Support filters without `enclosing_node` (#482)

Version 0.3.4
-------------
* Support richer constraints 
* ability to `include` or `exclude` particular paths
* Support variable / field inlining in Swift 
* More optimized if-statement cleanups
* Swift cleanup bug fixes
  
Version 0.3.3
-------------
* Added equality simplification for Java 
* Add support for thrift
* Ternary operator simplification for Swift

Version 0.3.2
-------------
* Improved the Python interface for constructing PiranhaArguments

Version 0.3.1
-------------
* Improve handling of leading (and trailing) commas and comments
* Fix boolean simplification rules 
* Add Enum scope for Java 
* Add option to transform/analyze partially parsable code (`--alow-dirty-tree`)
* Bug fixes

Version 0.3.0
-------------
* Introduce Feature flag cleanup for Go and Swift 
* Introduce a Python / Rust API for defining rules, edges and the arguments 
* Introduce code snippet mode 
* Improvements for the command line interface 
* Bug fixes

Version 0.2.0
-------------
* Fixed bug related to __build-in cleanup rules__ not being packaged [#247]
* Improve demos and documentation [#242] [#243]
* Add Python Structural/Replace support [#248]
* Improve logging and expose logs via pyo3 [#246]
* Fix *delete trailing comma* bug [#251]
* Add `File` Scope for Kotlin [#249]
* Added GitHub workflow to make release 

Version 0.2.1
-------------
* Fix disjunction rule [#269]
* Handle cleanups for feature flags within constructors [#268]
* Add `dry_run` as a command line argument [#263]
* Support for structural match/replace (with chaining) for Go [#256]
* Support for structural match/replace (with chaining) for TS / TSX [#260]
* Improve documentation [#261, #259, #258, #257]



