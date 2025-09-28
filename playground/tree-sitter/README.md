# Tree-sitter + Concrete Syntax Playground for Piranha

Piranha uses several grammar repositories with custom patches to support the transformations. While
these patches are being upstreamed, there may be discrepancies between the grammars in this 
repository and the upstream grammars.

This directory contains the build script to

(1) instantiate the index.html.template (slightly modified from [tree-sitter-cli-playground-html]) 
  and copy it to the given `dist` directory.

(2) clone the (custom) grammar repositories that we use in Piranha (by parsing the `Cargo.toml` 
  file), check out the specific versions, and then build the WASM files for the grammars and copy
  them to the `dist/assets` directory.

(3) automatically build the concrete syntax WASM bindings from the `concrete-syntax` crate and copy 
  them to the `dist` directory, enabling pattern matching functionality in the playground.

(4) the `dist` directory can be served by any web server (e.g., `python -m http.server`). 

We host the playground at https://uber.github.io/piranha/tree-sitter-playground/ via GitHub Pages.


[tree-sitter-cli-playground-html]: https://github.com/tree-sitter/tree-sitter/blob/eaa10b279f208b47f65e77833d65763f072f3030/crates/cli/src/playground.html#L13