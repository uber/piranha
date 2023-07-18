---
id: rules
label: Rules
title: Defining
---


Rules in Polyglot Piranha are defined using TOML (Tom's Obvious, Minimal Language). Each rule represents a transformation that identifies and modifies specific code snippets.
A rule should contain at least one rule with the following properties:
- `query`: A query to find the code pattern to refactor (currently only tree-sitter queries and regex are supported).
- `replace_node`: The captured node in the query that will be replaced.
- `replace_string`: Replacement string or pattern for the refactored code.
- `holes`: Placeholders in your queries that will be instantiated at runtime.
- `is_seed_rule`: Specifies whether this rule is an entry point for the rule graph.

Optionally, a rule can have filters. Piranha supports two kinds of filters:
- `enclosing_node`: A pattern that specifies the enclosing node of the rule.
- `not_enclosing_node`: A pattern that should not match any parent of the main match.

The `enclosing_node` and `not_enclosing_node` filters can be refined using contains with specified `[at_least, at_most]` bounds, as well as `not_contains`.


Here is an example of a rule:
```toml
[[rules]]
name = "your_rule_name"
query = """(
    (method_invocation name: (_) @name
                       arguments: (argument_list) @args) @invk
    (#eq? @name @hole1))
"""
replace_node = "invk"
replace = "X.other_string @args"
holes = ["hole1"]
is_seed_rule = true
[[rules.filters]]
enclosing_node = "(your_enclosing_node_pattern) @your_capture_name"
not_contains = [
    """(
    (identifier) @id
    (#eq? @id "x"))
    """,
]
contains =
    """(
    (identifier) @other_id
    (#eq? @other_id "y"))
    """
at_least = 1
at_most = 5
```