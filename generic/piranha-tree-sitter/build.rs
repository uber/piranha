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

//! Build script. 
use std::{path::PathBuf, process::{Command, Stdio}};

// Compiles the tree-sitter parser/scanner. 
// If the tree-sitter source code is not found, it clones the repositories from GitHub 
// into the folder `piranha/generic/tree-sitter-src` and then generates (and compiles) the parser.c and/or scanner.c
// Prerequisite: (i) Tree-sitter CLI and (ii) git.
// Installing tree-sitter's CLI: https://github.com/tree-sitter/tree-sitter/blob/master/cli/README.md

fn build(language: &str)  -> std::io::Result<()>  {
    let (ts_src, git_url) = match language {
        "Java" => ("tree-sitter-java", "https://github.com/tree-sitter/tree-sitter-java.git"),
        _ => panic!("Language not supported!")
    };

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{:?}",project_root.as_os_str().to_str());
    let path_to_all_tree_sitter_src = project_root.parent().unwrap().join("tree-sitter-src");
    let path_tree_sitter_src = path_to_all_tree_sitter_src.join(ts_src);
    if !path_tree_sitter_src.exists() {
        let mut clone_repo_cmd = Command::new("git")
            .stdout(Stdio::piped())
            .current_dir(path_to_all_tree_sitter_src)
            .arg("clone")
            .arg(git_url)
            .spawn().unwrap();

        let clone_repo = clone_repo_cmd.wait().unwrap();
        if clone_repo.success() {
            println!("Successfully cloned {ts_src}");
        }else{
            panic!("Could not clone repo!");
        }

        let mut build_repo_cmd = Command::new("tree-sitter")
            .stdout(Stdio::piped())
            .current_dir(&path_tree_sitter_src)
            .arg("generate")
            .spawn().unwrap();
        let build_repo = build_repo_cmd.wait().unwrap();
        if build_repo.success() {
            println!("Successfully generated {ts_src}");
        }else{
            panic!("Could not generate tree-sitter parser/scanner");
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
}
