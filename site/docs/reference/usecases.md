---
id: usecases
title: When is Polyglot Piranha useful?
sidebar_label: Why Piranha?
---


### Example 1 (Stale Feature Flag Cleanup) 

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

### Example 2 (Structural Find/Replace with built-in cleanup)

Let's say a user writes a piranha rule to delete an unused enum case (let's say `LOW`). However, this enum case "co-incidentally" is the only enum case in this enum declaration.
```java
enum Level {
  LOW,
}
```
If the user hooks up this *enum case deletion* rule to the pre-built rules, it would not only delete the enum case (`LOW`), but also the consequent empty enum declaration and also optionally delete the consequently empty compilation unit.


### Example 3 (Structural Find/Replace with custom cleanup)

Let's take a canonical example of replacing `Arrays.asList` with `Collections.singletonList`, when possible.
This task involves two steps (i) Replacing the expression (ii) Adding the import statement for `Collections` if absent (Assuming *google java format* takes care of the unused imports :smile:).
However, Piranha does not contain pre-built rules to add such a custom import statements.
````diff
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
````

For such a scenario a developer could first write a seed rule for replacing the expression and then craft a custom "cleanup" rule (that would be triggered by the seed rule) to add the import statement if absent within the same file.

:::info
Users can also craft a set of rules that trigger no other rules, i.e. use piranha as a simple structural find/replace tool.

If you end up implementing a cleanup rule that could be useful for the community, feel free to make a PR to add it into the pre-built language specific rules*
:::