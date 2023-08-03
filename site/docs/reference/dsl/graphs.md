---
id: graphs
label: Graph Language
title: Graph Language
---

Piranha Polyglot uses a domain-specific language (DSL) to specify program transformations. The DSL is used to define rules, scopes, and edges between rules. 
You either build rules using TOML are defined using TOML (Tom's Obvious, Minimal Language) or the Python API.

## Rules

Individual edits are represented as rules in Polyglot Piranha. Each rule represents a transformation that identifies and modifies specific code snippets.
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


Example of a rule in TOML:
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

[[rules.filters]]
enclosing_node = "(your_enclosing_node_pattern) @your_capture_name"
contains =
    """(
    (identifier) @other_id
    (#eq? @other_id "y"))
    """
at_least = 1
at_most = 5
```

## Edges

Edges in Polyglot Piranha allow rules to depend on each other, establishing a hierarchy or sequence of application among rules. An edge essentially describes the direction of dependency between two or more rules. It signifies that a particular rule (`from`) is based on, or derives information from, one or more other rules (`to`).
Edges are also represented in the TOML format.

Example edges in TOML:
```toml
[[edges]]
scope = "Method"
from = "your_rule_name"
to = ["other_rule_name", "another_rule_name"]

[[edges]]
scope = "Method"
from = "other_rule_name"
to = ["your_rule_name"]
```

## Scopes
