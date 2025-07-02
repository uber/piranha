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
  fn walk(&self) -> impl SyntaxCursor<Node = Self>;
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
#[derive(Clone, PartialEq)]
pub struct NativeNode<'a> {
  pub inner: tree_sitter::Node<'a>,
}

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

  fn walk(&self) -> impl SyntaxCursor<Node = Self> {
    NativeCursor {
      inner: self.inner.walk(),
    }
  }
}

#[derive(Clone)]
pub struct NativeCursor<'a> {
  pub inner: tree_sitter::TreeCursor<'a>,
}

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

// Re-export tree-sitter types for backward compatibility
pub use tree_sitter::{Node, TreeCursor};

/// Adapter functions for working with tree-sitter types directly
/// These provide a bridge between the trait-based and direct tree-sitter APIs
pub struct TreeSitterAdapter;

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
