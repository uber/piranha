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

use std::path::PathBuf;

use clap::Parser;

use super::{
  default_configs::default_name_of_piranha_argument_toml,
  piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder},
};

pub enum PiranhaInput {
  CommandLineInput,
  API {
    path_to_codebase: String,
    path_to_configurations: String,
    dry_run: bool,
  },
}

impl From<PiranhaInput> for PiranhaArguments {
  fn from(input: PiranhaInput) -> Self {
    let input_opts = match input {
      PiranhaInput::CommandLineInput => PiranhaArguments::parse(),
      PiranhaInput::API {
        path_to_codebase,
        path_to_configurations,
        dry_run,
      } => PiranhaArgumentsBuilder::default()
        .path_to_codebase(path_to_codebase)
        .path_to_configurations(path_to_configurations)
        .path_to_output_summary(None)
        .dry_run(dry_run)
        .build()
        .unwrap(),
    };
    let path_to_toml = PathBuf::from(input_opts.path_to_configurations())
      .join(default_name_of_piranha_argument_toml());
    let piranha_argument = PiranhaArguments::new(path_to_toml);
    input_opts.merge(piranha_argument)
  }
}
