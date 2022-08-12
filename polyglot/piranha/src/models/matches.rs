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

use tree_sitter::{Range};

#[derive(Debug, Clone)]
pub(crate) struct Match {
  // Range of the entire AST node captured by the match
  range: Range,
  // The mapping between tags and string representation of the AST captured.
  matches: HashMap<String, String>,
}

impl Match {
  pub(crate) fn new(
    range: Range, matches: HashMap<String, String>,
  ) -> Self {
    Self {
      range,
      matches,
    }
  }

  /// Get the edit's replacement range.
  pub(crate) fn range(&self) -> Range {
    self.range
  }

  pub(crate) fn matches(&self) -> &HashMap<String, String> {
    &self.matches
  }

}
