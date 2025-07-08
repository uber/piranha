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

//! Concrete Syntax Pattern Matching Library
//!
//! This library provides pattern matching capabilities for concrete syntax trees,
//! supporting both native Tree-sitter and WASM compilation modes.

// Main modules
pub mod models;

// Re-export from the module structure for convenience
pub use models::concrete_syntax::interpreter::get_all_matches_for_concrete_syntax;
pub use models::concrete_syntax::*;
pub use models::matches::{CapturedNode, Match, Point, Range};

// Feature-gated exports
#[cfg(feature = "native")]
pub use models::concrete_syntax::tree_sitter_adapter::{
  NativeCursor, NativeNode, TreeSitterAdapter,
};

#[cfg(feature = "wasm")]
pub use wasm::*;

// WASM interface when compiled for WASM
#[cfg(feature = "wasm")]
pub mod wasm {
  use wasm_bindgen::prelude::*;

  use crate::models::concrete_syntax::parser::ConcreteSyntax;

  /// WASM-compatible interface for the concrete syntax matcher
  #[wasm_bindgen]
  pub struct ConcreteSyntaxMatcher {
    pattern: ConcreteSyntax,
  }

  /// WASM-compatible node interface that JS implementations must provide
  #[wasm_bindgen]
  extern "C" {
    #[wasm_bindgen(typescript_type = "WasmSyntaxNode")]
    pub type WasmSyntaxNode;

    #[wasm_bindgen(method, getter, js_name = "startByte")]
    pub fn start_byte(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, getter, js_name = "endByte")]
    pub fn end_byte(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, getter, js_name = "startRow")]
    pub fn start_row(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, getter, js_name = "startColumn")]
    pub fn start_column(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, getter, js_name = "endRow")]
    pub fn end_row(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, getter, js_name = "endColumn")]
    pub fn end_column(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, getter)]
    pub fn kind(this: &WasmSyntaxNode) -> String;

    #[wasm_bindgen(method, getter, js_name = "childCount")]
    pub fn child_count(this: &WasmSyntaxNode) -> usize;

    #[wasm_bindgen(method, js_name = "child")]
    pub fn child(this: &WasmSyntaxNode, index: usize) -> Option<WasmSyntaxNode>;

    #[wasm_bindgen(method)]
    pub fn children(this: &WasmSyntaxNode) -> js_sys::Array;

    #[wasm_bindgen(method, js_name = "utf8Text")]
    pub fn utf8_text(this: &WasmSyntaxNode, source: &[u8]) -> String;

    #[wasm_bindgen(method)]
    pub fn walk(this: &WasmSyntaxNode) -> WasmSyntaxCursor;

    #[wasm_bindgen(method)]
    pub fn clone(this: &WasmSyntaxNode) -> WasmSyntaxNode;
  }

  /// WASM-compatible cursor interface that JS implementations must provide
  #[wasm_bindgen]
  extern "C" {
    #[wasm_bindgen(typescript_type = "WasmSyntaxCursor")]
    pub type WasmSyntaxCursor;

    #[wasm_bindgen(method)]
    pub fn node(this: &WasmSyntaxCursor) -> WasmSyntaxNode;

    #[wasm_bindgen(method, js_name = "gotoFirstChild")]
    pub fn goto_first_child(this: &WasmSyntaxCursor) -> bool;

    #[wasm_bindgen(method, js_name = "gotoNextSibling")]
    pub fn goto_next_sibling(this: &WasmSyntaxCursor) -> bool;

    #[wasm_bindgen(method, js_name = "gotoParent")]
    pub fn goto_parent(this: &WasmSyntaxCursor) -> bool;

    #[wasm_bindgen(method)]
    pub fn clone(this: &WasmSyntaxCursor) -> WasmSyntaxCursor;
  }

  #[wasm_bindgen]
  impl ConcreteSyntaxMatcher {
    #[wasm_bindgen(constructor)]
    pub fn new(pattern_str: &str) -> Result<ConcreteSyntaxMatcher, JsValue> {
      let pattern = ConcreteSyntax::parse(pattern_str).map_err(|e| JsValue::from_str(&e))?;

      Ok(ConcreteSyntaxMatcher { pattern })
    }

    #[wasm_bindgen(js_name = "findMatches")]
    pub fn find_matches(
      &self, node: &WasmSyntaxNode, source_code: &[u8], recursive: bool,
    ) -> Result<JsValue, JsValue> {
      use crate::models::concrete_syntax::resolver::resolve_concrete_syntax;

      // Convert WASM node to our internal representation
      let wrapped_node =
        crate::models::concrete_syntax::tree_sitter_adapter::WasmNodeWrapper::new(node.clone());
      let resolved = resolve_concrete_syntax(&self.pattern);

      // Call the actual matching function
      let matches =
        crate::models::concrete_syntax::interpreter::get_all_matches_for_concrete_syntax(
          &wrapped_node,
          source_code,
          &resolved,
          recursive,
          None,
        );

      serde_wasm_bindgen::to_value(&matches)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
  }

  /// Standalone function for direct pattern matching without creating a ConcreteSyntaxMatcher instance
  #[wasm_bindgen(js_name = "getAllMatches")]
  pub fn get_all_matches_wasm(
    pattern_str: &str, node: &WasmSyntaxNode, source_code: &[u8], recursive: bool,
    replace_node: Option<String>,
  ) -> Result<JsValue, JsValue> {
    use crate::models::concrete_syntax::resolver::resolve_concrete_syntax;

    // Parse the pattern
    let pattern = ConcreteSyntax::parse(pattern_str)
      .map_err(|e| JsValue::from_str(&format!("Pattern parse error: {}", e)))?;

    // Convert WASM node to our internal representation
    let wrapped_node =
      crate::models::concrete_syntax::tree_sitter_adapter::WasmNodeWrapper::new(node.clone());
    let resolved = resolve_concrete_syntax(&pattern);

    // Call the actual matching function
    let matches = crate::models::concrete_syntax::interpreter::get_all_matches_for_concrete_syntax(
      &wrapped_node,
      source_code,
      &resolved,
      recursive,
      replace_node,
    );

    serde_wasm_bindgen::to_value(&matches)
      .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
  }

  /// Parse a concrete syntax pattern and return it as a serialized object for inspection
  #[wasm_bindgen(js_name = "parsePattern")]
  pub fn parse_pattern_wasm(pattern_str: &str) -> Result<JsValue, JsValue> {
    let pattern = ConcreteSyntax::parse(pattern_str)
      .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    serde_wasm_bindgen::to_value(&pattern)
      .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
  }

  /// TypeScript interface definition for implementers
  pub const TYPESCRIPT_INTERFACES: &str = r#"
    export interface Point {
        readonly row: number;
        readonly column: number;
    }

    export interface Range {
        readonly startByte: number;
        readonly endByte: number;
        readonly startPoint: Point;
        readonly endPoint: Point;
    }

    export interface CapturedNode {
        readonly range: Range;
        readonly text: string;
    }

    export interface Match {
        readonly matchedString: string;
        readonly range: Range;
        readonly matches: Record<string, CapturedNode>;
        readonly replacement?: string;
    }

    export interface WasmSyntaxNode {
        readonly startByte: number;
        readonly endByte: number;
        readonly startRow: number;
        readonly startColumn: number;
        readonly endRow: number;
        readonly endColumn: number;
        readonly kind: string;
        readonly childCount: number;
        
        child(index: number): WasmSyntaxNode | null;
        children(): WasmSyntaxNode[];
        utf8Text(source: Uint8Array): string;
        walk(): WasmSyntaxCursor;
        clone(): WasmSyntaxNode;
    }
    
    export interface WasmSyntaxCursor {
        node(): WasmSyntaxNode;
        gotoFirstChild(): boolean;
        gotoNextSibling(): boolean;
        gotoParent(): boolean;
        clone(): WasmSyntaxCursor;
    }
    
    // Exported functions
    export function getAllMatches(
        pattern: string,
        node: WasmSyntaxNode,
        sourceCode: Uint8Array,
        recursive: boolean,
        replaceNode?: string
    ): Match[];
    
    export function parsePattern(pattern: string): any;
    
    export class ConcreteSyntaxMatcher {
        constructor(pattern: string);
        findMatches(node: WasmSyntaxNode, sourceCode: Uint8Array, recursive: boolean): Match[];
    }
    "#;
}
