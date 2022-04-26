/* 
Copyright (c) 2019 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

//! Defines the entrypoint for Piranha. 
use std::time::Instant;

use crate::{piranha::FlagCleaner};
use clap::StructOpt;
use config::command_line_arguments:: {PiranhaArguments, Args};

mod config;
mod piranha;
mod tree_sitter;
mod utilities;
#[cfg(test)]
mod test;

fn main() {
    let now = Instant::now();
    let args = Args::parse();
    let pa = PiranhaArguments::new(args);

    let mut flag_cleaner = FlagCleaner::new(pa);

    flag_cleaner.cleanup();

    for (k, v) in flag_cleaner.relevant_files {
        println!("Updating file {:?}", k);
        v.persist();
    }

    println!("Time elapsed - {:?}", now.elapsed().as_secs());
}
