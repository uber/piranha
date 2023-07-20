use derive_builder::Builder;
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use tree_sitter::{Node, Parser, TreeCursor};
use crate::models::matches::Range;

#[derive(Builder)]
pub struct QueryWriter<'a> {
  node: Node<'a>,
  code: &'a str,
  template_holes: Vec<String>,
  capture_groups: HashMap<String, Node<'a>>,
  count: i32,
  query_ctrs: Vec<String>,
  outer_most_node: Option<String>,
}

impl<'a> QueryWriter<'a> {
  pub fn process_child(&mut self, cursor: TreeCursor<'a>, depth: i32) -> String {
    let prefix = match cursor.field_name() {
      Some(name) => format!("{}: ", name),
      None => String::new(),
    };

    format!("\n{}", self.write_query(cursor.node(), depth + 1, &prefix))
  }

  pub fn write(&mut self) -> String {
    return self.write_query(self.node.clone(), 0, "");
  }

  pub fn write_query(&mut self, node: Node<'a>, depth: i32, prefix: &str) -> String {
    let indent = " ".repeat(depth as usize);

    self.count += 1;
    let node_repr = node.utf8_text(self.code.as_bytes()).unwrap().to_string();
    let node_name = format!("@tag{}n", self.count);

    // Regular node
    let node_type = node.kind();
    let mut s_exp = format!("{}{}({} ", indent, prefix, node_type);

    let mut cursor = node.walk();
    let mut visited = 0;
    cursor.goto_first_child();
    while cursor.goto_next_sibling() {
      if cursor.node().is_named() {
        visited += 1;
        s_exp += &self.process_child(cursor.clone(), depth);
      }
    }

    // if the node is an identifier, add it to eq constraints
    if visited == 0 {
      self.query_ctrs.push(format!(
        "(#eq? {} \"{}\")",
        node_name,
        node.utf8_text(self.code.as_bytes()).unwrap()
      ));
    }
    s_exp += ")";

    self.outer_most_node = Some(node_name.clone());

    let node = node.clone();
    let node_name = node_name.clone();
    self.capture_groups.insert(node_name, node);
    format!("{} {}", s_exp, self.outer_most_node.as_ref().unwrap())
  }
}

use crate::models::capture_group_patterns::MetaSyntax;
use crate::models::language::PiranhaLanguage;
use crate::models::matches::Match;
use crate::utilities::tree_sitter_utilities::get_all_matches_for_query;

pub(crate) fn get_all_matches_for_metasyntax(
  node: &Node, source_code: String, meta: &MetaSyntax, recursive: bool,
  _replace_node: Option<String>,
) -> Vec<Match> {
  let mut parser = Parser::new();
  parser
    .set_language(node.language())
    .expect("Could not set the language for the parser.");

  let re = Regex::new(r":\[\s*(?P<var_name>\w+)(?:\s*:\s*(?P<types>.*))?\s*\]").unwrap();
  let mut replacements = HashMap::new();

  let meta_source = re.replace_all(meta.0.as_str(), |caps: &regex::Captures| {
    let var_name = &caps["var_name"];
    let types = match caps.name("types") {
      Some(m) => m
        .as_str()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>(),
      None => Vec::new(),
    };
    replacements.insert(var_name.to_string(), types);
    var_name.to_string()
  });

  let tree = parser
    .parse(&meta_source.to_string().as_bytes(), None)
    .unwrap();
  let root = tree.root_node().child(0).unwrap();

  let code = meta_source.to_string();
  let mut query_writer = QueryWriterBuilder::default()
    .node(root)
    .code(&code)
    .template_holes(replacements.keys().map(|s| s.to_string()).collect())
    .capture_groups(HashMap::new())
    .count(0)
    .query_ctrs(Vec::new())
    .outer_most_node(None)
    .build()
    .unwrap();

  let query_str = query_writer.write();

  let lang = PiranhaLanguage::from("java");
  let query = lang.create_query(query_str);

  get_all_matches_for_query(
    &root,
    source_code,
    &query,
    recursive,
    Some(query_writer.outer_most_node.unwrap().as_str()[1..].to_string()),
    None,
  )
}



pub(crate) fn get_all_matches_for_metasyntax2(
  node: &Node,
  source_code: String,
  meta: &MetaSyntax,
  recursive: bool,
  _replace_node: Option<String>,
) -> Vec<Match> {
  let mut matches: Vec<Match> = Vec::new();
  let re_var = Regex::new(r"^:\s*\[(?P<var_name>\w+)]").unwrap();
  let syntx = meta.0.as_str();
  print!("meta: {}", syntx);
  let code = node.utf8_text(source_code.as_bytes()).unwrap();
  print!("node: {}", code);
  if let Some(caps) = re_var.captures(meta.0.as_str()) {
    let var_name = &caps["var_name"];
    if var_name == node.utf8_text(source_code.as_bytes()).unwrap() {
      // advance meta (i.e., strip off the matched portion) and return
      let new_meta = MetaSyntax(meta.0.replacen(var_name, "", 1));

      // Create a Match and add it to the matches vector
      let mut match_map = HashMap::new();
      match_map.insert(var_name.to_string(), node.utf8_text(source_code.as_bytes()).unwrap().to_string());
      matches.push(Match{
        matched_string: var_name.to_string(),
        range: Range::from(node.range()),
        matches: match_map,
        associated_comma: None,
        associated_comments: Vec::new()
      });

      matches.extend(get_all_matches_for_metasyntax2(node, source_code, &new_meta, recursive, None));
    }
  } else if meta.0.trim().starts_with(node.utf8_text(source_code.as_bytes()).unwrap().trim()) {
    // advance meta (i.e., strip off the matched portion) and return
    let node_text = node.utf8_text(source_code.as_bytes()).unwrap();
    let new_meta = MetaSyntax(meta.0.replacen(node_text, "", 1));
    matches.extend(get_all_matches_for_metasyntax2(node, source_code, &new_meta, recursive, None));
  } else {
    // for each child of metasyntax, recursive call this function
    let mut cursor = node.walk();
    let children = node.children(&mut cursor);
    for child in children {
      matches.extend(get_all_matches_for_metasyntax2(&child, source_code.clone(), meta, recursive, None));
    }
  }
  matches
}