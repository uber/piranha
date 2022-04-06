use std::env;

use config::PiranhaArguments;
use piranha::{perform_cleanups_for_code_base_new};

// mod lib;
mod config;
mod rule_graph;

mod tree_sitter;
mod utilities;
mod piranha;
#[cfg(test)]
mod test;
fn main() {
    let args: Vec<String> = env::args().collect();
     perform_cleanups_for_code_base_new(
        PiranhaArguments::new(
        &args[1],// path_to_test_resource.join("input").to_str().unwrap(),
        &args[2],//language,
        &args[3],//"STALE_FLAG",
        &args[4],//"some_long_name",   
        &args[5],//"true",
        &args[6] 
    ));

// ///"/Users/ketkara/repositories/open-source/piranha/generic/piranha-tree-sitter/src/configurations/",) 
}

