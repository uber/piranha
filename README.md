# PolyglotPiranha

PolyglotPiranha is a lightweight code transformation toolset for automating large scale changes. At Uber, it is mostly used to clean up stale feature flags.

We only support languages that are used at Uber. We likely won't be able to add new languages in this repo. There are a number of forks (see https://github.com/uber/piranha/forks for a full list) that may provide additional features.


## Installation

To install Polyglot Piranha, you can use it as a Python library or as a command-line tool.

### Python API

To install the Python API, run the following command:

```bash
pip install polyglot-piranha
```

### Command-line Interface

To install the command-line interface, follow these steps:

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone the repository: `git clone https://github.com/uber/piranha.git`
3. Navigate to the cloned directory: `cd piranha`
4. Build the project: `cargo build --release` (or `cargo build --release --no-default-features` for macOS)
5. The binary will be generated under `target/release`

## Example Usage

```python
from polyglot_piranha import execute_piranha, PiranhaArguments, Rule, RuleGraph, OutgoingEdges

# Original code snippet
code = """
if (obj.isLocEnabled() || x > 0) {
    // do something
} else {
    // do something else!
}
"""

# Define the rule to replace the method call
r1 = Rule(
    name="replace_method",
    query="cs :[x].isLocEnabled()", # cs indicates we are using concrete syntax
    replace_node="*",
    replace="true",
    is_seed_rule=True
)

# Define the edges for the rule graph. 
# In this case, boolean_literal_cleanup is already defined for java [see src/cleanup_rules]
edge = OutgoingEdges("replace_method", to=["boolean_literal_cleanup"], scope="parent")

# Create Piranha arguments
piranha_arguments = PiranhaArguments(
    code_snippet=code,
    language="java",
    rule_graph=RuleGraph(rules=[r1], edges=[edge])
)

# Execute Piranha and print the transformed code
piranha_summary = execute_piranha(piranha_arguments)
print(piranha_summary[0].content)
```


## Documentation

For more examples and explanations of the toolset, please check our demos and extended [POLYGLOT_README.md](POLYGLOT_README.md) file.


## Feature Flags


Feature flags are commonly used to enable gradual rollout or experiment with new features. In a few cases, even after the purpose of the flag is accomplished, the code pertaining to the feature flag is not removed. We refer to such flags as stale flags. The presence of code pertaining to stale flags can have the following drawbacks: 
- Unnecessary code clutter increases the overall complexity w.r.t maintenance resulting in reduced developer productivity 
- The flags can interfere with other experimental flags (e.g., due to nesting under a flag that is always false)
- Presence of unused code in the source as well as the binary 
- Stale flags can also cause bugs 

PolyglotPiranha is a tool that can automatically refactor code related to stale flags. At a higher level, the input to the tool is the name of the flag and the expected behavior, after specifying a list of APIs related to flags in a properties file. Piranha will use these inputs to automatically refactor the code according to the expected behavior.

PolyglotPiranha (as of May 2022) is a common refactoring tool to support multiple languages and feature flag APIs.
For legacy language-specific implementations please check following [tag](https://github.com/uber/piranha/releases/tag/last-version-having-legacy-piranha).



A few additional links on Piranha: 

- Research paper published at [PLDI 2024](https://dl.acm.org/doi/10.1145/3656429) on PolyglotPiranha.
- A technical [report](report.pdf) detailing our experiences with using Piranha at Uber.
- A [blogpost](https://eng.uber.com/piranha/) presenting more information on Piranha. 
- 6 minute [video](https://www.youtube.com/watch?v=V5XirDs6LX8&feature=emb_logo) overview of Piranha.

## Support

If you have any questions on how to use Piranha or find any bugs, please [open a GitHub issue](https://github.com/uber/piranha/issues).

## License
Piranha is licensed under the Apache 2.0 license.  See the LICENSE file for more information.

## Note

This is not an official Uber product, and provided as is.


