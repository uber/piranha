---
id: staleflags
title: Stale Flag Cleanup
sidebar_label: Stale Flag Cleanup
---

<h3> Adding support for a new feature flag system </h3>

To onboard a new feature flag system users will have to specify the `<path-to-configurations>/rules.toml` and `<path-to-configurations>/edges.toml` files (look [here](/src/cleanup_rules/java)). The `rules.toml` will contain rules that identify the usage of a feature flag system API. Defining `edges.toml` is required if your feature flag system API rules are inter-dependent.
For instance, you want to delete a method declaration with specific annotations and then update its usages with some boolean value.
Please refer to the `test-resources/java` for detailed examples.


<h3> Adding a new API usage </h3>

The example below shows a usage of a feature flag API (`experiment.isTreated(STALE_FLAG)`), in a `if_statement`.
```java
class PiranhaDemo {

    void demoMethod(ExperimentAPI experiment){
        // Some code
        if (experiment.isTreated(STALE_FLAG)) {
            // Do something
        } else {
            // Do something else
        }
        // Do other things
    }
}
```
In the case when STALE_FLAG is treated, we would expect Piranha to refactor the code as shown below (assuming that `STALE_FLAG` is treated) :
```java
class PiranhaDemo {

    void demoMethod(ExperimentAPI experiment){
        // Some code
        // Do something
        // Do other things
    }
}
```
This can be achieved by adding a rule in the `input_rules.toml` file (as shown below) :
```toml
[[rules]]
name = "Enum Based, toggle enabled"
query = """((
    (method_invocation
        name : (_) @n1
        arguments: ((argument_list
                        ([
                          (field_access field: (_)@f)
                          (_) @f
                         ])) )

    ) @mi
)
(#eq? @n1 "isTreated")
(#eq? @f "@stale_flag_name")
)"""
replace_node = "mi"
replace = "@treated"
groups = [ "replace_expression_with_boolean_literal"]
holes = ["treated", "stale_flag_name"]
```
This specifies a rule that matches against expressions like `exp.isTreated(SOME_FLAG_NAME)` and replaces it with `true` or `false`.
The `query` property of the rule contains a [tree-sitter query](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries) that is matched against the source code.
The node captured by the tag-name specified in the `replace_node` property is replaced with the pattern specified in the `replace` property.
The `replace` pattern can use the tags from the `query` to construct a replacement based on the match (like [regex-replace](https://docs.microsoft.com/en-us/visualstudio/ide/using-regular-expressions-in-visual-studio?view=vs-2022)).

Each rule also contains the `groups` property, that specifies the kind of change performed by this rule. Based on this group, appropriate
cleanup will be performed by Piranha. For instance, `replace_expression_with_boolean_literal` will trigger deep cleanups to eliminate dead code (like eliminating `consequent` of a `if statement`) caused by replacing an expression with a boolean literal.
Currently, Piranha provides deep clean-ups for edits that belong the groups - `replace_expression_with_boolean_literal`, `delete_statement`, and `delete_method`. Basically, by adding an appropriate entry to the groups, a user can hook up their rules to the pre-built cleanup rules.

Setting the `is_seed_rule=False` ensures that the user defined rule is treated as a cleanup rule not as a seed rule (For more details refer to `demo/find_replace_custom_cleanup`).

A user can also define exclusion filters for a rule (`rules.filters`). These filters allow matching against the context of the primary match. For instance, we can write a rule that matches the expression `new ArrayList<>()` and exclude all instances that occur inside static methods (For more details, refer to the `demo/match_only`).

At a higher level, we can say that - Piranha first selects AST nodes matching `rules.query`, excluding those that match **any of** the `rules.filters.not_contains` (within `rules.filters.enclosing_node`). It then replaces the node identified as `rules.replace_node` with the formatted (using matched tags) content of `rules.replace`.

<h3> Parameterizing the behavior of the feature flag API </h3>

The `rule` contains `holes` or template variables that need to be instantiated.
For instance, in the above rule `@treated` and `@stale_flag_name` need to be replaced with some concrete value so that the rule matches only the feature flag API usages corresponding to a specific flag, and replace it specifically with `true` or `false`.  To specify such a behavior,
user should create a `piranha_arguments.toml` file as shown below (assuming that the behavior of STALE_FLAG is **treated**):
```toml
language = ["java"]
substitutions = [
    ["stale_flag_name", "STALE_FLAG"],
    ["treated", "true"]
]
```
This file specifies that, the user wants to perform this refactoring for `java` files.
The `substitutions` field captures mapping between the tags and their corresponding concrete values. In this example, we specify that the tag named `stale_flag_name` should be replaced with `STALE_FLAG` and `treated` with `true`.


<h3> Adding Cleanup Rules </h3>

This section describes how to configure Piranha to support a new language. Users who do not intend to onboard a new language can skip this section.
This section will describe how to encode cleanup rules that are triggered based on the update applied to the flag API usages.
These rules should perform cleanups like simplifying boolean expressions, or if statements when the condition is constant, or deleting empty interfaces, or in-lining variables.
For instance, the below example shows a rule that simplifies a `or` operation where its `RHS` is true.
```toml
[[rules]]
name = "Or - right operand is True"
query = """(
(binary_expression
    left : (_)*
    operator:"||"
    right: (true)
) @binary_expression)"""
replace_node = "binary_expression"
replace = "true"
```

Currently, Piranha picks up the language specific configurations from `src/cleanup_rule/<language>`.


<h5> Example </h5>


<div style={{display: 'flex', justifyContent: 'space-around'}}>
    <div style={{width: '49%'}}>


```java
int foobar(){
    boolean x = exp.isTreated(SOME_STALE_FLAG);
        if (x || someCondition()) {
            return 100;
    }
        return 0;
    }
```

</div>
    <div style={{width: '2%'}}>
    </div>
    <div style={{width: '49%'}}>

```java
int foobar(){
    return 100;
}


```

</div>
</div>




We would first define flag API rules as discussed in the section [Adding Support for a new language](#adding-support-for-a-new-language). Assuming this rule replaces the occurrence of the flag API corresponding to `SOME_STALE_FLAG` with `true`; we would have to define more cleanup rules as follows:

* `R0`: Deletes the enclosing variable declaration (i.e. `x`) (E.g. [java-rules](/src/cleanup_rules/java/rules.toml):`delete_variable_declarations`)
* `R1`: replace the identifier with the RHS of the deleted variable declaration, within the body of the enclosing method where `R0` was applied i.e. replace `x` with `true` within the method body of `foobar`. (E.g. [java-rules](/src/cleanup_rules/java/rules.toml):`replace_expression_with_boolean_literal`)
* `R2`: simplify the boolean expressions, for example replace `true || someCondition()` with `true`, that encloses the node where `R1` was applied. (E.g. [java-rules](/src/cleanup_rules/java/rules.toml): `true_or_something`)
* `R3`: eliminate the enclosing if statement with a constant condition where `R2` was applied (`if (true) { return 100;}` → `return 100;`). E.g. [java-rules](/src/cleanup_rules/java/rules.toml): `simplify_if_statement_true, remove_unnecessary_nested_block`
* `R4`: eliminate unreachable code (`return 0;` in `return 100; return 0;`) in the enclosing block where `R3` was applied. (E.g. [java-rules](/src/cleanup_rules/java/rules.toml): `delete_all_statements_after_return`)

The fact that `R2` has to be applied to the enclosing node where `R1` was applied, is expressed by specifying the `edges.toml` file.

To define how these cleanup rules should be chained, one needs to specify edges (e.g. the [java-edges](/src/cleanup_rules/java/edges.toml) file) between the groups and (or) individual rules.
The edges can be labelled as `Parent`, `Global` or even much finer scopes like `Method` or `Class` (or let's say `functions` in `go-lang`).
* A `Parent` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules on any ancestor of `"n2"` (e.g. `R1` → `R2`, `R2` → `R3`, `R3` → `R4`)
* A `Method` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing method's body. (e.g. `R0` → `R1`)
* A `Class` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing class body. (e.g. in-lining a private field)
* A `Global` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules in the entire code base. (e.g. in-lining a public field).

`scope_config.toml` file specifies how to capture these fine-grained scopes like `method`, `function`, `lambda`, `class`.
First decide, what scopes you need to capture, for instance, in Java we capture "Method" and "Class" scopes. Once, you decide the scopes construct scope query generators similar to [java-scope_config](/src/cleanup_rules/java/scope_config.toml). Each scope query generator has two parts - (i) `matcher` is a tree-sitter query that matches the AST for the scope, and (ii) `generator` is a tree-sitter query with holes that is instantiated with the code snippets corresponding to tags when `matcher` is matched.
