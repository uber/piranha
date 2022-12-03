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
        .path_to_code_base(path_to_codebase)
        .path_to_configurations(path_to_configurations)
        .path_to_output_summaries(None)
        .dry_run(dry_run)
        .build()
        .unwrap(),
    };
    let path_to_toml = PathBuf::from(input_opts.path_to_configurations())
      .join(default_name_of_piranha_argument_toml());
    let piranha_argument = PiranhaArguments::new(path_to_toml);
    return input_opts.merge(piranha_argument);
  }
}
