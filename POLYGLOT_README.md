# Polyglot Piranha

Polyglot Piranha is a flexible multilingual structural search/replace engine that allows users to apply chains of interdependent structural search/replace rules for deeper cleanups. Polyglot Piranha builds upon tree-sitter queries for expressing the structural search/replace rules.

__This repository contains the Polyglot Piranha framework and pre-built cleanup rules that can be leveraged for deleting code related to stale feature flags__.

## Table of Contents
- [Polyglot Piranha](#polyglot-piranha)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [When is Polyglot Piranha useful?](#when-is-polyglot-piranha-useful)
  - [Using Polyglot Piranha](#using-polyglot-piranha)
    - [:snake: Python API](#snake-python-api)
    - [:computer: Command-line Interface](#computer-command-line-interface)
    - [Languages supported](#languages-supported)
  - [Getting Started with demos](#getting-started-with-demos)
  - [*Stale Feature Flag Cleanup* in depth](#stale-feature-flag-cleanup-in-depth)
  - [Visualizing Graphs for Rules and Groups](#visualizing-graphs-for-rules-and-groups)
  - [Piranha Arguments](#piranha-arguments)
  - [Contributing](#contributing)



## Overview
<p style="text-align:center;">
<img src="images/piranha_architecture.svg" width="800" height="500" alt="Polyglot Piranha Architecture"/>
</p>

This is the higher level architecture of Polyglot Piranha.
At its heart, Polyglot Piranha is a structural find/replacement (rewrite) engine and pre-build language specific cleanup rules like - like simplifying boolean expressions, simplifying `if-else` statements, deleting empty class, deleting files with no type declarations, inline local variables, and many more.
A user provides :
- A set (or, a graph) of structural find/replace rules
- Path to the code base
- [Arguments](#piranha-arguments) to modify Piranha's behavior (like deleting associated comments).

When Piranha applies the set (or graph) of user defined rules, it triggers the __pre-built__ language specific cleanup rules to do a deep cleanup.
Below we can see an [automatically generated graph](#visualizing-graphs-for-rules-and-groups) for the Java pre-built cleanup rules.

<p style="text-align:center;">
    <img src="images/java_prebuilt_rules.svg" width="800" height="500" alt="Java pre-built cleanup rules"/>
</p>

## When is Polyglot Piranha useful?

<h5> Example 1 (Stale Feature Flag Cleanup) </h5>

Let's take an example, where we know for a fact that the expression `exp.isTreated("SHOW_MENU")` always returns `true` (i.e. the feature *Show Menu* is treated)
```
public String fooBar(boolean x) {
    if(exp.isTreated("SHOW_MENU")|| x){
        String menu = getMenu();
        return menu;
    }
    return "";
}
```
To cleanup this code with Piranha, a user would have to write *one* rule to update the expressions like `exp.isTreated("SHOW_MENU")` to `true` and hook it to the pre-built boolean simplification rules. It would result in :
```
public String fooBar(boolean x) {
    String menu = getMenu();
    return menu;
}
```
Note how, user only specified the seed rule to update the expression to true, and Piranha simplified the disjunction (`exp.isTreated("SHOW_MENU")|| x` => `true`), then removed the stale if condition and finally deleted the unreachable return statement (`return "";`).

<h5> Example 2 (Structural Find/Replace with built-in cleanup) </h5>

Let's say a user writes a piranha rule to delete an unused enum case (let's say `LOW`). However, this enum case "co-incidentally" is the only enum case in this enum declaration.
```
enum Level {
  LOW,
}
```
If the user hooks up this *enum case deletion* rule to the pre-built rules, it would not only delete the enum case (`LOW`), but also the consequent empty enum declaration and also optionally delete the consequently empty compilation unit.


<h5> Example 3 (Structural Find/Replace with custom cleanup) </h5>

Let's take a canonical example of replacing `Arrays.asList` with `Collections.singletonList`, when possible.
This task involves two steps (i) Replacing the expression (ii) Adding the import statement for `Collections` if absent (Assuming *google java format* takes care of the unused imports :smile:).
However, Piranha does not contain pre-built rules to add such a custom import statements.
```
import java.util.ArrayList;
import java.util.Arrays;
+ import java.util.Collections;
class Character{
    String name;
    List<String> friends;
    List<String> enemies;

    Character(String name) {
        this.name = name;
        this.friends = new ArrayList<>();
 -         this.enemies = Arrays.asList(this.name);
 +         this.enemies = Collections.singletonList(this.name);
    }
}
```
For such a scenario a developer could first write a seed rule for replacing the expression and then craft a custom "cleanup" rule (that would be triggered by the seed rule) to add the import statement if absent within the same file.

*Note a user can also craft a set of rules that trigger no other rule, i.e. use piranha as a simple structural find/replace tool*

*If you end up implementing a cleanup rule that could be useful for the community, feel free to make a PR to add it into the pre-built language specific rules*

## Using Polyglot Piranha

Polyglot Piranha can be used as a python library or as a command line tool.

### :snake: Python API

<h3> Installing the Python API </h3>

`pip install polyglot-piranha`

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

### :computer: Command-line Interface


Get platform-specific binary from [releases](https://github.com/uber/piranha/releases) or build it from source following the below steps:

* Install [Rust](https://www.rust-lang.org/tools/install)
* `git clone https://github.com/uber/piranha.git`
* `cd piranha`
* `cargo build --release` (`cargo build --release --no-default-features` for macOS)
* Binary will be generated under `target/release`


```
Polyglot Piranha
A refactoring tool that eliminates dead code related to stale feature flags

Usage: polyglot_piranha [OPTIONS] --path-to-codebase <PATH_TO_CODEBASE> --path-to-configurations <PATH_TO_CONFIGURATIONS> -l <LANGUAGE>

Options:
  -c, --path-to-codebase <PATH_TO_CODEBASE>
          Path to source code folder or file
      --include [<INCLUDE>...]
          Paths to include (as glob patterns)
      --exclude [<EXCLUDE>...]
          Paths to exclude (as glob patterns)
          
  -t, --code-snippet <CODE_SNIPPET>
          Code snippet to transform [default: ]
  -s <SUBSTITUTIONS>
          These substitutions instantiate the initial set of rules. Usage : -s stale_flag_name=SOME_FLAG -s namespace=SOME_NS1
  -f, --path-to-configurations <PATH_TO_CONFIGURATIONS>
          Directory containing the configuration files -  `rules.toml` and  `edges.toml` (optional)
  -j, --path-to-output-summary <PATH_TO_OUTPUT_SUMMARY>
          Path to output summary json file
  -l <LANGUAGE>
          The target language [possible values: java, swift, py, kt, go, tsx, ts]
      --delete-file-if-empty
          User option that determines whether an empty file will be deleted
      --delete-consecutive-new-lines
          Replaces consecutive `\n`s  with a `\n`
      --global-tag-prefix <GLOBAL_TAG_PREFIX>
          the prefix used for global tag names [default: GLOBAL_TAG.]
      --number-of-ancestors-in-parent-scope <NUMBER_OF_ANCESTORS_IN_PARENT_SCOPE>
          The number of ancestors considered when `PARENT` rules [default: 4]
      --cleanup-comments-buffer <CLEANUP_COMMENTS_BUFFER>
          The number of lines to consider for cleaning up the comments [default: 2]
      --cleanup-comments
          Enables deletion of associated comments
      --dry-run
          Disables in-place rewriting of code
      --allow-dirty-ast
          Allows syntax errors in the input source code
  -h, --help
          Print help
```

The output JSON is the serialization of- [`PiranhaOutputSummary`](/src/models/piranha_output.rs) produced for each file touched or analyzed by Piranha.

*It can be seen that the Python API is basically a wrapper around this command line interface.*

### Languages supported

| Language         | Structural <br>Find-Replace | Chaining <br>Structural Find <br>Replace | Stale Feature <br>Flag Cleanup  <br> |
| ---------------- | --------------------------- | ---------------------------------------- | ------------------------------------ |
| Java             | :heavy_check_mark:          | :heavy_check_mark:                       | :heavy_check_mark:                   |
| Kotlin           | :heavy_check_mark:          | :heavy_check_mark:                       | :heavy_check_mark:                   |
| Java + Kotlin    | :x:                         | :calendar:                               | :calendar:                           |
| Swift            | :heavy_check_mark:          | :construction:                           | :construction:                       |
| Go               | :heavy_check_mark:          | :heavy_check_mark:                       | :heavy_check_mark:                   |
| Python           | :heavy_check_mark:          | :calendar:                               | :calendar:                           |
| TypeScript       | :heavy_check_mark:          | :calendar:                               | :calendar:                           |
| TypeScript+React | :heavy_check_mark:          | :calendar:                               | :calendar:                           |
| C#               | :calendar:                  | :calendar:                               | :calendar:                           |
| JavaScript       | :calendar:                  | :calendar:                               | :calendar:                           |

Contributions for the :calendar: (`planned`) languages or any other languages are welcome :)


## Getting Started with demos

<h3> Running the Demos </h3>

We believe, the easiest way to get started with Piranha is to build upon the demos.

To setup the demo please follow the below steps:
* `git clone https://github.com/uber/piranha.git`
* `cd piranha`
* Create a virtual environment:
  - `python3 -m venv .env`
  - `source .env/bin/activate`
* Install Polyglot Piranha
  - `pip install --upgrade pip`
  - `pip install .` to run demo against current source code (please install [Rust](https://www.rust-lang.org/tools/install), it takes less than a minute)
  - Or, `pip install polyglot-piranha` to run demos against the latest release.


Currently, we have demos for the following :

<h4>Stale Feature Flag Cleanup:</h4>

  * run `python3 demo/stale_feature_flag_cleanup_demos.py`. It will execute the scenarios listed under [demo/feature_flag_cleanup/java](demo/feature_flag_cleanup/java/configurations/rules.toml) and [demo/feature_flag_cleanup/kt](demo/feature_flag_cleanup/kt/configurations/rules.toml). These scenarios use simple feature flag API.
  * In these demos the `configurations` contain :
    * `rules.toml` : expresses how to capture different feature flag APIs (`isTreated`, `enum constant`)
    * `piranha_arguments.toml` : expresses the flag behavior, i.e. the flag name and whether it is treated or not. Basically the `substitutions` provided in the `piranha_arguments.toml` can be used to instantiate the rules [reference](#piranha-arguments).

<h4>  Match-only rules: </h4>

  * run `python3 demo/match_only_demos.py`
  * This demo also shows how the piranha summary output can be used.
    * `rules.toml` : express how to capture two patterns - (i) invocation of the method `fooBar("...")`  and invocation of the method `barFoo("...")` (but only in non-static methods)

<h4>  Structural Find/Replace </h4>

  * run `python3 demo/find_replace_demos.py`
  * This demo shows how to use Piranha as a simple structural find/replace tool (that optionally hooks up to built-in cleanup rules)

<h4>  Structural Find/Replace with Custom Cleanup </h4>

   * run `python3 demo/find_replace_custom_cleanup_demos.py`
   * This demo shows how to replace `new ArrayList<>()` with `Collections.emptyList()`. Note it also adds the required import statement.


*Please refer to our test cases at [`/test-resources/<language>/`](/test-resources/) as a reference for handling complicated scenarios*


<h3>Building upon the stale feature flag cleanup demo </h3>

First, check if Polyglot Piranha supports *Stale feature flag cleanup* for the required language.

Then see if your API usage is similar to the ones shown in the demo ([java-demo](/demo/java/configurations/rules.toml)) or in the test resources ([java-ff_system1](/test-resources/java/feature_flag_system_1/control/configurations/rules.toml), [java-ff_system2](/test-resources/java/feature_flag_system_2/control/configurations/rules.toml), [kt-ff_system1](/test-resources/kotlin/feature_flag_system_1/control/configurations/rules.toml), [kt-ff_system2](/test-resources/kotlin/feature_flag_system_2/control/configurations/rules.toml)).

If not :|, try to adapt these examples to your requirements. Further, you can study the [tree-sitter query documentation](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries) to understand how tree-sitter queries work. It is recommended to read the section- [Adding support for a new feature flag system](#adding-support-for-a-new-feature-flag-system)

Then adapt the [argument file](/demo/java/configurations/piranha_arguments.toml) as per your requirements. For instance, you may want to update the value corresponding to the `@stale_flag_name` and `@treated`. If your rules do not contain require other tags feel free to remove them from your arguments file. In most cases [edges file](/src/cleanup_rules/java/edges.toml) is not required, unless your feature flag system API rules are inter-dependent.


More details for configuring Piranha - [Adding support for a new feature flag system](#adding-support-for-a-new-feature-flag-system)
and [Adding Cleanup Rules](#adding-cleanup-rules).


*One can similarly build upon the other demos too.*

## *Stale Feature Flag Cleanup* in depth

<h3> Adding support for a new feature flag system </h3>

To onboard a new feature flag system users will have to specify the `<path-to-configurations>/rules.toml` and `<path-to-configurations>/edges.toml` files (look [here](/src/cleanup_rules/java)). The `rules.toml` will contain rules that identify the usage of a feature flag system API. Defining `edges.toml` is required if your feature flag system API rules are inter-dependent.
For instance, you want to delete a method declaration with specific annotations and then update its usages with some boolean value.
Please refer to the `test-resources/java` for detailed examples.


<h3> Adding a new API usage </h3>

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
Currently, Piranha provides deep clean-ups for edits that belong the groups - `replace_expression_with_boolean_literal`, `delete_statement`, and `delete_method`. Basically, by adding an appropriate entry to the groups, a user can hook up their rules to the pre-built cleanup rules.

Setting the `is_seed_rule=False` ensures that the user defined rule is treated as a cleanup rule not as a seed rule (For more details refer to `demo/find_replace_custom_cleanup`).

A user can also define exclusion filters for a rule (`rules.filters`). These filters allow matching against the context of the primary match. For instance, we can write a rule that matches the expression `new ArrayList<>()` and exclude all instances that occur inside static methods (For more details, refer to the `demo/match_only`).

At a higher level, we can say that - Piranha first selects AST nodes matching `rules.query`, excluding those that match **any of** the `rules.filters.not_contains` (within `rules.filters.enclosing_node`). It then replaces the node identified as `rules.replace_node` with the formatted (using matched tags) content of `rules.replace`.

<h3> Parameterizing the behavior of the feature flag API </h3>

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


<h3> Adding Cleanup Rules </h3>

This section describes how to configure Piranha to support a new language. Users who do not intend to onboard a new language can skip this section.
This section will describe how to encode cleanup rules that are triggered based on the update applied to the flag API usages.
These rules should perform cleanups like simplifying boolean expressions, or if statements when the condition is constant, or deleting empty interfaces, or in-lining variables.
For instance, the below example shows a rule that simplifies a `or` operation where its `RHS` is true.
```
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

## Visualizing Graphs for Rules and Groups

Visualizing rules, groups and their edges through a graph is a great way to understand how Piranha Polyglot works.

We use [Graphviz](https://graphviz.org/) to generate a .dot file and a .svg image generated by `visualize_rules_graph.py`.
Please follow the instructions to install Graphviz at <https://graphviz.org/download/>.
Moreover, the script also needs the `toml` and `graphviz` python packages.

The script takes as first argument a path for the generated .dot file.
The following arguments are paths for directories containing `rules.toml` and `edges.toml` files.
Optionally, you can provide a `--title` argument to give a title to the generated graph.
To generate the .dot file and the .svg image used in this README (assuming a Python venv and a valid Graphviz installation):

```bash
pip install toml
pip install graphviz
python visualize_rules_graph.py ./java_prebuilt_rules.dot src/cleanup_rules/java --title "Java pre-built cleanup rules"
```

To generate an image for [java-ff_system1](test-resources/java/feature_flag_system_1/control/configurations/) in our tests:

```bash
python visualize_rules_graph.py ./java-ff_system1.dot src/cleanup_rules/java test-resources/java/feature_flag_system_1/control/configurations --title "Java Test Feature Flag Cleanup System 1"
```


## Piranha Arguments

The purpose of Piranha Arguments is determining the behavior of Piranha.
- `language` : The programming language used by the source code
- `substitutions` : Seed substitutions for the rules (if any). In case of stale feature flag cleanup, we pass the stale feature flag name and whether it is treated or not.
- `delete_file_if_empty` : enables delete file if it consequently becomes empty
-  `delete_consecutive_new_lines` : enables deleting consecutive empty new line
-  `cleanup_comments` : enables cleaning up the comments associated to the deleted code elements like fields, methods or classes
-  `cleanup_comments_buffer` : determines how many lines above to look up for a comment.




## Contributing

Prerequisites: 
* Install [pre-commit](https://pre-commit.com/)
* Install [taplo](https://taplo.tamasfe.dev/cli/introduction.html)

<h4> Naming conventions for the rules </h4>

* We name the rules in the format - <verb>_<ast_kind>. E.g., `delete_method_declaration` or `replace_expression with_boolean_literal`
* We name the dummy rules in the format - `<ast_kind>_cleanup` E.g. `statement_cleanup` or `boolean_literal_cleanup`. Using dummy rules (E.g. [java-rules](/src/cleanup_rules/java/rules.toml): `boolean_literal_cleanup`) makes it easier and cleaner when specifying the flow between rules.

<h4> Writing tests </h4>

Currently we maintain
* Unit tests for the internal functionality can be found under `<models|utilities>/unit_test`.
* End-to-end tests for the configurations execute  Piranha on the test scenarios in `test-resources/<language>/input` and check if the output is as expected (`test-resources/<language>/expected_treated` and `test-resources/<language>/expected_control`).

To add new scenarios to the existing tests for a given language, you can add them to new file in the `input` directory and then create similarly named files with the expected output in `expected_treated` and `expected_control` directory.
Update the `piranha_arguments_treated.toml` and `piranha_arguments_control.toml` files too.

To add tests for a new language, please add a new `<language>` folder inside `test-resources/` and populate the `input`, `expected_treated` and `expected_control` directories appropriately.
