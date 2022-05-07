/*
Copyright (c) 2022 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Build script.
use std::{
  path::PathBuf,
  process::{Command, Stdio},
};

// Compiles the tree-sitter parser/scanner.
// If the tree-sitter source code is not found, it clones the repositories from GitHub
// into the folder `piranha/generic/tree-sitter-src` and then generates (and compiles) the parser.c and/or scanner.c
// Prerequisite: (i) Tree-sitter CLI and (ii) git.
// Installing tree-sitter's CLI: https://github.com/tree-sitter/tree-sitter/blob/master/cli/README.md

fn build(language: &str) -> Result<&str, &str> {
  let (ts_src, git_url) = match language {
    "java" => (
      "tree-sitter-java",
      "https://github.com/tree-sitter/tree-sitter-java.git",
    ),
    _ => panic!("Language not supported!"),
  };

  let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let path_to_all_tree_sitter_src = project_root.parent().unwrap().join("tree-sitter-src");
  let path_to_tree_sitter_language = path_to_all_tree_sitter_src.join(ts_src);
  let path_to_tree_sitter_language_src = path_to_tree_sitter_language.join("src");
  let path_to_tree_sitter_language_parser = path_to_tree_sitter_language_src.join("parser.c");
  let path_to_tree_sitter_language_scanner = path_to_tree_sitter_language.join("scanner.c");

  println!("{:?}", project_root.as_os_str().to_str());
  if !path_to_tree_sitter_language.exists() {
    let mut clone_repo_cmd = Command::new("git")
      .stdout(Stdio::piped())
      .current_dir(path_to_all_tree_sitter_src)
      .arg("clone")
      .arg(git_url)
      .spawn()
      .unwrap();

    let clone_repo = clone_repo_cmd.wait().unwrap();
    if !clone_repo.success() {
      return Err("Could not clone tree-sitter language repository - {git_url}");
    }

    let mut build_repo_cmd = Command::new("tree-sitter")
      .stdout(Stdio::piped())
      .current_dir(&path_to_tree_sitter_language)
      .arg("generate")
      .spawn()
      .unwrap();

    let build_repo = build_repo_cmd.wait().unwrap();
    if !build_repo.success() {
      return Err("Could not generate tree-sitter parser/scanner");
    }
  }

  let mut build = cc::Build::new();
  build
    .include(&path_to_tree_sitter_language_src)
    .warnings(false);
  build.file(path_to_tree_sitter_language_parser);
  if path_to_tree_sitter_language_scanner.exists() {
    build.file(path_to_tree_sitter_language_scanner);
  }
  build.compile(ts_src);

  Ok("Successfully built {ts_src}")
}

fn main() {
  let _ = build("java");
}
