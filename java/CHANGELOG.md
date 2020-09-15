Version 0.1.2
-------------
* Fix issue with code not being deleted across files [#99]
* Add DisabledUnlessConfigured option [#97]
* (repo) Update RELEASING.md instructions [#88]

Version 0.1.1
-------------
* Upgrade Error Prone dependency to 2.4.0 [#73]
* Improvements to default formatting [#64] [#75]
Add note recommending automated code formatting [#69]
* Fail hard in the presence of configuration errors [#79]
* (tooling) Enable NullAway for the PiranhaJava core [#62]

Version 0.1.0
-------------
* [IMPORTANT] Switch config to structured properties.json [#39] 
* Refactor PiranhaJava configuration internals. [#55]
* New/extra types allowed for flags [#28]
  - Ability to use string-literal flags
  - Ability to use string-constant flags
* Build/repo cleanup:
  - Add maven instructions [#27], additional links [#29]
  - Set up travis CI for PiranhaJava and PiranhaSwift [#33]
  - Enforce Google Java Format for PiranhaJava [#34]

Version 0.0.3
-------------
* Improve simplification of nested conditionals [#18]
* Fix overlap checking for nodes with same end position [#16]
* Fix broken link in README [#14]
* Add auto-cleanup for java/sample integration test [#21]

Version 0.0.2
-------------
* Add TREATED to common group names (do not try to remove) [#12]
* Piranha Java is now cut from the same repository as 
  Piranha for Swift and Objective-C.
