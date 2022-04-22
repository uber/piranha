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

fn build(package_name: &str, package_dir: &str, extra_files: &[&str]) {
    let dir = PathBuf::from(package_dir);

    let mut c_files = vec!["parser.c"];
    let mut cpp_files = vec![];

    for file in extra_files {
        if file.ends_with(".c") {
            c_files.push(file);
        } else {
            cpp_files.push(file);
        }
    }

    if !cpp_files.is_empty() {
        let mut cpp_build = cc::Build::new();
        cpp_build.include(&dir).cpp(true);
        for file in cpp_files {
            cpp_build.file(dir.join(file));
        }
        cpp_build.compile(&format!("{}-cpp", package_name));
    }

    let mut build = cc::Build::new();
    build.include(&dir).warnings(false); // ignore unused parameter warnings
    for file in c_files {
        build.file(dir.join(file));
    }
    build.compile(package_name);
}

fn main() {
    // Only rerun if files in the vendor/ directory change.
    println!("cargo:rerun-if-changed=vendor");
    build("tree-sitter-java", "../tree-sitter-src/tree-sitter-java/src", &[]);

    build(
        "tree-sitter-swift",
        "../tree-sitter-src/tree-sitter-swift/src",
        &["scanner.c"],
    );
}
