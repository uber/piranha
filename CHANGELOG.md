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


Version 0.3.0
-------------
* Introduce Feature flag cleanup for Go and Swift 
* Introduce a Python / Rust API for defining rules, edges and the arguments 
* Introduce code snippet mode 
* Improvements for the command line interface 
* Bug fixes


Version 0.3.1
-------------
* Improve handling of leading (and trailing) commas and comments
* Fix boolean simplification rules 
* Add Enum scope for Java 
* Add option to transform/analyze partially parsable code (`--alow-dirty-tree`)
* Bug fixes

Version 0.3.2
-------------
* Improved the Python interface for constructing PiranhaArguments
