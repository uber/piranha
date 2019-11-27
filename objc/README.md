# Development
  
- To update the functionality, modify the necessary sources in the `src` directory
- Run `generate-piranha-artifact.sh` (or `generate-dylib.sh`, if the `llvm` sources are already downloaded)
- Run tests by executing `test.sh` and ensuring there are no changes in the refactorings

# Usage
- Run `piranha-objc.sh` with the appropriate parameters (file, flagname, type).
-- Update `XcodeSDK` path appropriately
- See the examples in `tests` directory for the original code and refactored code.

# TODO
- Refactoring the implementation to accept other APIs
