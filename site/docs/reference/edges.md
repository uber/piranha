---
id: edges
label: Edges
title: Defining edges
---


Edges in Polyglot Piranha allow rules to depend on each other, establishing a hierarchy or sequence of application among rules. An edge essentially describes the direction of dependency between two or more rules. It signifies that a particular rule (`from`) is based on, or derives information from, one or more other rules (`to`).
Edges are also represented in the TOML format.

Examples:
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