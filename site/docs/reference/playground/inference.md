---
id: inference
title: Rule Inference
sidebar_label: Rule Inference
---


Writing Piranha rules can become complex and cumbersome when you choose to use the tree-sitter queries
for matching code. While Piranha's implementation also supports regex rules,
they tend to be less precise and can also become quite tricky.

To make it easier to write rewrite rules, **Piranha offers an inference mechanism that allows you to generate tree-sitter matchers and the rule graph from provided templates or examples**.
This approach makes it easier to write precise matchers for transforming code

## Rule Generation from Examples

Here's how you can leverage Piranha's rule generation feature:

You simply annotate your code with comments indicating the 'before' and 'after' states.
Additionally, you should also specify how to cascade these transformations with the `->` notation.
For instance, consider the following Java code:



<div style={{display: 'flex', justifyContent: 'space-around'}}>
    <div style={{width: '49%'}}>


```java title="Code before" {1,8,13} showLineNumbers
// 1
import java.util.Executor;

// end

class SomeClass {
  void someMethod() {
    // 2
    Executor(x,y,z);
    // end
  }
}
// 2 -> 1
```

</div>
    <div style={{width: '2%'}}>
    </div>
    <div style={{width: '49%'}}>

```java title="Code after" {1,8} showLineNumbers
// 1
import java.util.NovelExecutor;
import java.util.Wrapper;
// end

class SomeClass {
  void someMethod() {
    // 2
    Wrapper(NovelExecutor(x,y));
    // end
  }
}

```

</div>
</div>

From these annotated examples, Piranha can automatically generate the rule graph for you:

```toml
[[rules]]
name = "import_rule"
query = """(
 (import_declaration
  (scoped_identifier
    scope: (scoped_identifier
     scope: (identifier) @base_package
     name: (identifier) @sub_package) @scoped_import
    name: (identifier) @old_class) @import_stmt
 (#eq? @base_package "java")
 (#eq? @sub_package "util")
 (#eq? @old_class "Executor"))
"""
replace_node = "import_stmt"
replace = "@sub_package @scoped_import . Novel@old_class ; @sub_package @scoped_import . Wrapper ;"

[[rules]]
name = "method_rule"
query = """(
 (method_invocation
   name: (identifier) @old_method_name
   arguments: (argument_list
    (identifier) @arg_x
    (identifier) @arg_y
    (identifier) @arg_z) @arguments) @method_invocation
 (#eq? @old_method_name "Executor")
 (#eq? @arg_x "x")
 (#eq? @arg_y "y")
 (#eq? @arg_z "z"))
"""
replace_node = "method_invocation"
replace = "Wrapper ( Novel@old_method_name ( @arg_x , @arg_y ) )"

[[edges]]
scope = "File"
from = "method_rule"
to = ["import_rule"]

```

:::tip
When annotating your code for rule generation, be sure to only surround the exact piece of code you want to transform with the `// x` and `// end` annotations.
Including more than necessary may result in overly specific matchers which may limit the applicability of your rules.
:::

:::info
Currently, we don't support overlap between templates. Consequently, there should not be any nesting of templates in the form `// x // y // end // end`. Ensure your templates are independent and properly closed within their own start and end tags to avoid conflicts.
:::


## Using template variables

Piranha also supports comby-style template variables, allowing you to abstract away specific parts of the code in your rewrite rules.
Placeholders can be useful when you want to ignore or generalize certain nodes to prevent overly specific templates.
For example:

```java
// Before
:[x].someMethod(:[y]);

// After
newMethod(:[x], :[y: identifier]);
```

This generates the rule:

```toml {5,8} showLineNumbers
[[rules]]
name = "wildcard_rule"
query = """(
 (method_invocation
  object: (_) @wild_card
  name: (identifier ) @old_method_name
  arguments: (argument_list
    (identifier) @method_arg
  ) @args) @method_invocation
  (#eq? @old_method_name "someMethod"))"""
replace_node = "method_invocation"
replace = """newMethod ( @wild_card , @method_arg )"""
```

In the above template, `:[x]` is a match wildcard, and `:[y: identifier]` is a type-constrained wildcard, which will match any node of the specified type(s). We also support alternations or multiple nodes like `:[x: identifier, string_literal]`.

:::tip
Templating syntax offers enhanced control over the code constructs you want to match.

- Use the `:[x]` wildcard when you want to match any node regardless of its type.
- For specific node type matching, use the type-constrained wildcard `:[x:node1,node2,node3]`. This allows you to precisely select the node types you want to match, improving the accuracy and flexibility of your rules.
:::

