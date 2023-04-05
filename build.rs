/*
 Copyright (c) 2023 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use std::process::Command;

/// Set up the development environment
/// Creates a `venv` with pre-commit / maturin
fn main() {
  // Create python virtual environment
  _ = Command::new("python3")
    .arg("-m")
    .args(["venv", ".env"])
    .spawn()
    .expect("Could not create virtual environment");

  // Install pre-commit and maturin
  _ = Command::new("pip3")
    .args(["install", "pre-commit", "maturin"])
    .spawn()
    .expect("Could not install pre-commit (pip dependency)");

  // Add pre-commit hook
  _ = Command::new("sh")
    .arg("-c")
    .arg("pre-commit install")
    .spawn()
    .expect("Install pre-commit hook");

  // Install taplo-cli
  _ = Command::new("cargo")
    .args(["install", "taplo-cli", "--locked"])
    .spawn()
    .expect("Could not install taplo (toml formatter)");
}
