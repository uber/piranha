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

pub(crate) mod tree_sitter_utilities;
use std::collections::HashMap;
use std::fs::File;
#[cfg(test)]
use std::fs::{self, DirEntry};
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::PathBuf;
// Reads a file.
pub(crate) fn read_file(file_path: &PathBuf) -> Result<String, String> {
  File::open(file_path)
    .map(|file| {
      let mut content = String::new();
      let _ = BufReader::new(file).read_to_string(&mut content);
      content
    })
    .map_err(|error| error.to_string())
}

// Reads a toml file. In case of error, it returns a default value (if return_default is true) else panics.
pub(crate) fn read_toml<T>(file_path: &PathBuf, return_default: bool) -> T
where
  T: serde::de::DeserializeOwned + Default,
{
  match read_file(file_path)
    .and_then(|content| toml::from_str::<T>(content.as_str()).map_err(|e| e.to_string()))
  {
    Ok(obj) => obj,
    Err(err) => {
      if return_default {
        T::default()
      } else {
        #[rustfmt::skip]
      panic!("Could not read file: {file_path:?} \n Error : \n {err:?}");
      }
    }
  }
}

pub(crate) fn parse_toml<T>(content: &str) -> T
where
  T: serde::de::DeserializeOwned + Default,
{
  toml::from_str::<T>(content).unwrap()
}

pub(crate) trait MapOfVec<T, V> {
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

/// Compares two strings, ignoring whitespace
pub(crate) fn eq_without_whitespace(s1: &str, s2: &str) -> bool {
  s1.split_whitespace()
    .collect::<String>()
    .eq(&s2.split_whitespace().collect::<String>())
}

/// Checks if the given `dir_entry` is a file named `file_name`
#[cfg(test)] // Rust analyzer FP
pub(crate) fn has_name(dir_entry: &DirEntry, file_name: &str) -> bool {
  dir_entry
    .path()
    .file_name()
    .map(|e| e.eq(file_name))
    .unwrap_or(false)
}

/// Returns the file with the given name within the given directory.
#[cfg(test)] // Rust analyzer FP
pub(crate) fn find_file(input_dir: &PathBuf, name: &str) -> PathBuf {
  fs::read_dir(input_dir)
    .unwrap()
    .filter_map(|d| d.ok())
    .find(|de| has_name(de, name))
    .unwrap()
    .path()
}

macro_rules! gen_py_str_methods {
  ($struct_name:ident) => {
    #[pymethods]
    impl $struct_name {
      fn __repr__(&self) -> String {
        format!("{:?}", self)
      }
      fn __str__(&self) -> String {
        self.__repr__()
      }
    }
  };
}

pub(crate) use gen_py_str_methods;

#[cfg(test)]
#[path = "unit_tests/utilities_test.rs"]
mod utilities_test;
