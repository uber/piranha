---
id: intro
title: Overview
sidebar_label: Overview
---

import useBaseUrl from '@docusaurus/useBaseUrl';

Polyglot Piranha is a flexible multilingual structural search/replace engine that allows users to apply chains of interdependent structural search/replace rules for deeper cleanups. Polyglot Piranha builds upon tree-sitter queries for expressing the structural search/replace rules.

<div style={{display: 'flex', justifyContent: 'center'}}>
  <img src={useBaseUrl('/img/piranha_architecture.svg')} alt="Polyglot Piranha Architecture" width="800" height="500"/>
</div>

This is the higher level architecture of Polyglot Piranha.
At its heart, Polyglot Piranha is a structural find/replacement (rewrite) engine and pre-build language specific cleanup rules like - like simplifying boolean expressions, simplifying `if-else` statements, deleting empty class, deleting files with no type declarations, inline local variables, and many more.
A user provides :
- A set (or, a graph) of structural find/replace rules
- Path to the code base
- [Arguments](#piranha-arguments) to modify Piranha's behavior (like deleting associated comments).

When Piranha applies the set (or graph) of user defined rules, it triggers the __pre-built__ language specific cleanup rules to do a deep cleanup.
Below we can see an [automatically generated graph](#visualizing-graphs-for-rules-and-groups) for the Java pre-built cleanup rules.

<div style={{display: 'flex', justifyContent: 'center'}}>
  <img src={useBaseUrl('/img/java_prebuilt_rules.svg')} alt="Java pre-built cleanup rules" width="800" height="500"/>
</div>


## When is Polyglot Piranha useful?

<h5> Example 1 (Stale Feature Flag Cleanup) </h5>

Let's take an example, where we know for a fact that the expression `exp.isTreated("SHOW_MENU")` always returns `true` (i.e. the feature *Show Menu* is treated)
```java
public String fooBar(boolean x) {
    if(exp.isTreated("SHOW_MENU")|| x){
        String menu = getMenu();
        return menu;
    }
    return "";
}
```
To cleanup this code with Piranha, a user would have to write *one* rule to update the expressions like `exp.isTreated("SHOW_MENU")` to `true` and hook it to the pre-built boolean simplification rules. It would result in :
```java
public String fooBar(boolean x) {
    String menu = getMenu();
    return menu;
}
```
Note how, user only specified the seed rule to update the expression to true, and Piranha simplified the disjunction (`exp.isTreated("SHOW_MENU")|| x` => `true`), then removed the stale if condition and finally deleted the unreachable return statement (`return "";`).

<h5> Example 2 (Structural Find/Replace with built-in cleanup) </h5>

Let's say a user writes a piranha rule to delete an unused enum case (let's say `LOW`). However, this enum case "co-incidentally" is the only enum case in this enum declaration.
```java
enum Level {
  LOW,
}
```
If the user hooks up this *enum case deletion* rule to the pre-built rules, it would not only delete the enum case (`LOW`), but also the consequent empty enum declaration and also optionally delete the consequently empty compilation unit.


<h5> Example 3 (Structural Find/Replace with custom cleanup) </h5>

Let's take a canonical example of replacing `Arrays.asList` with `Collections.singletonList`, when possible.
This task involves two steps (i) Replacing the expression (ii) Adding the import statement for `Collections` if absent (Assuming *google java format* takes care of the unused imports :smile:).
However, Piranha does not contain pre-built rules to add such a custom import statements.
```java
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

