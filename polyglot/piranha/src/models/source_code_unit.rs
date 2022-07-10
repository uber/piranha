use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use regex::Regex;
use tree_sitter::{InputEdit, Node, Parser, Range, Tree};
use tree_sitter_traversal::{traverse, Order};

use crate::utilities::{eq_without_whitespace, tree_sitter_utilities::get_tree_sitter_edit};

use super::{edit::Edit, rule_store::RuleStore};

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
  pub fn persist(&self) {
    if self.code.as_str().is_empty() {
      _ = fs::remove_file(&self.path).expect("Unable to Delete file");
    } else {
      fs::write(&self.path, self.code.as_str()).expect("Unable to Write file");
    }
  }

  pub(crate) fn apply_edit(
    &mut self, edit: &Edit, parser: &mut Parser, is_match_only: bool,
  ) -> InputEdit {
    // Get the tree_sitter's input edit representation
    self._apply_edit(
      edit.replacement_range(),
      edit.replacement_string(),
      parser,
      true,
      is_match_only,
    )
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
    &mut self, range: Range, replacement_string: &str, parser: &mut Parser, handle_error: bool,
    is_match_only: bool,
  ) -> InputEdit {
    // Get the tree_sitter's input edit representation
    let (new_source_code, ts_edit) =
      get_tree_sitter_edit(self.code.clone(), range, replacement_string, is_match_only);
    // Apply edit to the tree
    if !is_match_only {
      self.ast.edit(&ts_edit);
      // Create a new updated tree from the previous tree
      let new_tree = parser
        .parse(&new_source_code, Some(&self.ast))
        .expect("Could not generate new tree!");
      self.ast = new_tree;
      self.code = new_source_code;
      // Handle errors, like removing extra comma.
      if self.ast.root_node().has_error() && handle_error {
        self.fix_syntax_for_comma_separated_expressions(parser);
      }
    }
    ts_edit
  }

  /// Applies an edit to the source code unit
  /// # Arguments
  /// * `replacement_content` - new content of file
  /// * `parser`
  ///
  /// Note - Causes side effect. - Updates `self.ast` and `self.code`
  pub(crate) fn _apply_edit_replace_file_contents(
    &mut self, replacement_content: &str, parser: &mut Parser,
  ) {
    println!("Replacing file contents");
    // Create a new updated tree from the previous tree
    let new_tree = parser
      .parse(&replacement_content, None)
      .expect("Could not generate new tree!");
    self.ast = new_tree;
    self.code = replacement_content.to_string();
  }



  // Tries to remove the extra comma - 
  // -->  Remove comma if extra
  //    --> Check if AST is correct 
  //      ---> No: Undo the change
  // Returns true if the comma was successfully removed.
  fn try_to_remove_extra_comma(&mut self, parser: &mut Parser) -> bool{
    let c_ast = self.ast.clone();
      for n in traverse(c_ast.walk(), Order::Post) {
        // Remove the extra comma
        if n.is_extra() && eq_without_whitespace(n.utf8_text(self.code().as_bytes()).unwrap(), ",")
        {
          let current_version_code = self.code().clone();
          self._apply_edit(n.range(), "", parser, false, false);
          if self.ast.root_node().has_error() {
            // Undo the edit applied above
            self._apply_edit_replace_file_contents(&current_version_code, parser);
          } else {
            return true;
          }
        }
      }
      false
  }


  // Tries to remove the extra comma - 
  // Applies some Regex Replacements to the source file
  // Returns true if the comma was successfully removed.
  fn try_to_fix_code_with_regex_replace(&mut self, parser: &mut Parser) -> bool{
    let consecutive_comma_pattern = Regex::new(r",\s*\n*,").unwrap();
    let square_bracket_comma_pattern = Regex::new(r"\[\s*\n*,").unwrap();
    let strategies = [(consecutive_comma_pattern, ","), (square_bracket_comma_pattern, "[")];
    
    let mut content = self.code();
    for (regex_pattern, replacement ) in strategies{
        if regex_pattern.is_match(&content){
          content = regex_pattern.replace_all(&content, replacement).to_string();
          self._apply_edit_replace_file_contents(&content, parser);
        }
    }
    return !self.ast.root_node().has_error();
  }


  /// Sometimes our rewrite rules may produce errors (recoverable errors), like failing to remove an extra comma.
  /// This function, applies the recovery strategies.
  /// Currently, we only support recovering from extra comma.
  fn fix_syntax_for_comma_separated_expressions(&mut self, parser: &mut Parser) {
   
    let is_fixed = self.try_to_remove_extra_comma(parser) 
        || self.try_to_fix_code_with_regex_replace(parser);

    if !is_fixed{
      if traverse(self.ast.walk(), Order::Post).any(|n| n.is_error()) {
        panic!(
          "Produced syntactically incorrect source code {}",
          self.code()
        );
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

  pub(crate) fn add_to_substitutions(
    &mut self, new_entries: &HashMap<String, String>, rule_store: &mut RuleStore,
  ) {
    let _ = &self.substitutions.extend(new_entries.clone());
    let global_substitutions: HashMap<String, String> = new_entries
      .iter()
      .filter(|e| e.0.starts_with("global_var_"))
      .map(|(a, b)| (a.to_string(), b.to_string()))
      .collect();
    rule_store.add_to_input_substitutions(&global_substitutions);
  }
}

#[cfg(test)]
#[path = "unit_tests/source_code_unit_test.rs"]
mod source_code_unit_test;
