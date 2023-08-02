---
id: cli
label: CLI
title: Command Line Interface
---


Get platform-specific binary from [releases](https://github.com/uber/piranha/releases) or build it from source following the below steps:

* Install [Rust](https://www.rust-lang.org/tools/install)
* `git clone https://github.com/uber/piranha.git`
* `cd piranha`
* `cargo build --release` (`cargo build --release --no-default-features` for macOS)
* Binary will be generated under `target/release`


```shell
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