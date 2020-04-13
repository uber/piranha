# Piranha

Feature flags are commonly used to enable gradual rollout or experiment with new features. In a few cases, even after the purpose of the flag is accomplished, the code pertaining to the feature flag is not removed. We refer to such flags as stale flags. The presence of code pertaining to stale flags can have the following drawbacks: 
- Unnecessary code clutter increases the overall complexity w.r.t maintenance resulting in reduced developer productivity 
- The flags can interfere with other experimental flags (e.g., due to nesting under a flag that is always false)
- Presence of unused code in the source as well as the binary 
- Stale flags can also cause bugs 

Piranha is a tool to automatically refactor code related to stale flags. At a higher level, the input to the tool is the name of the flag and the expected behavior, after specifying a list of APIs related to flags in a properties file. Piranha will use these inputs to automatically refactor the code according to the expected behavior. 

This repository contains three independent versions of Piranha, one for each of the three supported languages: Objective-C, Swift, and Java.

To use/build each version, look under the corresponding [lang]/ directory and follow instructions in the corresponding [lang]/README.md file. Make sure to cd into that directory to build any related code following the instructions in the README. 

- [PiranhaJava](java/README.md)
- [PiranhaObjC](objc/README.md)
- [PiranhaSwift](swift/README.md)

A few additional links on Piranha: 

- A technical [report](report.pdf) detailing our experiences with using Piranha at Uber.
- A [blogpost](https://eng.uber.com/piranha/) presenting more information on Piranha. 
- 6 minute [video](https://www.youtube.com/watch?v=V5XirDs6LX8&feature=emb_logo) overview of Piranha.

## Support

Please feel free to [open a GitHub issue](https://github.com/uber/piranha/issues) if you have any questions on how to use Piranha.  

## Contributors

We'd love for you to contribute to Piranha!  Please note that once
you create a pull request, you will be asked to sign our [Uber Contributor License Agreement](https://cla-assistant.io/uber/piranha).

## License
Piranha is licensed under the Apache 2.0 license.  See the LICENSE file for more information.
