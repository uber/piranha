# Multilingual Piranha
This variant of Piranha provides a flexible multilingual solution for deleting code related to stale feature flags leading to a cleaner, safer, more performant, and more maintainable code base.

## Motivation 

Adding Piranha support for a new language requires re-implementing the entire refactoring logic for that particular language. It is time-consuming and expensive to develop and maintain these similar implementations.
This is overcome by extracting the language specific syntactic transformations to tree-sitter query based rewrite rules, and applying them to the input program as chains of rules.

## Usage 
Piranha can be configured to recognize different flag APIs by specifying a `rules.toml` file (and optionally a `edges.toml`). Piranha will then perform the refactoring based on the flag behavior, which can be specified by providing `piranha_arguments.toml`. Moreover, Piranha can be configured to operate upon a new language by specifying a `/configuration/<lang-name>/rules.toml`, `/configuration/<lang-name>/edges.toml` and `/configuration/<lang-name>/scope_generators.toml`.

```
Piranha
A refactoring tool that eliminates dead code related to stale feature flags.

USAGE:
    piranha --path-to-codebase <PATH_TO_CODEBASE> --path-to-configurations <PATH_TO_CONFIGURATIONS>

OPTIONS:
    -c, --path-to-codebase <PATH_TO_CODEBASE>
            Path to source code folder

    -f, --path-to-configurations <PATH_TO_CONFIGURATIONS>
            Directory containing the configuration files - `piranha_arguments.toml`, `rules.toml`,
            and `edges.toml` (optional)

    -h, --help
            Print help information

    -V, --version
            Print version information
```

Languages supported :
* Java
* Kotlin (planned)
* Java + Kotlin (planned)
* Swift (planned)
* JavaScript (planned)
* Go (requested)
* C# (requested)
* TypeScript (requested)
* Python (requested)
* PHP (requested)
* Contributions for the `requested` languages or any other languages are welcome :) 

