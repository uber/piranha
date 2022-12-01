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
use serde_derive::Deserialize;
use std::collections::HashMap;
use getset::Getters;

use super::default_configs::{default_languages, default_substitutions, default_global_tag_prefix, default_cleanup_comments_buffer, default_number_of_ancestors_in_parent_scope, default_delete_file_if_empty};
/// Captures the Piranha arguments by from the file at `path_to_feature_flag_rules`.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Getters, Default)]
pub(crate) struct PiranhaConfiguration {
  #[serde(default = "default_languages")]
  language: Vec<String>,

  #[serde(default = "default_substitutions")]
  substitutions: Vec<Vec<String>>,

  #[serde(default = "default_delete_file_if_empty")]
  #[get = "pub"]
  delete_file_if_empty: bool,
  
  #[serde(default)]
  #[get = "pub"]
  delete_consecutive_new_lines: bool,
  
  #[serde(default = "default_global_tag_prefix")]
  #[get = "pub"]
  global_tag_prefix: String,

  #[serde(default = "default_cleanup_comments_buffer")]
  #[get = "pub"]
  cleanup_comments_buffer: usize,

  #[serde(default)]
  #[get = "pub"]
  cleanup_comments: bool,
  
  #[serde(default = "default_number_of_ancestors_in_parent_scope")]
  #[get = "pub"]
  number_of_ancestors_in_parent_scope: u8,
}


impl PiranhaConfiguration {
  pub(crate) fn substitutions(&self) -> HashMap<String, String> {
    self
      .substitutions
      .iter()
      .map(|x| (x[0].clone(), x[1].clone()))
      .collect()
  }

  pub fn language(&self) -> String {
    self.language[0].clone()
  }

}
