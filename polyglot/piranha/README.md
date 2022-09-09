# Polyglot Piranha 

Polyglot Piranha is a flexible multilingual structural search/replace engine that allows users to apply chains (actually, a graph) of interdependent structural search/replace rules. Polyglot Piranha builds upon tree-sitter queries for expressing the structural search/replace rules.

__This repository contains the Polyglot Piranha framework and its instantiation for deleting code related to stale feature flags__.


## Using Polyglot Piranha

Polyglot Piranha can be used as a python library or as a command line tool.

### :snake: Python API 

### Installing the Python API

`pip install polyglot_piranha`

Currently, we support one simple API (`run_piranha_cli`) that wraps the command line usage of Polyglot Piranha. We believe this makes it easy to incorporate Piranha in *"pipelining"*. 

#### `run_piranha_cli`

```
from polyglot_piranha import run_piranha_cli

path_to_codebase = "..." 
path_to_configurations = "..." 
piranha_summary = run_piranha_cli(path_to_codebase,
                                  path_to_configurations,
                                  should_rewrite_files=True)
```
##### Arguments 
- `path_to_codebase` : Path to source code folder
- `path_to_configuration` : A directory containing files named `piranha_arguments.toml`, `rules.toml` and optionally `edges.toml`
  * `piranha_arguments.toml`: Allows a user to choose language (`java`, `kotlin`, ...), opt-in/out of other features like cleaning up comments, or even provide arguments to the piranha rules
  * `rules.toml`: *piranha rules* expresses the specific AST patterns to match and __replacement patterns__ for these matches (in-place).
  * `edges.toml` (_optional_): expresses the flow between the rules 
- `should_rewrite_files` : Enables in-place rewriting of code 

##### Returns:

`[Piranha_Output]` : a [`PiranhaOutputSummary`](/polyglot/piranha/src/models/piranha_output.rs) for each file touched or analyzed by Piranha. It contains useful information like, matches found (for *match-only* rules), rewrites performed, and content of the file after the rewrite. The content is particularly useful when `should_rewrite_files` is passed as `false`. 

### :computer: Command-line Interface


##### Get platform-specific binary from [releases](https://github.com/uber/piranha/releases) or build it from source following the below steps:

* Install [Rust](https://www.rust-lang.org/tools/install)
* `git checkout https://github.com/uber/piranha.git` 
* `cd piranha/polyglot/piranha`
* `cargo build --release` (`cargo build --release --no-default-features` for macOS)
* Binary will be generated under `target/release`


```
Polyglot Piranha
A refactoring tool that eliminates dead code related to stale feature flags.

USAGE:
    polyglot_piranha --path-to-codebase <PATH_TO_CODEBASE> --path-to-configurations <PATH_TO_CONFIGURATIONS>

OPTIONS:
    -c, --path-to-codebase <PATH_TO_CODEBASE>
            Path to source code folder

    -f, --path-to-configurations <PATH_TO_CONFIGURATIONS>
            Directory containing the configuration files - `piranha_arguments.toml`, `rules.toml`,
            and  `edges.toml` (optional)

    -h, --help
            Print help information

    -j, --path-to-output-summary <PATH_TO_OUTPUT_SUMMARY>
            Path to output summary json
```

The output JSON is the serialization of- [`PiranhaOutputSummary`](/polyglot/piranha/src/models/piranha_output.rs) produced for each file touched or analyzed by Piranha.

*It can be seen that the Python API is basically a wrapper around this command line interface.*

### Languages supported :

| Language   | Structural <br>Find-Replace  | Chaining <br>Structural Find <br>Replace | Stale Feature <br>Flag Cleanup  <br>  :broom: |
|------------|------------------------------|------------------------------------------|---------------------------------|
| Java       | :heavy_check_mark:           | :heavy_check_mark:                       | :heavy_check_mark:              |
| Kotlin     | :heavy_check_mark:           | :heavy_check_mark:                       | :heavy_check_mark:              |
| Java + Kotlin      | :x:           | :calendar:        | :calendar:  |
| Swift      | :heavy_check_mark:           | :construction:        | :construction:   |
| Go         | :construction:                    | :construction:                                | :construction:                       |
| Python     | :calendar:                    | :calendar:                                | :calendar: |
| TypeScript | :calendar:                    | :calendar:                                | :calendar:                       |
| C#         | :calendar:                 | :calendar:                          | :calendar:                    |
| JavaScript | :calendar:                 | :calendar:                          | :calendar:                    |
| Strings Resource | :heavy_check_mark:  | :x: | :x:                    |

Contributions for the :calendar: (`planned`) languages or any other languages are welcome :) 


