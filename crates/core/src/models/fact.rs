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

use std::collections::HashMap;

use getset::Getters;
use pyo3::prelude::{pyclass, pymethods};
use serde_derive::{Deserialize, Serialize};

use super::matches::{Point, Range};

/// A fact recorded by a fact rule: a `String -> String` map associated to a range in source code.
/// When a rewrite overlaps the fact's range, the fact is marked `voided`.
/// When a rewrite precedes the fact's range, the fact's byte offsets and row/col points are shifted.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Getters)]
#[pyclass]
pub struct Fact {
  /// The range in source code this fact is associated with
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) range: Range,
  /// The fact data
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) data: HashMap<String, String>,
  /// Whether this fact was voided by a subsequent rewrite that overlapped its range
  #[get = "pub"]
  #[pyo3(get)]
  pub(crate) voided: bool,
}

impl Fact {
  pub(crate) fn new(range: Range, data: HashMap<String, String>) -> Self {
    Fact { range, data, voided: false }
  }

  /// Shift byte offsets and row/col points after a preceding edit.
  ///
  /// Parameters come from the `InputEdit` that was applied before this fact's range:
  /// - `byte_delta`:     new_end_byte − old_end_byte (negative for deletions)
  /// - `old_end_row/col`: row/col of the edit's old end position
  /// - `new_end_row/col`: row/col of the edit's new end position
  ///
  /// Row/col adjustment follows the standard tree-sitter convention:
  /// - Facts on rows *after* the edit's end row shift by `row_delta` rows;
  ///   their column is unchanged.
  /// - Facts whose start/end sits on the *same* row as the edit's end
  ///   also shift by `row_delta` rows and their column shifts by
  ///   `new_end_col − old_end_col`.
  pub(crate) fn shift_range(
    &mut self, byte_delta: isize, old_end_row: usize, old_end_col: usize, new_end_row: usize,
    new_end_col: usize,
  ) {
    self.range.start_byte = (self.range.start_byte as isize + byte_delta) as usize;
    self.range.end_byte = (self.range.end_byte as isize + byte_delta) as usize;

    let row_delta = new_end_row as isize - old_end_row as isize;
    let col_delta = new_end_col as isize - old_end_col as isize;

    self.range.start_point = shift_point(self.range.start_point, row_delta, col_delta, old_end_row);
    self.range.end_point = shift_point(self.range.end_point, row_delta, col_delta, old_end_row);
  }
}

/// Shift a single point after an edit whose old end was at `old_end_row`.
/// - If the point is on a row *after* `old_end_row`: shift the row, keep the column.
/// - If the point is on *exactly* `old_end_row`: shift both row and column.
/// - If the point is before `old_end_row`: should not occur for facts entirely after the edit.
fn shift_point(p: Point, row_delta: isize, col_delta: isize, old_end_row: usize) -> Point {
  if p.row > old_end_row {
    Point { row: (p.row as isize + row_delta) as usize, column: p.column }
  } else {
    // p.row == old_end_row (same row as edit end)
    Point {
      row: (p.row as isize + row_delta) as usize,
      column: (p.column as isize + col_delta) as usize,
    }
  }
}

#[pymethods]
impl Fact {
  fn __repr__(&self) -> String {
    format!("{self:?}")
  }

  fn __str__(&self) -> String {
    self.__repr__()
  }
}
