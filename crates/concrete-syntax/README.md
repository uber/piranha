# Concrete Syntax

A Rust library for structural pattern matching on code using concrete syntax patterns.

## Building

### WebAssembly (WASM)

To build for WebAssembly and generate Node.js bindings:

```bash
# Build for WASM target
cargo build --target wasm32-unknown-unknown --features wasm --no-default-features

# Generate Node.js bindings
wasm-bindgen --out-dir pkg --target nodejs target/wasm32-unknown-unknown/debug/concrete_syntax.wasm
```

### Native (Piranha)

To build for native use within Piranha:

```bash
cargo build --release
```

## Tree-Sitter Integration

The WASM bindings require tree-sitter node wrappers to interface with the Rust code. See [`index.js`](./index.js) for the complete implementation of:

- `TreeSitterNodeWrapper` - Implements the `WasmSyntaxNode` interface
- `TreeSitterCursorWrapper` - Implements the `WasmSyntaxCursor` interface

These wrappers bridge tree-sitter's JavaScript API with the Rust WASM module's expected interfaces.

## Usage

After building the WASM bindings, you can use the pattern matching functionality in Node.js:

```javascript
const { getAllMatches, parsePattern } = require('./pkg/concrete_syntax.js');

// Parse your code with tree-sitter
const tree = parser.parse(sourceCode);
const wrappedRootNode = new TreeSitterNodeWrapper(tree.rootNode);

// Apply pattern matching
const matches = getAllMatches(pattern, wrappedRootNode, sourceBytes, true, null);
```

See [`index.js`](./index.js) for a complete working example with Python code analysis. 