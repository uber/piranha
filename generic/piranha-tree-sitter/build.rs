/*
Copyright (c) 2019 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use std::path::PathBuf;

// TODO: Add a way to checkout github repository, generate

fn build(tree_sitter_src: &str) {
    let dir = PathBuf::from(format!("../tree-sitter-src/{tree_sitter_src}/src"));
    let mut build = cc::Build::new();
    build.include(&dir).warnings(false);
    build.file(dir.join("parser.c"));
    if dir.join("scanner.c").exists() {
        build.file(dir.join("scanner.c"));
    }
    build.compile(tree_sitter_src);
}

fn main() {
    build("tree-sitter-java");
    build("tree-sitter-swift");
}
