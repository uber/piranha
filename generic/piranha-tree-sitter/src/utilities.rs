use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::{DirEntry, File, self};
use std::io::{BufReader, Read};
use std::hash::Hash;
// use extend::ext;

pub fn read_file(file_path: &PathBuf) -> String {
    let mut content = String::new();
    let file =
        File::open(&file_path).expect(format!("Could not read file {:?}", &file_path).as_str());
    let mut buf_reader = BufReader::new(file);
    let _ = buf_reader.read_to_string(&mut content);
    content
}

pub fn has_extension(dir_entry: &DirEntry, extension: &str) -> bool {
    dir_entry
        .path()
        .extension()
        .map(|e| e.eq(extension))
        .unwrap_or(false)
}

pub fn get_extension(language: &str) -> &str {
    match language {
        "Java" => "java",
        "Swift" => "swift",
        _ => panic!("Language not supported"),
    }
}

pub fn get_files_with_extension(input_dir: &str, extension: &str) -> Vec<DirEntry>{
      fs::read_dir(input_dir)
            .unwrap()
            .filter_map(|d| d.ok())
            .filter(|de| has_extension(de, extension))
            .collect()
    
}

pub fn substitute_in_str(
    substitutes: &HashMap<String, String>,
    value: &String,
    key_mapper: &dyn Fn(&String) -> String
) -> String {
    let mut output = String::from(value);
    for (tag, substitute) in substitutes {
        let key = key_mapper(tag);
        output = output.replace(&key, substitute)
    }
    output
}

pub trait MapOfVec<T, V> {
    fn collect(&mut self,key: T, value: V) ;
}

impl<T: Hash + Eq, U> MapOfVec<T, U> for HashMap<T, Vec<U>>  {
    fn collect(self: &mut HashMap<T, Vec<U>>, key: T, value : U) {
        self.entry(key)
                .or_insert_with(Vec::new)
                .push(value);
    }
}

// pub fn apply_substitutions_to_string(item: String, substitutions:HashMap<String,String>)-> String{
//     let mut s = item;
//     for (k, v) in substitutions{
//         s = s.replace(k.as_str(), v.as_str());
//     }
//     return s;
// }