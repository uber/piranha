---
id: refactor
title: Refactoring your code base
sidebar_label: Refactoring
---
Our playground provides a user-friendly way to apply structural transformations to your code base. 
We use the same syntax used in the Piranha CLI configuration files, with one small difference: **all 
configurations need to be in the same file**. To use it, simply copy and paste your configurations into the playground.

Besides the standard CLI options, like rules and edges, you also need to specify substitutions in the same file. 
Here's an example to illustrate how it's done:

```toml
[[rules]]
name = "delete_variable_declaration"
query = """(
 (local_variable_declaration 
  declarator: (variable_declarator 
                   name: (_) @vdcl.lhs) @field_declaration)
  (#eq? @vdcl.lhs "@identifier")
"""
replace_node = "field_declaration"
replace = ""
holes = ["variable_name"]

[[rules]]
name = "replace_identifier_with_value"
query = """(
 (identifier) @identifier
 (#eq? @identifier "@variable_name")
)
"""
replace_node = "identifier"
replace = "@value"
holes = ["variable_name", "value"]

[[edges]]
scope = "Method"
from = "replace_identifier_with_value"
to = ["delete_variable_declaration"]

[[substitutions]]
variable_name = "xyz"
value = "123"
```

We plan to incorporate all Piranha arguments into our playground in the future. However, as of now, we only support substitutions.

:::info
Please be aware that refactoring your code base may take a few seconds, 
especially when dealing with larger directories containing millions of lines of code.
:::
