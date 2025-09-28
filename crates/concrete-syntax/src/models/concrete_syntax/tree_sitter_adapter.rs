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

//! Trait abstraction for tree-sitter dependencies
//! This allows the concrete syntax module to work with different tree implementations

use crate::models::matches::Range;

/// Minimal node interface - only the methods actually used  
pub trait SyntaxNode: PartialEq + Clone {
  fn range(&self) -> Range;
  fn kind(&self) -> &str;
  fn child_count(&self) -> usize;
  fn child(&self, index: usize) -> Option<Self>;
  fn children(&self) -> Vec<Self>;
  fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error>;
}

/// Minimal cursor interface - only the methods actually used
pub trait SyntaxCursor: Clone {
  type Node: SyntaxNode;

  fn node(&self) -> Self::Node;
  fn goto_first_child(&mut self) -> bool;
  fn goto_next_sibling(&mut self) -> bool;
  fn goto_parent(&mut self) -> bool;
}

// Native tree-sitter implementation
#[cfg(feature = "native")]
#[derive(Clone, PartialEq)]
pub struct NativeNode<'a> {
  pub inner: tree_sitter::Node<'a>,
}

#[cfg(feature = "native")]
impl<'a> SyntaxNode for NativeNode<'a> {
  fn range(&self) -> Range {
    let ts_range = self.inner.range();
    Range {
      start_byte: ts_range.start_byte,
      end_byte: ts_range.end_byte,
      start_point: crate::models::matches::Point {
        row: ts_range.start_point.row,
        column: ts_range.start_point.column,
      },
      end_point: crate::models::matches::Point {
        row: ts_range.end_point.row,
        column: ts_range.end_point.column,
      },
    }
  }

  fn kind(&self) -> &str {
    self.inner.kind()
  }

  fn child_count(&self) -> usize {
    self.inner.child_count()
  }

  fn child(&self, index: usize) -> Option<Self> {
    self
      .inner
      .child(index)
      .map(|node| NativeNode { inner: node })
  }

  fn children(&self) -> Vec<Self> {
    let mut cursor = self.inner.walk();
    self
      .inner
      .children(&mut cursor)
      .map(|node| NativeNode { inner: node })
      .collect()
  }

  fn utf8_text<'b>(&self, source: &'b [u8]) -> Result<&'b str, std::str::Utf8Error> {
    self.inner.utf8_text(source)
  }
}

#[cfg(feature = "native")]
impl<'a> NativeNode<'a> {
  pub fn walk(&self) -> NativeCursor<'_> {
    NativeCursor {
      inner: self.inner.walk(),
    }
  }
}

#[cfg(feature = "native")]
#[derive(Clone)]
pub struct NativeCursor<'a> {
  pub inner: tree_sitter::TreeCursor<'a>,
}

#[cfg(feature = "native")]
impl<'a> SyntaxCursor for NativeCursor<'a> {
  type Node = NativeNode<'a>;

  fn node(&self) -> Self::Node {
    NativeNode {
      inner: self.inner.node(),
    }
  }

  fn goto_first_child(&mut self) -> bool {
    self.inner.goto_first_child()
  }

  fn goto_next_sibling(&mut self) -> bool {
    self.inner.goto_next_sibling()
  }

  fn goto_parent(&mut self) -> bool {
    self.inner.goto_parent()
  }
}

// Type aliases for backward compatibility
#[cfg(feature = "native")]
pub type Node<'a> = NativeNode<'a>;
#[cfg(feature = "native")]
pub type TreeCursor<'a> = NativeCursor<'a>;

// Re-export raw tree-sitter types with different names to avoid confusion
#[cfg(feature = "native")]
pub use tree_sitter::{Node as RawNode, TreeCursor as RawTreeCursor};

// WASM implementation - delegates to JavaScript implementations
#[cfg(feature = "wasm")]
use wasm_bindgen::JsCast;

// Direct reference to WASM types when needed

// Manual Clone implementations for WASM types (since extern types can't derive Clone)
#[cfg(feature = "wasm")]
impl Clone for crate::wasm::WasmSyntaxNode {
  fn clone(&self) -> Self {
    // Delegate to JavaScript's clone method implementation
    self.clone()
  }
}

#[cfg(feature = "wasm")]
impl Clone for crate::wasm::WasmSyntaxCursor {
  fn clone(&self) -> Self {
    // Delegate to JavaScript's clone method implementation
    self.clone()
  }
}

// Wrapper around JavaScript-implemented node
#[cfg(feature = "wasm")]
#[derive(Clone)]
pub struct WasmNodeWrapper {
  pub js_node: crate::wasm::WasmSyntaxNode,
  // Cache strings to work around lifetime issues
  pub cached_kind: String,
}

