use std::collections::HashMap;
use std::fs::{File};
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use colored::Colorize;

// Reads a file at `flie_path`
pub fn read_file(file_path: &PathBuf) -> String {
    if let Ok(file) =  File::open(&file_path) {
        let mut content = String::new();
        let _ = BufReader::new(file).read_to_string(&mut content);
        return content;
    }
    panic!("{}",format!("Could not read file {:?}", &file_path).red())
}


pub trait MapOfVec<T, V> {
    fn collect(&mut self, key: T, value: V);
}

// Implements trait `MapOfVec` for `HashMap<T, Vec<U>>`.
impl<T: Hash + Eq, U> MapOfVec<T, U> for HashMap<T, Vec<U>> {
    
    // Adds the given `value` to the vector corresponding to the `key`. 
    fn collect(self: &mut HashMap<T, Vec<U>>, key: T, value: U) {
        self.entry(key).or_insert_with(Vec::new).push(value);
    }
}
