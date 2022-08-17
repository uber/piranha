/*
Copyright (c) 2022 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

use std::collections::HashMap;

use serde::{Serialize, Serializer};
use tree_sitter::Range;

#[derive(serde_derive::Serialize, Debug, Clone)]
pub(crate) struct Match {
  // Range of the entire AST node captured by the match
  #[serde(serialize_with = "ser_range")]
  range: Range,
  // The mapping between tags and string representation of the AST captured.
  matches: HashMap<String, String>,
}

impl Match {
  pub(crate) fn new(range: Range, matches: HashMap<String, String>) -> Self {
    Self { range, matches }
  }

  /// Get the edit's replacement range.
  pub(crate) fn range(&self) -> Range {
    self.range
  }

  pub(crate) fn matches(&self) -> &HashMap<String, String> {
    &self.matches
  }
}

/// This method serializes the `tree_sitter::Range` struct. Originally, it does not derive serde::Serialize
/// So in this method, we cast `Range` to a local type `LocalRange` and serialize this casted object.
/// Note `LocalRange` derives serialize.
fn ser_range<S: Serializer>(range: &Range, serializer: S) -> Result<S::Ok, S::Error> {
  let local_range = LocalRange {
    start_byte: range.start_byte,
    end_byte: range.end_byte,
    start_point: LocalPoint {
      row: range.start_point.row,
      column: range.start_point.column,
    },
    end_point: LocalPoint {
      row: range.end_point.row,
      column: range.end_point.column,
    },
  };

  // Instead of serializing Vec<ExternalCrateColor>, we serialize Vec<LocalColor>.
  local_range.serialize(serializer)
}

/// A range of positions in a multi-line text document, both in terms of bytes and of
/// rows and columns.
/// Note `LocalRange` derives serialize.
#[derive(serde_derive::Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct LocalRange {
  start_byte: usize,
  end_byte: usize,
  start_point: LocalPoint,
  end_point: LocalPoint,
}

/// A range of positions in a multi-line text document, both in terms of bytes and of
/// rows and columns.
#[derive(serde_derive::Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct LocalPoint {
  row: usize,
  column: usize,
}
