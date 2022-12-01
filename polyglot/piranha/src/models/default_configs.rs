pub fn default_number_of_ancestors_in_parent_scope() -> u8 {
  4
}

pub fn default_languages() -> Vec<String> {
  vec![default_language()]
}

pub fn default_language() -> String {
    "java".to_string()
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