#[cfg(feature = "wasm")]
impl PartialEq for WasmNodeWrapper {
  fn eq(&self, other: &Self) -> bool {
    // For simplicity, compare ranges
    self.range() == other.range()
  }
}

#[cfg(feature = "wasm")]
impl SyntaxNode for WasmNodeWrapper {
  fn range(&self) -> Range {
    Range {
      start_byte: self.js_node.start_byte(),
      end_byte: self.js_node.end_byte(),
      start_point: crate::models::matches::Point {
        row: self.js_node.start_row(),
        column: self.js_node.start_column(),
      },
      end_point: crate::models::matches::Point {
        row: self.js_node.end_row(),
        column: self.js_node.end_column(),
      },
    }
  }

  fn kind(&self) -> &str {
    &self.cached_kind
  }

  fn child_count(&self) -> usize {
    self.js_node.child_count()
  }

  fn child(&self, index: usize) -> Option<Self> {
    self.js_node.child(index).map(|js_node| {
      let cached_kind = js_node.kind();
      WasmNodeWrapper {
        js_node,
        cached_kind,
      }
    })
  }

  fn children(&self) -> Vec<Self> {
    let js_children = self.js_node.children();
    let mut children = Vec::new();

    for i in 0..js_children.length() {
      let js_value = js_children.get(i);
      // Instead of trying to cast to WasmSyntaxNode, use the raw JsValue
      // and assume it implements the WasmSyntaxNode interface
      let js_node = js_value.unchecked_into::<crate::wasm::WasmSyntaxNode>();
      let cached_kind = js_node.kind();
      children.push(WasmNodeWrapper {
        js_node,
        cached_kind,
      });
    }

    children
  }

  fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
    // For WASM, we'll assume the text extraction is handled by JS
    // and we just work with the source slice based on the range
    let range = self.range();
    std::str::from_utf8(&source[range.start_byte..range.end_byte])
  }
}

#[cfg(feature = "wasm")]
impl WasmNodeWrapper {
  pub fn new(js_node: crate::wasm::WasmSyntaxNode) -> Self {
    let cached_kind = js_node.kind();
    WasmNodeWrapper {
      js_node,
      cached_kind,
    }
  }

  pub fn walk(&self) -> WasmCursorWrapper {
    WasmCursorWrapper {
      js_cursor: self.js_node.walk(),
    }
  }
}

// Wrapper around JavaScript-implemented cursor
#[cfg(feature = "wasm")]
#[derive(Clone)]
pub struct WasmCursorWrapper {
  pub js_cursor: crate::wasm::WasmSyntaxCursor,
}

#[cfg(feature = "wasm")]
impl SyntaxCursor for WasmCursorWrapper {
  type Node = WasmNodeWrapper;

  fn node(&self) -> Self::Node {
    WasmNodeWrapper::new(self.js_cursor.node())
  }

  fn goto_first_child(&mut self) -> bool {
    // Note: WASM binding limitation - we have to use & instead of &mut
    // The JS implementation should handle internal mutation
    self.js_cursor.goto_first_child()
  }

  fn goto_next_sibling(&mut self) -> bool {
    self.js_cursor.goto_next_sibling()
  }

  fn goto_parent(&mut self) -> bool {
    self.js_cursor.goto_parent()
  }
}

// WASM type aliases
#[cfg(feature = "wasm")]
pub type Node<'a> = WasmNodeWrapper;
#[cfg(feature = "wasm")]
pub type TreeCursor<'a> = WasmCursorWrapper;

/// Adapter functions for working with tree-sitter types directly
/// These provide a bridge between the trait-based and direct tree-sitter APIs
#[cfg(feature = "native")]
pub struct TreeSitterAdapter;

#[cfg(feature = "native")]
impl TreeSitterAdapter {
  /// Wrap a tree-sitter node in our trait wrapper
  pub fn wrap_node<'a>(node: tree_sitter::Node<'a>) -> NativeNode<'a> {
    NativeNode { inner: node }
  }

  /// Wrap a tree-sitter cursor in our trait wrapper
  pub fn wrap_cursor<'a>(cursor: tree_sitter::TreeCursor<'a>) -> NativeCursor<'a> {
    NativeCursor { inner: cursor }
  }

  /// Extract the tree-sitter node from our wrapper
  pub fn unwrap_node<'a>(node: &NativeNode<'a>) -> tree_sitter::Node<'a> {
    node.inner
  }

  /// Extract the tree-sitter cursor from our wrapper  
  pub fn unwrap_cursor<'a>(cursor: &'a NativeCursor<'a>) -> &'a tree_sitter::TreeCursor<'a> {
    &cursor.inner
  }
}
