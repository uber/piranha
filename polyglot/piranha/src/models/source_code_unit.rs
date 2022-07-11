use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use regex::Regex;
use tree_sitter::{InputEdit, Node, Parser, Range, Tree};
use tree_sitter_traversal::{traverse, Order};

use crate::utilities::{eq_without_whitespace, tree_sitter_utilities::get_tree_sitter_edit};

use super::{edit::Edit, piranha_arguments::PiranhaArguments};

// Maintains the updated source code content and AST of the file
#[derive(Clone)]
pub struct SourceCodeUnit {
  // The tree representing the file
  ast: Tree,
  // The content of a file
  code: String,
  // The tag substitution cache.
  // This map is looked up to instantiate new rules.
  substitutions: HashMap<String, String>,
  // The path to the source code.
  path: PathBuf,
}

impl SourceCodeUnit {
  pub(crate) fn new(
    parser: &mut Parser, code: String, substitutions: &HashMap<String, String>, path: &Path,
  ) -> Self {
    let ast = parser.parse(&code, None).expect("Could not parse code");
    Self {
      ast,
      code,
      substitutions: substitutions.clone(),
      path: path.to_path_buf(),
    }
  }

  pub fn root_node(&self) -> Node<'_> {
    self.ast.root_node()
  }

  /// Writes the current contents of `code` to the file system.
  /// Based on the user's specifications, this function will delete a file if empty
  /// and replace three consecutive newline characters with two.
  pub(crate) fn persist(&self, piranha_arguments: &PiranhaArguments) {
    if self.code.as_str().is_empty() {
      if piranha_arguments.delete_file_if_empty() {
        _ = fs::remove_file(&self.path).expect("Unable to Delete file");
      }
    } else {
      let content = if piranha_arguments.delete_consecutive_new_lines() {
        let regex = Regex::new(r"\n\s*\n(\s*\n)").unwrap();
        regex.replace_all(&self.code(), "\n${1}").to_string()
      } else {
        self.code()
      };
      fs::write(&self.path, content).expect("Unable to Write file");
    }
  }

  pub(crate) fn apply_edit(&mut self, edit: &Edit, parser: &mut Parser) -> InputEdit {
    // Get the tree_sitter's input edit representation
    self._apply_edit(edit.replacement_range(), edit.replacement_string(), parser)
  }

  /// Applies an edit to the source code unit
  /// # Arguments
  /// * `replace_range` - the range of code to be replaced
  /// * `replacement_str` - the replacement string
  /// * `parser`
  ///
  /// # Returns
  /// The `edit:InputEdit` performed.
  ///
  /// Note - Causes side effect. - Updates `self.ast` and `self.code`
  pub(crate) fn _apply_edit(
    &mut self, range: Range, replacement_string: &str, parser: &mut Parser,
  ) -> InputEdit {
    // Get the tree_sitter's input edit representation
    let (new_source_code, ts_edit) =
      get_tree_sitter_edit(self.code.clone(), range, replacement_string);
    // Apply edit to the tree
    self.ast.edit(&ts_edit);
    // Create a new updated tree from the previous tree
    let new_tree = parser
      .parse(&new_source_code, Some(&self.ast))
      .expect("Could not generate new tree!");
    self.ast = new_tree;
    self.code = new_source_code;
    // Handle errors, like removing extra comma.
    self.remove_additional_comma_from_sequence_list(parser);
    if self.ast.root_node().has_error() {
      panic!(
        "Produced syntactically incorrect source code {}",
        self.code()
      );
    }
    ts_edit
  }

  /// Sometimes our rewrite rules may produce errors (recoverable errors), like failing to remove an extra comma.
  /// This function, applies the recovery strategies.
  /// Currently, we only support recovering from extra comma.
  fn remove_additional_comma_from_sequence_list(&mut self, parser: &mut Parser) {
    if self.ast.root_node().has_error() {
      let c_ast = self.ast.clone();
      for n in traverse(c_ast.walk(), Order::Post) {
        // Remove the extra comma
        if n.is_extra() && eq_without_whitespace(n.utf8_text(self.code().as_bytes()).unwrap(), ",")
        {
          self._apply_edit(n.range(), "", parser);
          break;
        }
      }
    }
  }

  // #[cfg(test)] // Rust analyzer FP
  pub fn code(&self) -> String {
    String::from(&self.code)
  }
  #[cfg(test)] // Rust analyzer FP
  pub fn path(&self) -> &PathBuf {
    &self.path
  }

  pub(crate) fn substitutions(&self) -> &HashMap<String, String> {
    &self.substitutions
  }

  pub(crate) fn add_to_substitutions(&mut self, new_entries: &HashMap<String, String>) {
    let _ = &self.substitutions.extend(new_entries.clone());
  }
}

#[cfg(test)]
#[path = "unit_tests/source_code_unit_test.rs"]
mod source_code_unit_test;
