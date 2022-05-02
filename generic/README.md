# Generic Piranha 
Piranha scans source files to delete code related to stale feature flags leading to a cleaner, safer, more performant, and more maintainable code base.
This generic tree-sitter based implementation for Piranha makes it easy to extend it to new languages, new feature flag APIs (and their usage).


## Motivation 

Adding Piranha support for a new language requires re-implementing the entire refactoring logic for that particular language. This is time consuming and expensive to develop and maintain such nearly similar implementations.
This implementation overcomes this problem by extracting the language specific syntactic transformations to tree-sitter query API based rewrite rules, and applying them to the input program as chains of rules.

## Usage 
Piranha can be configured to recognize different flag APIs by specifying a `rules.toml` file (and optionally an `edges.toml`). Piranha can be then perform the refactoring based on the flag behavior, which can be specified by providing `piranha_arguments.toml` . Moreover, Piranha can be configured to operate upon a new language by specifying a `/configuration/<lang-name>/rules.toml`, `/configuration/<lang-name>/edges.toml` and `/configuration/<lang-name>/scope_generators.toml`.

```
piranha 0.1.0

USAGE:
    piranha --path-to-codebase <PATH_TO_CODEBASE> --path-to-feature-flag-rules <PATH_TO_FEATURE_FLAG_RULES> --path-to-piranha-arguments <PATH_TO_PIRANHA_ARGUMENTS>

OPTIONS:
    -c, --path-to-codebase <PATH_TO_CODEBASE>
            Path to source code folder

    -f, --path-to-feature-flag-rules <PATH_TO_FEATURE_FLAG_RULES>
            Folder containing the required configuration files

    -h, --help
            Print help information

    -p, --path-to-piranha-arguments <PATH_TO_PIRANHA_ARGUMENTS>
            Path to the file containing arguments for Piranha

    -V, --version
            Print version information
```

## Configuring Piranha  
### Adding a flag API and specifying how it will be updated

The example below shows a usage of an feature flag API (`experiment.isTreated(STALE_FLAG)`), in an `if_statement`. 
```
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
In the case when STALE_FLAG is treated, we would expect Piranha to refactor the code as shown below (assuming ) : 
```
class PiranhaDemo {

    void demoMethod(ExperimentAPI experiment){
        // Some code 
        // Do something
        // Do other things
    }
}
```
This can be achieved by adding a rule in the `input_rules.toml` file (as shown below) :
```
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

The `rule` contains `holes` or template variables that need to be instantiated.
For instance, in the above rule `@treated` and `@stale_flag_name` need to be replaced with some concrete value so that the rule matches only the feature flag API usages corresponding to a specific flag, and replace it specifically with `true` or `false`.  To specify such a behavior,
user should create a `piranha_arguments.toml` file as shown below (assuming that the behavior of STALE_FLAG is **treated**): 
```
language = ["java"]
substitutions = [
    ["stale_flag_name", "STALE_FLAG"],
    ["treated", "true"]
]
```
This file specifies that, the user wants to perform this refactoring for `java` files. 
The `substitutions` field captures mapping between the tags and their corresponding concrete values. In this example, we specify that the tag named `stale_flag_name` should be replaced with `STALE_FLAG` and `treated` with `true`.



Each rule also contains the `groups` property, that specifies the kind of change performed by this rule. Based on this group, appropriate 
cleanup rules will be performed by Piranha. For instance, `replace_expression_with_boolean_literal` will trigger deep cleanups (like eliminating `consequent` of a `if statement`) to eliminate dead code caused by replacing an expression with a boolean literal. 
Currently, Piranha provides deep clean ups for edits that belong the groups -  `replace_expression_with_boolean_literal` , `delete_statement`, and `delete_method`. 

### Configuring a new language 
This section describes how to configure Piranha to support a new language. 
Users who do not intend to onboard a new language can skip this section.
This section will describe how to encode cleanup rules that are triggered based on the update applied to the flag API usages.
These rules should perform cleanups like simplifying boolean expressions, or if statements when the condition is constant, or deleting empty interfaces, or inlining variables.
For instance, the below example shows a rule that simplifies a `or` operation where its RHS is true. 
```
[[rules]]
name = "Or - right operand is True"
query = """
(
    (binary_expression
        left : (_)* @other
        operator:"||"
        right: (true)
    )
@b)"""
replace_node = "b"    
replace = "true"
groups = ["Boolean expression cleanup", "replace_expression_with_boolean_literal"]
```

Currently, Piranha picks up the language specific configurations from `src/cleanup_rule/<language>`.


# Example
Let's consider an example where we want to define a cleanup for the scenario where 
<table>
<tr>
<td> Before </td> <td> After </td>
</tr>
<tr>
<td>

```
int foobar(){
    boolean x = exp.isTreated(SOME_STALE_FLAG);
    if (x || someCondition()) {
        return 100;
    }
    return 0;
}
```

</td>

<td>

```
int foobar(){
    return 100;
}
```

</td>

</table>

We would first define flag API rules as discussed in the section *Configuring Piranha*. Let's say this rule (`F`) would replaces the occurrence of the flag API corresponding to `SOME_STALE_FLAG` with `true`. To perform the desired refactoring we would have to define cleanup rules as follows :

* `R0`: Deletes the enclosing variable declaration (i.e. `x`) (E.g. `cleanup_rules/java/rules.toml: delete_variable_declarations`)
* `R1`: replace the identifier with the RHS of the deleted variable declaration, within the body of the enclosing method where `R0` was applied i.e. replace `x` with `true` within the method body of `foobar`. (E.g. `cleanup_rules/java/rules.toml: replace_expression_with_boolean_literal`) 
* `R2`: simplify the boolean expressions, for example replace `true || someCondition()` with `true`, that encloses the node where `R1` was applied. (E.g. `cleanup_rules/java/rules.toml: true_or_something`)
* `R3`: eliminate the enclosing if statement with a constant condition where `R2` was applied(`if (true) { return 100;}` -> `return 100;`. (E.g. `cleanup_rules/java/rules.toml:  simplify_if_statement_true, remove_unnecessary_nested_block`)
* `R4`: eliminate unreachable code (`return 0;` in `return 100; return 0;`) in the enclosing block where `R3` was applied.
(E.g. `cleanup_rules/java/rules.toml:  delete_all_statements_after_return`)

The fact that `R2` has to be applied to the enclosing node where `R1` was applied, is expressed by specifying the `edges.toml` file. 

To define how these cleanup rules should be chained, one needs to specify edges (in `edges.toml` file) between the groups and (or) individual rules.
The edges can be labelled as `ANCESTOR`, `METHOD`, `CLASS` or `GLOBAL`. 
* An `ANCESTOR` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules on any ancestor of `"n2"` (e.g. `R1` -> `R2`, `R2` -> `R3`, `R3` -> `R4`)
* A `METHOD` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing method's body. (e.g. `R0` -> `R1`)
* A `CLASS` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing class body. (e.g. inlining a private field)
* A `GLOBAL` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules in the entire code base. (e.g. inlining a public field).

One would also have to define how to capture the `METHOD` and `CLASS` scopes for the new language by specifying the `scope_config.toml` file.
Please see `/src/cleanup_rules/java`.
