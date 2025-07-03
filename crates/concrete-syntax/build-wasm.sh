#!/bin/bash

# Build WASM package for concrete-syntax
echo "Building WASM package..."

# Build the WASM binary
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm

# Generate JavaScript bindings
wasm-bindgen --out-dir pkg --target nodejs ../../target/wasm32-unknown-unknown/debug/concrete_syntax.wasm

echo "✅ WASM package built successfully in pkg/ directory"
echo "📦 Files generated:"
ls -la pkg/ 