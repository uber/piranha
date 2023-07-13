---
id: api
title: Python API
---

Currently, we support one simple API (`execute_piranha`), a simple python wrapper around Polyglot Piranha's CLI.
We believe this makes it easy to incorporate Piranha in *"pipelining"*.

<h4> <code>execute_piranha</code></h4>

```python
from polyglot_piranha import execute_piranha, PiranhaArguments

piranha_arguments = PiranhaArguments(
    path_to_codebase = "...",
    path_to_configurations = "...",
    language= "java",
    substitutions = {},
    dry_run = False, 
    cleanup_comments = True
)
piranha_summary = execute_piranha(piranha_arguments)
```
The API `execute_piranha` accepts a `PiranhaArguments`
An object of PiranhaArguments can be instantiated with the following arguments:

- (*required*) `path_to_codebase` (`str`): Path to source code folder
- (*required*) `path_to_configuration` (`str`) : A directory containing files named `rules.toml` and `edges.toml`
    * `rules.toml`: *piranha rules* expresses the specific AST patterns to match and __replacement patterns__ for these matches (in-place). These rules can also specify the pre-built language specific cleanups to trigger.
    * `edges.toml` : expresses the flow between the rules
- (*required*) `language` (`str`) : Target language (`java`, `py`, `kt`, `swift`, `py`, `ts` and `tsx`)
- (*required*) `substitutions` (`dict`): Substitutions to instantiate the initial set of feature flag rules
- (*optional*) `dry_run` (`bool`) : Disables in-place rewriting of code
- (*optional*) `cleanup_comments` (`bool`) : Enables deletion of associated comments
- (*optional*) `cleanup_comments_buffer` (`usize`): The number of lines to consider for cleaning up the comments
- (*optional*) `number_of_ancestors_in_parent_scope` (`usize`): The number of ancestors considered when `PARENT` rules
- (*optional*) `delete_file_if_empty` (`bool`): User option that determines whether an empty file will be deleted
- (*optional*) `delete_consecutive_new_lines` (`bool`) : Replaces consecutive `\n`s  with a single `\n`

<h5> Returns </h5>

`[Piranha_Output]` : a [`PiranhaOutputSummary`](/src/models/piranha_output.rs) for each file touched or analyzed by Piranha. It contains useful information like, matches found (for *match-only* rules), rewrites performed, and content of the file after the rewrite. The content is particularly useful when `dry_run` is passed as `true`.