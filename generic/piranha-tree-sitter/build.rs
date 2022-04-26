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

use std::{path::PathBuf, process::{Command, Stdio}};

// TODO: Add a way to checkout github repository, generate

fn build(language: &str)  -> std::io::Result<()>  {
    let (ts_src, git_url) = match language {
        "Java" => ("tree-sitter-java", "https://github.com/tree-sitter/tree-sitter-java.git"),
        "Swift" => ("tree-sitter-swift","https://github.com/alex-pinkus/tree-sitter-swift.git"),
        _ => panic!("Language not supported!")
    };
    let path_to_all_tree_sitter_src = PathBuf::from(format!("../tree-sitter-src"));
    let path_tree_sitter_src = path_to_all_tree_sitter_src.join(ts_src);
    if !path_tree_sitter_src.exists() {
        let clone_repo = Command::new("git")
            .stdout(Stdio::piped())
            .current_dir(path_to_all_tree_sitter_src)
            .arg("clone")
            .arg(git_url)
            .output()
            .unwrap();
        if clone_repo.status.success() {
            println!("Successfully cloned {ts_src}");
        }else{
            panic!("Could not clone repo!");
        }

        let build_repo = Command::new("tree-sitter")
            .stdout(Stdio::piped())
            .current_dir(&path_tree_sitter_src)
            .arg("generate")
            .output().unwrap();

        if build_repo.status.success() {
            println!("Successfully generated {ts_src}");
        }else{
            panic!("Could not generate tree-sitter parser/scanner, {:?}", build_repo.stderr);
        }   
    }
    let dir = path_tree_sitter_src.join("src");
    let mut build = cc::Build::new();
    build.include(&dir).warnings(false);
    build.file(dir.join("parser.c"));
    if dir.join("scanner.c").exists() {
        build.file(dir.join("scanner.c"));
    }
    build.compile(ts_src);

    Ok(())
}

fn main() {
    let _ = build("Java");
    let _ = build("Swift");
}
