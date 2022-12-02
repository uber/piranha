use clap::Parser;

use super::piranha_arguments::{PiranhaArguments, PiranhaArgumentsBuilder};

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
    let piranha_argument = PiranhaArguments::new(input_opts.get_path_to_piranha_arguments_toml());
    return input_opts.merge(piranha_argument);
  }
}