## Building Piranha from Source
* Install [Rust](https://www.rust-lang.org/tools/install), Git and [tree-sitter CLI](https://github.com/tree-sitter/tree-sitter/blob/master/cli/README.md)
* Checkout this repository - `git checkout https://github.com/uber/piranha.git` 
* `cd piranha/polyglot/piranha`
* `cargo build --release`
* You will see the binary under `target/release`

## Getting started with Piranha

*Please refer to our [demo](/polyglot/piranha/demo/run_piranha_demo.sh) - to quickly get started with Piranha.*
To run the demo : 
* `cd polyglot/piranha`
* `./demo/run_piranha_demo.sh`
*Please refer to our test cases at [`/polyglot/piranha/src/test-resources/<language>/`](/polyglot/piranha/src/test-resources/) as a reference for handling complicated scenarios*

To get started with Piranha, please follow the below steps:
* Check if the current version of Piranha supports the required language.
* If so, then check if the API usage is similar to the ones shown in the demo ([java-demo](/polyglot/piranha/demo/java/configurations/rules.toml)) or in the test resources ([java-ff_system1](/piranha/test-resources/java/feature_flag_system_1/control/configurations/rules.toml), [java-ff_system2](/piranha/test-resources/java/feature_flag_system_2/control/configurations/rules.toml)).
*  If not, adapt these examples to your requirements. Further, you can study the [tree-sitter query documentation](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries) to understand how tree-sitter queries work.
* Now adapt the [argument file](/polyglot/piranha/demo/java/configurations/piranha_arguments.toml) as per your requirements. For instance, you may want to update the value corresponding to the `@stale_flag_name` and `@treated`. If your rules do not contain require other tags feel free to remove them from your arguments file. In most cases [edges file](/demo/java/configurations/edges.toml) is not required, unless your feature flag system API rules are inter-dependent. 

More details for configuring Piranha - [Adding support for a new feature flag system](adding-support-for-a-new-feature-flag-system)
and [Adding Support for a new language](adding-support-for-a-new-language).

## Adding support for a new feature flag system

To onboard a new feature flag system users will have to specify the `<path-to-configurations>/rules.toml` and `<path-to-configurations>/edges.toml` files (look [here](/polyglot/piranha/src/cleanup_rules/java)). The `rules.toml` will contain rules that identify the usage of a feature flag system API. Defining `edges.toml` is required if your feature flag system API rules are inter-dependent. 
For instance, you want to delete a method declaration with specific annotations and then update its usages with some boolean value. 
Please refer to the `test-resources/java` for detailed examples. 


### Adding a new API usage
The example below shows a usage of a feature flag API (`experiment.isTreated(STALE_FLAG)`), in a `if_statement`. 
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
In the case when STALE_FLAG is treated, we would expect Piranha to refactor the code as shown below (assuming that `STALE_FLAG` is treated) : 
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

Each rule also contains the `groups` property, that specifies the kind of change performed by this rule. Based on this group, appropriate 
cleanup will be performed by Piranha. For instance, `replace_expression_with_boolean_literal` will trigger deep cleanups to eliminate dead code (like eliminating `consequent` of a `if statement`) caused by replacing an expression with a boolean literal. 
Currently, Piranha provides deep clean-ups for edits that belong the groups - `replace_expression_with_boolean_literal`, `delete_statement`, and `delete_method`. 

### Defining behavior of the API
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


## Adding Support for a new language
This section describes how to configure Piranha to support a new language. 
Users who do not intend to onboard a new language can skip this section.
This section will describe how to encode cleanup rules that are triggered based on the update applied to the flag API usages.
These rules should perform cleanups like simplifying boolean expressions, or if statements when the condition is constant, or deleting empty interfaces, or in-lining variables.
For instance, the below example shows a rule that simplifies a `or` operation where its `RHS` is true. 
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
```

Currently, Piranha picks up the language specific configurations from `src/cleanup_rule/<language>`.


### Example
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

We would first define flag API rules as discussed in the section [Adding Support for a new language](adding-support-for-a-new-language). Assuming this rule replaces the occurrence of the flag API corresponding to `SOME_STALE_FLAG` with `true`; we would have to define more cleanup rules as follows:

* `R0`: Deletes the enclosing variable declaration (i.e. `x`) (E.g. [java-rules](/polyglot/piranha/src/cleanup_rules/java/rules.toml):`delete_variable_declarations`)
* `R1`: replace the identifier with the RHS of the deleted variable declaration, within the body of the enclosing method where `R0` was applied i.e. replace `x` with `true` within the method body of `foobar`. (E.g. [java-rules](/polyglot/piranha/src/cleanup_rules/java/rules.toml):`replace_expression_with_boolean_literal`) 
* `R2`: simplify the boolean expressions, for example replace `true || someCondition()` with `true`, that encloses the node where `R1` was applied. (E.g. [java-rules](/polyglot/piranha/src/cleanup_rules/java/rules.toml): `true_or_something`)
* `R3`: eliminate the enclosing if statement with a constant condition where `R2` was applied (`if (true) { return 100;}` → `return 100;`). E.g. [java-rules](/polyglot/piranha/src/cleanup_rules/java/rules.toml): `simplify_if_statement_true, remove_unnecessary_nested_block`
* `R4`: eliminate unreachable code (`return 0;` in `return 100; return 0;`) in the enclosing block where `R3` was applied. (E.g. [java-rules](/polyglot/piranha/src/cleanup_rules/java/rules.toml): `delete_all_statements_after_return`)

The fact that `R2` has to be applied to the enclosing node where `R1` was applied, is expressed by specifying the `edges.toml` file. 

To define how these cleanup rules should be chained, one needs to specify edges (e.g. the [java-edges](/polyglot/piranha/src/cleanup_rules/java/edges.toml) file) between the groups and (or) individual rules.
The edges can be labelled as `Parent`, `Global` or even much finer scopes like `Method` or `Class` (or let's say `functions` in `go-lang`).
* A `Parent` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules on any ancestor of `"n2"` (e.g. `R1` → `R2`, `R2` → `R3`, `R3` → `R4`)
* A `Method` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing method's body. (e.g. `R0` → `R1`)
* A `Class` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules within the enclosing class body. (e.g. in-lining a private field)
* A `Global` edge implies that after Piranha applies the `"from"` rule to update the node `n1` in the AST to node `n2`, Piranha tries to apply `"to"` rules in the entire code base. (e.g. in-lining a public field).

`scope_config.toml` file specifies how to capture these fine-grained scopes like `method`, `function`, `lambda`, `class`.
First decide, what scopes you need to capture, for instance, in Java we capture "Method" and "Class" scopes. Once, you decide the scopes construct scope query generators similar to [java-scope_config](/polyglot/piranha/src/cleanup_rules/java/scope_config.toml). Each scope query generator has two parts - (i) `matcher` is a tree-sitter query that matches the AST for the scope, and (ii) `generator` is a tree-sitter query with holes that is instantiated with the code snippets corresponding to tags when `matcher` is matched.


## Contributing

### Naming conventions for the rules 
* We name the rules in the format - <verb>_<ast_kind>. E.g., `delete_method_declaration` or `replace_expression with_boolean_literal`
* We name the dummy rules in the format - `<ast_kind>_cleanup` E.g. `statement_cleanup` or `boolean_literal_cleanup`. Using dummy rules (E.g. [java-rules](/polyglot/piranha/src/cleanup_rules/java/rules.toml): `boolean_literal_cleanup`) makes it easier and cleaner when specifying the flow between rules.

### Writing tests
Currently we maintain 
* Unit tests for the internal functionality can be found under `<models|utilities>/unit_test`.
* End-to-end tests for the configurations execute  Piranha on the test scenarios in `test-resources/<language>/input` and check if the output is as expected (`test-resources/<language>/expected_treated` and `test-resources/<language>/expected_control`).

To add new scenarios to the existing tests for a given language, you can add them to new file in the `input` directory and then create similarly named files with the expected output in `expected_treated` and `expected_control` directory.
Update the `piranha_arguments_treated.toml` and `piranha_arguments_control.toml` files too.

To add tests for a new language, please add a new `<language>` folder inside `test-resources/` and populate the `input`, `expected_treated` and `expected_control` directories appropriately.
