use std::{env, time::Instant};

use crate::piranha::FlagCleaner;
use config::PiranhaArguments;

mod config;
mod rule_graph;
mod piranha;
mod tree_sitter;
mod utilities;
#[cfg(test)]
mod test;
fn main() {
    let now = Instant::now();
    let args: Vec<String> = env::args().collect();
    let pa = PiranhaArguments::new(
        &args[1], // path_to_test_resource.join("input").to_str().unwrap(),
        &args[2], //language,
        &args[3], //"STALE_FLAG",
        &args[4], //"some_long_name",
        &args[5], //"true",
        &args[6],
    );

    let mut flag_cleaner = FlagCleaner::new(pa);

    flag_cleaner.cleanup();

    for (k, v) in flag_cleaner.relevant_files {
        println!("Rewriting file {:?}", k);
        v.persist(&k);
    }

    println!("Time elapsed - {:?}", now.elapsed().as_secs());
    // ///"/Users/ketkara/repositories/open-source/piranha/generic/piranha-tree-sitter/src/configurations/",)
}
