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

# Acknowledgements
- Some aspects of the refactoring are based on this blog article:
http://www.goldsborough.me/c++/clang/llvm/tools/2017/02/24/00-00-06-emitting_diagnostics_and_fixithints_in_clang_tools/

- A majority of the test cases (as of Nov 11, 2019) are due to Nick Lauer.

