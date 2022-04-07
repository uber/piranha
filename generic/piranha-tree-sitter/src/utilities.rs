use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use regex::Regex;
use jwalk::{DirEntry, WalkDir};
// use extend::ext;

pub fn read_file(file_path: &PathBuf) -> String {
    let mut content = String::new();
    let file =
        File::open(&file_path).expect(format!("Could not read file {:?}", &file_path).as_str());
    let mut buf_reader = BufReader::new(file);
    let _ = buf_reader.read_to_string(&mut content);
    content
}

// pub fn has_extension(dir_entry: &DirEntry, extension: &str) -> bool {
//     dir_entry
//         .path()
//         .extension()
//         .map(|e| e.eq(extension))
//         .unwrap_or(false)
// }

pub fn get_files_with_extension(input_dir: &String, extension: &str, pattern: Regex) -> HashMap<PathBuf, String> {
    WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|de| de.path().extension().map(|e|e.eq(extension)).unwrap_or(false))
        .map(|f|(f.path().to_path_buf(),read_file(&f.path().to_path_buf())))
        .filter(|x|pattern.is_match(x.1.as_str()))
        .collect()
}

pub trait MapOfVec<T, V> {
    fn collect_as_counter(&mut self, key: T, value: V);
}

impl<T: Hash + Eq, U> MapOfVec<T, U> for HashMap<T, Vec<U>> {
    fn collect_as_counter(self: &mut HashMap<T, Vec<U>>, key: T, value: U) {
        self.entry(key).or_insert_with(Vec::new).push(value);
    }
}
