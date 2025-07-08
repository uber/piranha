# Concrete Syntax

A Rust library for structural pattern matching on code using concrete syntax patterns.

## Building

### WebAssembly (WASM)

To build for web browsers:

```bash
# Build for web target
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm
wasm-pack build --target web --out-dir pkg-web --no-default-features --features wasm
```

### Native (Piranha)

To build for native use within Piranha:

```bash
cargo build --release
```

## Tree-Sitter Integration

The WASM bindings require tree-sitter node wrappers to interface with the Rust code. The wrappers implement:

- `TreeSitterNodeWrapper` - Implements the `WasmSyntaxNode` interface  
- `TreeSitterCursorWrapper` - Implements the `WasmSyntaxCursor` interface

These wrappers bridge tree-sitter's JavaScript API with the Rust WASM module's expected interfaces.

## Tree-Sitter Playground Integration

The concrete syntax functionality is integrated into the tree-sitter playground at https://uber.github.io/piranha/tree-sitter-playground/.
