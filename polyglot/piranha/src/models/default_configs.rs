use std::collections::HashMap;

use super::language::PiranhaLanguage;


pub const JAVA: &str = "java";
pub const KOTLIN: &str = "kt";
pub const GO: &str = "go";
pub const PYTHON: &str = "py";
pub const SWIFT: &str = "swift";
pub const STRINGS: &str = "strings";
pub const TYPESCRIPT: &str = "ts"; 
pub const TSX: &str = "tsx";

pub fn default_number_of_ancestors_in_parent_scope() -> u8 {
  4
}

pub fn default_languages() -> Vec<String> {
  vec![default_language()]
}

pub fn default_language() -> String {
  JAVA.to_string()
}

pub fn default_substitutions() -> Vec<Vec<String>> {
  vec![]
}

pub fn default_delete_file_if_empty() -> bool {
  true
}

pub fn default_cleanup_comments_buffer() -> usize {
  2
}

pub fn default_cleanup_comments() -> bool {
  false
}

pub fn default_global_tag_prefix() -> String {
  "GLOBAL_TAG.".to_string()
}

pub fn default_dry_run() -> bool {
  false
}

pub fn default_name_of_piranha_argument_toml() -> String {
  "piranha_arguments.toml".to_string()
}

pub fn default_path_to_codebase() -> String {
  String::new()
}

pub fn default_input_substitutions() -> HashMap<String, String> {
  HashMap::new()
}

pub fn default_path_to_configurations() -> String {
  String::new()
}

pub fn default_path_to_output_summaries() -> Option<String> {
  None
}

pub fn default_piranha_language() -> PiranhaLanguage {
  PiranhaLanguage::default()
}

pub fn default_delete_consecutive_new_lines() -> bool {
  false
}
