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

use crate::models::matches::Range;
use tree_sitter::{Node, TreeCursor};

/// Cursor navigation utilities for tree traversal during pattern matching.
pub struct CursorNavigator;

impl CursorNavigator {
  /// Advances the cursor to the next sibling, or if no sibling exists,
  /// moves up to the parent and tries to find the next sibling at that level.
  pub fn find_next_sibling_or_ancestor_sibling(cursor: &mut TreeCursor) -> bool {
    while !cursor.goto_next_sibling() {
      if !cursor.goto_parent() {
        return false;
      }
    }
    true
  }

  /// Skips over comment nodes by advancing the cursor to the next non-comment sibling.
  /// If the current node is not a comment, the cursor position remains unchanged.
  pub fn skip_comment_nodes(cursor: &mut TreeCursor) {
    let mut node = cursor.node();
    while node.kind().contains("comment") && cursor.goto_next_sibling() {
      node = cursor.node();
    }
  }

  /// Finds the index of a target node among its parent's children.
  pub fn find_child_index(target_node: &Node, parent_node: &Node) -> Option<usize> {
    parent_node
      .children(&mut parent_node.walk())
      .enumerate()
      .find(|&(_i, child)| child == *target_node)
      .map(|(i, _child)| i)
  }

  /// Creates a range that spans from the start of the first node to the end of the second node.
  pub fn span_node_ranges(first_node: &Node, last_node: &Node) -> Range {
    Range::span_ranges(first_node.range(), last_node.range())
  }

  /// Extracts text from source code within the specified byte range.
  pub fn get_text_from_range(start_byte: usize, end_byte: usize, source_code: &[u8]) -> String {
    let text_slice = &source_code[start_byte..end_byte];
    String::from_utf8_lossy(text_slice).to_string()
  }
}

#[cfg(test)]
mod tests {

  // Tests will be added as we migrate functionality
}
