---
id: demos
title: Demos
sidebar_label: Demos
---

We believe, the easiest way to get started with Piranha is to build upon the demos.

To setup the demo please follow the below steps:
* `git clone https://github.com/uber/piranha.git`
* `cd polyglot/piranha`
* Create a virtual environment:
   - `python3 -m venv .env`
   - `source .env/bin/activate`
* Install Polyglot Piranha
   - `pip install --upgrade pip`
   - `pip install .` to run demo against current source code (please install [Rust](https://www.rust-lang.org/tools/install), it takes less than a minute)
   - Or, `pip install polyglot-piranha` to run demos against the latest release.


Currently, we have demos for the following :

### Stale Feature Flag Cleanup

* run `python3 demo/stale_feature_flag_cleanup_demos.py`. It will execute the scenarios listed under [demo/java/ff](/demo/java/ff/configurations/rules.toml) and [demo/kt/ff](/demo/kt/ff/configurations/rules.toml). These scenarios use simple feature flag API.
* In these demos the `configurations` contain :
   * `rules.toml` : expresses how to capture different feature flag APIs (`isTreated`, `enum constant`)
   * `piranha_arguments.toml` : expresses the flag behavior, i.e. the flag name and whether it is treated or not. Basically the `substitutions` provided in the `piranha_arguments.toml` can be used to instantiate the rules [reference](#piranha-arguments).

### Match-only rules

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


### Building upon the stale feature flag cleanup demo

First, check if Polyglot Piranha supports *Stale feature flag cleanup* for the required language.

Then see if your API usage is similar to the ones shown in the demo ([java-demo](/demo/java/configurations/rules.toml)) or in the test resources ([java-ff_system1](/test-resources/java/feature_flag_system_1/control/configurations/rules.toml), [java-ff_system2](/test-resources/java/feature_flag_system_2/control/configurations/rules.toml), [kt-ff_system1](/test-resources/kotlin/feature_flag_system_1/control/configurations/rules.toml), [kt-ff_system2](/test-resources/kotlin/feature_flag_system_2/control/configurations/rules.toml)).

If not :|, try to adapt these examples to your requirements. Further, you can study the [tree-sitter query documentation](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries) to understand how tree-sitter queries work. It is recommended to read the section- [Adding support for a new feature flag system](#adding-support-for-a-new-feature-flag-system)

Then adapt the [argument file](/demo/java/configurations/piranha_arguments.toml) as per your requirements. For instance, you may want to update the value corresponding to the `@stale_flag_name` and `@treated`. If your rules do not contain require other tags feel free to remove them from your arguments file. In most cases [edges file](/src/cleanup_rules/java/edges.toml) is not required, unless your feature flag system API rules are inter-dependent.

More details for configuring Piranha - [Adding support for a new feature flag system](#adding-support-for-a-new-feature-flag-system)
and [Adding Cleanup Rules](#adding-cleanup-rules).



:::info

One can similarly build upon the other demos too.
:::