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

//! Defines the entry-point for Piranha.
use log::{debug, info};
use polyglot_piranha::models::default_configs::RUBY;
use polyglot_piranha::utilities::tree_sitter_utilities::get_all_matches_for_query;
use polyglot_piranha::{
  execute_piranha,
  models::{
    default_configs::ERB,
    language::PiranhaLanguage,
    piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
    piranha_output::PiranhaOutputSummary,
  },
};
use serde::de;
use std::{fs, time::Instant};
use tree_sitter::Query;
use tree_sitter::{Parser, Range};
/// Demo function showing proper tree-sitter multi-language parsing for ERB
fn demo_erb_parsing() {
  println!("=== Tree-sitter ERB Multi-language Range-based POC ===");

  // Read ERB content from file
  let erb_content = std::fs::read_to_string("/Users/bsunder/uber/piranha/erb_test/sample.html.erb")
    .expect("Failed to read ERB file");
  let erb_content_ref = &erb_content;

  println!("Original ERB Content:");
  println!("{}", erb_content_ref);
  println!();

  // Step 1: Parse the entire document as ERB using embedded-template parser
  let mut parser = Parser::new();
  parser
    .set_language(tree_sitter_embedded_template::language().into())
    .unwrap();
  let erb_tree = parser.parse(erb_content_ref, None).unwrap();
  let erb_root = erb_tree.root_node();

  println!("ERB AST:");
  println!("{}", erb_root.to_sexp());
  println!();

  // Step 2: Extract ranges for HTML content and Ruby code
  let mut content_ranges = Vec::new();
  let mut ruby_ranges = Vec::new();
  extract_ranges(
    erb_root,
    erb_content_ref,
    &mut content_ranges,
    &mut ruby_ranges,
  );

  println!("Extracted Ruby ranges: {} found", ruby_ranges.len());
  for (i, range) in ruby_ranges.iter().enumerate() {
    let text = &erb_content_ref[range.start_byte..range.end_byte];
    println!(
      "  Ruby {}: bytes[{}..{}] -> '{}'",
      i, range.start_byte, range.end_byte, text
    );
  }
  println!();

  // Step 3: Parse Ruby using ranges on the ORIGINAL ERB text
  parser.set_language(tree_sitter_ruby::language()).unwrap();
  parser.set_included_ranges(&ruby_ranges).unwrap();

  // Parse the ORIGINAL ERB text with Ruby ranges - this is the key insight!
  let ruby_tree = parser.parse(erb_content_ref, None).unwrap();
  let ruby_root = ruby_tree.root_node();

  println!("Ruby AST (parsed from original ERB with ranges):");
  println!("{}", ruby_root.to_sexp());
  println!();

  // Step 4: Create a query to find feature flag patterns
  let ruby_language = PiranhaLanguage::from(RUBY);
  let ruby_query = Query::new(
    *ruby_language.language(),
    r#"
      (((call
        method: (identifier) @flag_name
      )@flag_exp
      )
      (#eq? @flag_name "msp?")
      )    
    "#,
  )
  .unwrap();

  // Step 5: Find matches in the Ruby tree (positions refer to original ERB!)
  let matches = get_all_matches_for_query(
    &ruby_root,
    erb_content_ref.to_string(),
    &ruby_query,
    true,
    None,
    None,
  );

  println!("Ruby matches found: {}", matches.len());
  for (i, match_) in matches.iter().enumerate() {
    let range = match_.range();
    let matched_text = &erb_content_ref[*range.start_byte()..*range.end_byte()];
    println!(
      "  Match {}: bytes[{}..{}] -> '{}' (in original ERB!)",
      i,
      range.start_byte(),
      range.end_byte(),
      matched_text
    );
  }
  println!();

  // Step 6: Apply transformations directly to original ERB text
  if !matches.is_empty() {
    let mut updated_erb = erb_content_ref.to_string();

    // Sort matches by position (reverse order to avoid offset shifts)
    let mut sorted_matches = matches.clone();
    sorted_matches.sort_by(|a, b| b.range().start_byte().cmp(a.range().start_byte()));

    println!("Applying transformations to original ERB:");
    for (i, match_) in sorted_matches.iter().enumerate() {
      let range = match_.range();
      let old_text = &updated_erb[*range.start_byte()..*range.end_byte()];
      let new_text = "true"; // Replace msp? with true

      println!(
        "  Transformation {}: '{}' -> '{}' at bytes[{}..{}]",
        i,
        old_text,
        new_text,
        range.start_byte(),
        range.end_byte()
      );

      updated_erb.replace_range(*range.start_byte()..*range.end_byte(), new_text);
    }

    println!();
    println!("Updated ERB Content:");
    println!("{}", updated_erb);
    println!();

    // Step 7: Verify the transformation by parsing the updated content
    println!("Verification - parsing updated ERB:");
    let updated_erb_tree = parser.parse(&updated_erb, Some(&erb_tree)).unwrap();
    let updated_erb_root = updated_erb_tree.root_node();

    // Extract ranges from updated ERB
    let mut updated_content_ranges = Vec::new();
    let mut updated_ruby_ranges = Vec::new();
    extract_ranges(
      updated_erb_root,
      &updated_erb,
      &mut updated_content_ranges,
      &mut updated_ruby_ranges,
    );

    // Parse Ruby from updated ERB
    parser.set_language(tree_sitter_ruby::language()).unwrap();
    parser.set_included_ranges(&updated_ruby_ranges).unwrap();
    let updated_ruby_tree = parser.parse(&updated_erb, None).unwrap();

    println!("Updated Ruby AST:");
    println!("{}", updated_ruby_tree.root_node().to_sexp());

    // Check if msp? matches still exist
    let verification_matches = get_all_matches_for_query(
      &updated_ruby_tree.root_node(),
      updated_erb.to_string(),
      &ruby_query,
      true,
      None,
      None,
    );

    println!(
      "Verification - msp? matches remaining: {}",
      verification_matches.len()
    );
  }

  println!("=== End ERB Range-based POC ===");
  println!();
}

/// Extract HTML and Ruby ranges from the ERB AST
fn extract_ranges(
  node: tree_sitter::Node, source: &str, content_ranges: &mut Vec<Range>,
  ruby_ranges: &mut Vec<Range>,
) {
  let node_type = node.kind();
  // Check if this is a content node (HTML) or code node (Ruby)
  if node_type == "content" {
    content_ranges.push(Range {
      start_byte: node.start_byte(),
      end_byte: node.end_byte(),
      start_point: node.start_position(),
      end_point: node.end_position(),
    });
  } else if node_type == "directive" {
    // For code nodes, we want the actual Ruby code inside
    if let Some(code_child) = node.named_child(0) {
      ruby_ranges.push(Range {
        start_byte: code_child.start_byte(),
        end_byte: code_child.end_byte(),
        start_point: code_child.start_position(),
        end_point: code_child.end_position(),
      });
    }
  }

  // Recursively process children
  for i in 0..node.child_count() {
    if let Some(child) = node.child(i) {
      extract_ranges(child, source, content_ranges, ruby_ranges);
    }
  }
}

fn demo_rb_piranha_cleanups() {
  let args = PiranhaArguments::from_cli();

  let piranha_output_summaries = execute_piranha(&args);

  if let Some(path) = args.path_to_output_summary() {
    write_output_summary(piranha_output_summaries, path);
  }
}

// Add ERB parsing demo
// demo_erb_parsing();

// demo_rb_parsing();
// return;

fn main() {
  let now = Instant::now();
  env_logger::init();

  info!("Executing Polyglot Piranha");

  let args = PiranhaArguments::from_cli();
  println!("args: {:?}", args);

  debug!("Piranha Arguments are \n{:#?}", args);
  let piranha_output_summaries = execute_piranha(&args);

  if let Some(path) = args.path_to_output_summary() {
    write_output_summary(piranha_output_summaries, path);
  }

  info!("Time elapsed - {:?}", now.elapsed().as_secs());
}

/// Writes the output summaries to a Json file named `path_to_output_summaries` .
fn write_output_summary(
  piranha_output_summaries: Vec<PiranhaOutputSummary>, path_to_json: &String,
) {
  if let Ok(contents) = serde_json::to_string_pretty(&piranha_output_summaries) {
    if fs::write(path_to_json, contents).is_ok() {
      return;
    }
  }
  panic!("Could not write the output summary to the file - {path_to_json}");
}
