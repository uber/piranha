use std::path::PathBuf;
use std::fs::{DirEntry, File, self};
use std::io::{BufReader, Read};

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

pub fn has_name(dir_entry: &DirEntry, extension: &str) -> bool {
    dir_entry
        .path()
        .file_name()
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

pub fn get_file_with_name(input_dir: &str, name: &str, ) -> Option<DirEntry>{
    fs::read_dir(input_dir)
          .unwrap()
          .filter_map(|d| d.ok())
          .filter(|de| has_name(de, name))
          .next()
  
}