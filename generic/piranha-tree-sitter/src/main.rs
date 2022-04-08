use std::{time::Instant};

use crate::{piranha::FlagCleaner, config::Args};
use clap::StructOpt;
use config::PiranhaArguments;

mod config;
mod rule_graph;
mod piranha;
mod tree_sitter;
mod utilities;
#[cfg(test)]
mod test;

// TODO: Add an argument parser 
//      1. Add parser 
//      2. Adapt other code   (all the scipts) 
// TODO: Add mappings for test method annotations 
// TODO: Should the rules be changed to matcher, replace node, rewrite template
fn main() {
    let now = Instant::now();
    let args = Args::parse();
    let pa = PiranhaArguments::new(args);

    let mut flag_cleaner = FlagCleaner::new(pa);

    flag_cleaner.cleanup();

    for (k, v) in flag_cleaner.relevant_files {
        println!("Rewriting file {:?}", k);
        v.persist(&k);
    }

    println!("Time elapsed - {:?}", now.elapsed().as_secs());
}
