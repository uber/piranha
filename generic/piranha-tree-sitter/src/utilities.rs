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

use std::collections::HashMap;
use std::fs::{File};
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use colored::Colorize;

// Reads a file at `file_path`
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
