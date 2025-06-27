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
use std::{fs, process, time::Instant};

use log::{debug, info};
use polyglot_piranha::{
  execute_piranha, models::piranha_arguments::PiranhaArguments,
  models::piranha_output::PiranhaOutputSummary,
};

fn main() {
  // Set up the Ctrl+C handler
  ctrlc::set_handler(move || {
    println!("Received Ctrl+C! Exiting...");
    process::exit(130);
  })
  .expect("Error setting Ctrl+C handler");

  let now = Instant::now();
  env_logger::init();

  info!("Executing Polyglot Piranha");

  let args = PiranhaArguments::from_cli();

  debug!("Piranha Arguments are \n{args:#?}");
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
