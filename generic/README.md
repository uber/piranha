# Generic Piranha 
Piranha scans source files to delete code related to stale feature flags leading to a cleaner, safer, more performant, and more maintainable code base.
This generic tree-sitter based implementation for Piranha makes it easy to extend it to new languages, new feature flag APIs (and their usage).


## Motivation 

Adding Piranha support for a new language requires re-implementing the entire refactoring logic for that particular language. This is time consuming and expensive to develop and maintain such nearly similar implementations.
This implementation overcomes this problem by extracting the language specific syntactic transformations to tree-sitter query API based rewrite rules, and applying them to the input program as chains of rules.

## Usage 
Piranha can be configured to recognize different flag APIs and flag behaviors by specifying a `input_rules.toml` file (and optionally a `input_edges.toml`) and the appropriate command line options. Moreover, Piranha can be configured to operate upon a new language by specifying a `rules.toml`, `edges.toml` and `scope_generators.toml`.
```
piranha 0.1.0

USAGE:
    piranha [OPTIONS] --path-to-codebase <PATH_TO_CODEBASE> --language <LANGUAGE> --flag-name <FLAG_NAME> --flag-namespace <FLAG_NAMESPACE> --path-to-configuration <PATH_TO_CONFIGURATION>

OPTIONS:
    -c, --path-to-configuration <PATH_TO_CONFIGURATION>
            Folder containing the required configuration files
    -f, --flag-name <FLAG_NAME>
            Name of the stale flag
    -h, --help
            Print help information
    -l, --language <LANGUAGE>
    -n, --flag-namespace <FLAG_NAMESPACE>
    -p, --path-to-codebase <PATH_TO_CODEBASE>
            Input source code folder
    -t, --flag-value
            is treated?
    -V, --version
            Print version information
```

## Configuring Piranha  
### Adding a flag API and specifying how it will be updated
Consider the following definition of a rule in the `input_rules.toml` file. 
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
(#eq? @n1 "isToggleEnabled")
(#eq? @f "@stale_flag_name")
)"""
replace_node = "mi"
replace = "@treated"
groups = [ "Replace with boolean"]
holes = ["treated", "stale_flag_name"]
```
This specifies a rule that matches against expressions like `exp.isToggleEnabled(SOME_FLAG_NAME)` and replaces it with `true` or `false`. 
The `rule` contains `holes` or template variables that need to be instantiated.
For instance, in the above rule `@treated` and `@stale_flag_name` need to be replaced with some concrete value so that the rule matches only the feature flag API usages corresponding to a specific flag, and replace it specifically with `true` or `false`. 
The `query` property of the rule contains a tree-sitter query that is matched against the source code. 
The node captured by the tag-name specified in the `replace_node` property is replaced with the pattern specified in the `replace` property.
The `replace` pattern can use the tags from the `query` to construct a replacement based on the match (like regex-replace).
Each rule also contains the `groups` property, that specifies the kind of change performed by this rule. Based on this group, appropriate 
cleanup rules will be performed by Piranha. For instance, `Replace with boolean literal` will trigger deep cleanups (like eliminating `consequent` of a `if statement`) to eliminate dead code caused by replacing an expression with a boolean literal. 
Currently, Piranha provides deeper clean up after edits that `Replace with boolean literal` , `Delete Statement`, and `Delete Method`.

### Configuring a new language 
At a higher level, Piranha updates the feature flag API usages related to the stale flag and then cleans up the resulting dead code. 
The configuration discussed in the previous section, describes how to specify the updates to the flag API usages. 
This section will describe how to encode cleanup rules that would be triggered based on the update to the flag API usages.
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
groups = ["Boolean expression cleanup", "Returns boolean"]
```

Currently Piranha picks up the language specific configurations from `/generic/piranha-tree-sitter/src/config`.


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

* `R0`: Deletes the enclosing variable declaration (i.e. `x`) 
* `R1`: replace the identifier with the RHS of the deleted variable declaration, within the body of the enclosing method where `R0` was applied (i.e. replace `x` with `true` within the method body of `foobar`)
* `R2`: simplify the boolean expressions (`true || someCondition()` to `true`, that encloses the node where `R1` was applied )
* `R3`: eliminate the enclosing if statement with a constant condition where `R2` was applied(`if (true) { return 100;}` -> `return 100;`),
* `R4`: eliminate unreachable code (`return 0;` in `return 100; return 0;`) in the enclosing block where `R3` was applied.

The fact that `R2` has to be applied to the enclosing node where `R1` was applied, is expressed by specifying the `edges.toml` file. 

To define how these cleanup rules should be chained, one needs to specify edges (in `edges.toml` file) between the groups and (or) individual rules.
The edges can be labelled as `ANCESTOR`, `METHOD`, `CLASS` or `GLOBAL`. 
An `ANCESTOR` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules on any ancestor of `"n2"` (e.g. `R1` -> `R2`, `R2` -> `R3`, `R3` -> `R4`)
A `METHOD` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing method's body. (e.g. `R0` -> `R1`)
A `CLASS` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing class body. (e.g. inlining a private field)
A `GLOBAL` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules in the entire code base. (e.g. inlining a public field)


