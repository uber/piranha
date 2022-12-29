# Piranha

[![Join the chat at https://gitter.im/uber/piranha](https://badges.gitter.im/uber/piranha.svg)](https://gitter.im/uber/piranha?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Feature flags are commonly used to enable gradual rollout or experiment with new features. In a few cases, even after the purpose of the flag is accomplished, the code pertaining to the feature flag is not removed. We refer to such flags as stale flags. The presence of code pertaining to stale flags can have the following drawbacks: 
- Unnecessary code clutter increases the overall complexity w.r.t maintenance resulting in reduced developer productivity 
- The flags can interfere with other experimental flags (e.g., due to nesting under a flag that is always false)
- Presence of unused code in the source as well as the binary 
- Stale flags can also cause bugs 

Piranha is a tool to automatically refactor code related to stale flags. At a higher level, the input to the tool is the name of the flag and the expected behavior, after specifying a list of APIs related to flags in a properties file. Piranha will use these inputs to automatically refactor the code according to the expected behavior. 

This repository contains four independent versions of Piranha, one for each of the four supported languages: Java, JavaScript, Objective-C and Swift. **It also contains a redesigned variant of Piranha (as of May 2022) that is a common refactoring tool to support multiple languages and feature flag APIs. If interested in this polyglot variant, goto [Polyglot Piranha](POLYGLOT_README.md)**.

To use/build each version, look under the corresponding [lang]/ directory and follow instructions in the corresponding [lang]/README.md file. Make sure to cd into that directory to build any related code following the instructions in the README. 

- [PiranhaJava](legacy/java/README.md)
- [PiranhaJS](legacy/javascript/README.md)
- [PiranhaObjC](legacy/objc/README.md)
- [PiranhaSwift](legacy/swift/README.md)

A few additional links on Piranha: 

- A technical [report](report.pdf) detailing our experiences with using Piranha at Uber.
- A [blogpost](https://eng.uber.com/piranha/) presenting more information on Piranha. 
- 6 minute [video](https://www.youtube.com/watch?v=V5XirDs6LX8&feature=emb_logo) overview of Piranha.

## Support

If you have any questions on how to use Piranha, please feel free to reach out to us on the [gitter channel](https://gitter.im/uber/piranha?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge). For bugs and enhancement requests, [open a GitHub issue](https://github.com/uber/piranha/issues).

## Contributors

We'd love for you to contribute to Piranha!  Please note that once
you create a pull request, you will be asked to sign our [Uber Contributor License Agreement](https://cla-assistant.io/uber/piranha).

We are also looking for contributions to extend Piranha to other languages (C++, C#, Kotlin). 

## License
Piranha is licensed under the Apache 2.0 license.  See the LICENSE file for more information.
