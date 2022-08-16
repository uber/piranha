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

use crate::{
  models::piranha_arguments::PiranhaArguments, piranha::execute_piranha,
  utilities::initialize_logger,
};
use clap::StructOpt;
use config::CommandLineArguments;
use log::info;
use models::piranha_output::PiranhaOutputSummary;

mod config;
mod models;
mod piranha;
#[cfg(test)]
mod tests;
mod utilities;

fn main() {
  let now = Instant::now();
  initialize_logger(false);

  let args = PiranhaArguments::new(CommandLineArguments::parse());

  let (source_code_units, piranha_output_summaries) = execute_piranha(&args);

  for scu in source_code_units {
    scu.persist(&args);
  }

  if args.path_to_output_summaries().is_some() {
    write_output_summary(piranha_output_summaries, args);
  }

  info!("Time elapsed - {:?}", now.elapsed().as_secs());
}

fn write_output_summary(piranha_output_summaries: Vec<PiranhaOutputSummary>, args: PiranhaArguments) {
    let serialized_summary = serde_json::to_string_pretty(&piranha_output_summaries);
    
    fs::write(
      args.path_to_output_summaries().unwrap(),
      serialized_summary.expect("Unable to Write file"),
    )
    .expect("Unable to Write file");
    
}
