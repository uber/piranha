---
id: install
title: Installation
sidebar_label: Installation
---

To install Polyglot Piranha, you can use it as a Python library or as a command-line tool.

### Python API

To install the Python API, run the following command:

```bash
pip install polyglot-piranha
```

### Command-line Interface

To install the command-line interface, follow these steps:

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone the repository: `git clone https://github.com/uber/piranha.git`
3. Navigate to the cloned directory: `cd piranha`
4. Build the project: `cargo build --release` (or `cargo build --release --no-default-features` for macOS)
5. The binary will be generated under `target/release`