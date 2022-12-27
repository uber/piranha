use std::process::Command;
// Set up the development environment
// Creates a `venv` with pre-commit / maturin
fn main() {
  // Create python virtual environment
  _ = Command::new("python3")
    .arg("-m")
    .args(["venv", ".env"])
    .spawn()
    .expect("Could not create virtual environment");

  // Install pre-commit / maturin binary
  _ = Command::new("pip3")
    .args(["install", "pre-commit", "maturin"])
    .spawn()
    .expect("Could not install pre-commit (pip dependency)");

  _ = // Install taplo-cli
  _ = Command::new("cargo")
    .args(["install", "taplo-cli", "--locked"])
    .spawn()
    .expect("Could not install taplo (toml formatter)");
}
