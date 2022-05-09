use std::{collections::HashMap, path::{PathBuf, Path}, fs};

use tree_sitter::{Tree, Node, InputEdit, Range, Parser};

use crate::utilities::tree_sitter_utilities::get_edit;

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
      fs::write(&self.path, self.code.as_str()).expect("Unable to Write file");
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
    pub(crate) fn apply_edit(
      &mut self, replace_range: Range, replacement_str: String, parser: &mut Parser,
    ) -> InputEdit {
      // Get the tree_sitter's input edit representation
      let (new_source_code, edit) = get_edit(self.code.clone(), replace_range, &replacement_str);
      // Apply edit to the tree
      self.ast.edit(&edit);
      // Create a new updated tree from the previous tree
      let new_tree = parser
        .parse(&new_source_code, Some(&self.ast))
        .expect("Could not generate new tree!");
      self.ast = new_tree;
      self.code = new_source_code;
      edit
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

    pub(crate) fn add_to_substitutions(&mut self, new_entries: HashMap<String, String>) {
        let _ = &self.substitutions.extend(new_entries);
    }
}