## Getting Started

### Demos

#### Running the Demos
*Please refer to our [demo](/polyglot/piranha/demo/run_piranha_demo.sh) - to quickly get started with using Piranha stale feature flag cleanup.* 
To run the demo : 
* `cd polyglot/piranha`
* `./demo/run_piranha_demo.sh`


Currently, we have 4 demos : 
- [demo/java](/polyglot/piranha/demo/java/configurations/rules.toml) and [demo/kt](/polyglot/piranha/demo/kt/configurations/rules.toml) showcase *stale feature flag cleanups* :broom: for a simple feature flag API using Piranha. 
  * In these demos the `/demo/java/configurations` and `/demo/kt/configurations` contain :
    * `rules.toml` : expresses how to capture different feature flag APIs (`isTreated`, `enum constant`)
    * `piranha_arguments.toml` : expresses the flag behavior, i.e. the flag name and whether it is treated or not. Basically the `substitutions` provided in the `piranha_arguments.toml` can be used to instantiate the rules.
- [demo/strings](/polyglot/piranha/demo/strings/configurations/rules.toml) and [demo/swift](/polyglot/piranha/demo/swift/configurations/rules.toml) showcase simple *structural find/replace* using Piranha - like regex/replace but using tree-sitter query.

*Please refer to our test cases at [`/polyglot/piranha/test-resources/<language>/`](/polyglot/piranha/test-resources/) as a reference for handling complicated scenarios*


#### Building upon the  *stale feature flag cleanup* :broom: demos: 

First, check if Polyglot Piranha supports *Stale feature flag cleanup* for the required language.

Then see if your API usage is similar to the ones shown in the demo ([java-demo](/polyglot/piranha/demo/java/configurations/rules.toml)) or in the test resources ([java-ff_system1](/polyglot/piranha/test-resources/java/feature_flag_system_1/control/configurations/rules.toml), [java-ff_system2](/polyglot/piranha/test-resources/java/feature_flag_system_2/control/configurations/rules.toml), [kt-ff_system1](/polyglot/piranha/test-resources/kotlin/feature_flag_system_1/control/configurations/rules.toml), [kt-ff_system2](/polyglot/piranha/test-resources/kotlin/feature_flag_system_2/control/configurations/rules.toml)).

If not :|, try to adapt these examples to your requirements. Further, you can study the [tree-sitter query documentation](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries) to understand how tree-sitter queries work. It is recommended to read the section- [Adding support for a new feature flag system](#adding-support-for-a-new-feature-flag-system)

Then adapt the [argument file](/polyglot/piranha/demo/java/configurations/piranha_arguments.toml) as per your requirements. For instance, you may want to update the value corresponding to the `@stale_flag_name` and `@treated`. If your rules do not contain require other tags feel free to remove them from your arguments file. In most cases [edges file](/polyglot/piranha/src/cleanup_rules/java/edges.toml) is not required, unless your feature flag system API rules are inter-dependent. 


More details for configuring Piranha - [Adding support for a new feature flag system](#adding-support-for-a-new-feature-flag-system)
and [Adding Cleanup Rules](#adding-cleanup-rules).


## :toolbox: *Stale Feature Flag Cleanup* Under the hood 

### Adding support for a new feature flag system
To onboard a new feature flag system users will have to specify the `<path-to-configurations>/rules.toml` and `<path-to-configurations>/edges.toml` files (look [here](/polyglot/piranha/src/cleanup_rules/java)). The `rules.toml` will contain rules that identify the usage of a feature flag system API. Defining `edges.toml` is required if your feature flag system API rules are inter-dependent. 
For instance, you want to delete a method declaration with specific annotations and then update its usages with some boolean value. 
Please refer to the `test-resources/java` for detailed examples. 


#### Adding a new API usage
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

#### Defining behavior of the API
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


### Adding Cleanup Rules

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

We would first define flag API rules as discussed in the section [Adding Support for a new language](#adding-support-for-a-new-language). Assuming this rule replaces the occurrence of the flag API corresponding to `SOME_STALE_FLAG` with `true`; we would have to define more cleanup rules as follows:

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