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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A position in a text document represented as zero-based line and column numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
  pub row: usize,
  pub column: usize,
}

impl Point {
  pub fn new(row: usize, column: usize) -> Self {
    Point { row, column }
  }
}

/// A range in a text document represented as byte offsets and line/column positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
  pub start_byte: usize,
  pub end_byte: usize,
  pub start_point: Point,
  pub end_point: Point,
}

impl Range {
  pub fn new(start_byte: usize, end_byte: usize, start_point: Point, end_point: Point) -> Self {
    Range {
      start_byte,
      end_byte,
      start_point,
      end_point,
    }
  }

  /// Create a range that spans from the start of the first range to the end of the second range
  pub fn span_ranges(first: Range, second: Range) -> Range {
    Range {
      start_byte: first.start_byte,
      end_byte: second.end_byte,
      start_point: first.start_point,
      end_point: second.end_point,
    }
  }

  /// Get the length of the range in bytes
  pub fn len(&self) -> usize {
    self.end_byte - self.start_byte
  }

  /// Check if the range is empty
  pub fn is_empty(&self) -> bool {
    self.start_byte == self.end_byte
  }

  /// Check if this range contains another range
  pub fn contains(&self, other: &Range) -> bool {
    self.start_byte <= other.start_byte && other.end_byte <= self.end_byte
  }

  /// Check if this range overlaps with another range
  pub fn overlaps(&self, other: &Range) -> bool {
    self.start_byte < other.end_byte && other.start_byte < self.end_byte
  }
}

/// Represents a captured node with its text content and position information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedNode {
  pub range: Range,
  pub text: String,
}

impl CapturedNode {
  pub fn new(range: Range, text: String) -> Self {
    CapturedNode { range, text }
  }
}

/// A match result containing the matched text, position, and captured variables
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
  /// The main matched text
  pub matched_string: String,
  /// The range of the match in the source code
  pub range: Range,
  /// Captured variables from the pattern (with full range information)
  pub matches: HashMap<String, CapturedNode>,
  /// Optional replacement text for this match
  pub replacement: Option<String>,
}

impl Match {
  pub fn new(matched_string: String, range: Range, matches: HashMap<String, CapturedNode>) -> Self {
    Match {
      matched_string,
      range,
      matches,
      replacement: None,
    }
  }

  pub fn with_replacement(mut self, replacement: String) -> Self {
    self.replacement = Some(replacement);
    self
  }

  /// Get a captured variable by name
  pub fn get_match(&self, name: &str) -> Option<&CapturedNode> {
    self.matches.get(name)
  }

  /// Get the text of a captured variable
  pub fn get_match_text(&self, name: &str) -> Option<&str> {
    self
      .matches
      .get(name)
      .map(|captured| captured.text.as_str())
  }

  /// Get the range of a captured variable
  pub fn get_match_range(&self, name: &str) -> Option<Range> {
    self.matches.get(name).map(|captured| captured.range)
  }

  /// Get all match names
  pub fn match_names(&self) -> Vec<&String> {
    self.matches.keys().collect()
  }
}
