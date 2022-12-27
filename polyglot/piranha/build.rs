use std::process::Command;

// Example custom build script.
fn main() {
  // Create python virtual environment
  _ = Command::new("python3")
    .arg("-m")
    .args(["venv", ".env"])
    .spawn()
    .expect("Could not create virtual environment");

  // Install pre-commit binary
  _ = Command::new("pip3")
    .args(["install", "pre-commit", "maturin"])
    .spawn()
    .expect("Could not install pre-commit (pip dependency)");
}
