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

//! Defines general utility functions. 
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use toml;

// Reads a file.
pub fn read_file(file_path: &PathBuf) -> Result<String, String>  {

    File::open(&file_path)
        .map(|file|{
            let mut content = String::new();
            let _ = BufReader::new(file).read_to_string(&mut content);
            content        
        })
        .map_err(|error| error.to_string())
}

// Reads a toml file. In case of error, it returns a default value (if return_default is true) else panics.
pub fn read_toml<T>(file_path: &PathBuf, return_default: bool) -> T where T: serde::de::DeserializeOwned + Default {
    let obj = read_file(file_path)
        .and_then(|content|toml::from_str::<T>(content.as_str()).map_err(|e|e.to_string()));  
    if obj.is_ok() {
        return obj.unwrap();
    }
    if return_default{
        return T::default();
    }
    panic!("Could not read file: {:?} \n Error : \n {:?}", file_path, obj.err().unwrap());
    
}


pub trait MapOfVec<T, V> {
    fn collect(&mut self, key: T, value: V);
}

// Implements trait `MapOfVec` for `HashMap<T, Vec<U>>`.
impl<T: Hash + Eq, U> MapOfVec<T, U> for HashMap<T, Vec<U>> {
    
    // Adds the given `value` to the vector corresponding to the `key`. 
    // Like an adjacency list. 
    fn collect(self: &mut HashMap<T, Vec<U>>, key: T, value: U) {
        self.entry(key).or_insert_with(Vec::new).push(value);
    }
}


/// Initialize logger.
pub fn initialize_logger(is_test: bool) {
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("piranha.log")
        .unwrap();
    let _ = env_logger::builder().format_timestamp(None)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .is_test(is_test).try_init();
}
