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

//! Defines the entry-point for Piranha.
use std::{fs, time::Instant};

use log::info;
use polyglot_piranha::{
  execute_piranha, models::piranha_arguments::PiranhaArguments,
  models::piranha_output::PiranhaOutputSummary, utilities::initialize_logger,
};

fn main() {
  let now = Instant::now();
  initialize_logger(false);

  let args = PiranhaArguments::from_command_line();

  let piranha_output_summaries = execute_piranha(&args, true);

  if args.path_to_output_summaries().is_some() {
    write_output_summary(
      piranha_output_summaries,
      args.path_to_output_summaries().unwrap(),
    );
  }

  info!("Time elapsed - {:?}", now.elapsed().as_secs());
}

/// Writes the output summaries to a Json file named `path_to_output_summaries` .
fn write_output_summary(
  piranha_output_summaries: Vec<PiranhaOutputSummary>, path_to_json: &String,
) {
  if let Ok(contents) = serde_json::to_string_pretty(&piranha_output_summaries) {
    if let Ok(_) = fs::write(path_to_json, contents) {
      return;
    }
  }
  panic!(
    "Could not write the output summary to the file - {}",
    path_to_json
  );
}
