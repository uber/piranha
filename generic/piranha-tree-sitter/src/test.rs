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
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::config::{Args, PiranhaArguments};
use crate::piranha::FlagCleaner;
use crate::utilities::read_file;


#[test]
fn test_java_scenarios_treated() {
    let language = "Java";
    let path_to_test_resource =  get_path_to_test_resource(language);
    let args = PiranhaArguments::new(Args {
        path_to_codebase: get_path_to_test_code_base(language),
        language: language.to_string(),
        flag_name: "STALE_FLAG".to_string(),
        flag_namespace: "some_long_name".to_string(),
        flag_value: true,
        path_to_configuration: get_path_test_configurations(language),
    });

    let updated_files = get_cleanups_for_code_base_new(args);

    let path_to_expected = path_to_test_resource.join("expected_treated");

    assert_eq!(updated_files.len(), 5);

    check_result(updated_files, path_to_expected);
}

#[test]
fn test_java_scenarios_control() {
    let language = "Java";
    let path_to_test_resource =  get_path_to_test_resource(language);
    let args = PiranhaArguments::new(Args {
        path_to_codebase: get_path_to_test_code_base(language),
        language: language.to_string(),
        flag_name: "STALE_FLAG".to_string(),
        flag_namespace: "some_long_name".to_string(),
        flag_value: false,
        path_to_configuration: get_path_test_configurations(language),
    });

    let updated_files = get_cleanups_for_code_base_new(args);

    let path_to_expected = path_to_test_resource.join("expected_control");

    assert_eq!(updated_files.len(), 5);

    check_result(updated_files, path_to_expected);
}

fn get_path_to_test_resource(language: &str) -> PathBuf {
    match language {
        "Java" => Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test-resources")
        .join("java"),
        _ => panic!("{} not supported!", language)
    }
}

fn get_path_test_configurations(language: &str) -> String {
    match language {
        "Java" => "src/test-resources/java/configurations/".to_string(),
        _ => panic!("{} not supported!", language)
    }
}

fn get_path_to_test_code_base(language: &str) -> String {
    get_path_to_test_resource(language)
        .join("input")
        .to_str()
        .unwrap()
        .to_string()
}

fn eq_without_whitspace(s1: &String, s2: &String) -> bool {
    s1.replace("\n", "")
            .replace(" ", "")
            .eq(&s2.replace("\n", "").replace(" ", ""))
}

fn check_result(updated_files: HashMap<PathBuf, String>, path_to_expected: PathBuf) {
    let mut results = HashMap::new();
    for (path_buf, new_content) in &updated_files {
        let ufn = &path_buf.file_name().clone();
        let updated_file_name =  ufn.unwrap().to_str().unwrap().to_string();
        let expected_file_path = get_file_with_name(path_to_expected.to_path_buf(), &updated_file_name);
        let expected_content = read_file(&expected_file_path);
        let result =  eq_without_whitspace(&new_content, &expected_content);
        results.insert(path_buf, result);
    }

    let mut all_success_scenarios = true;
    for (file_name, result) in results {
        if result {
            println!("{}", format!("Match successful for {:?}", file_name).green());
        }else{
            println!("{}", format!("Match failed for {:?}", file_name).red());
            all_success_scenarios = false;
            println!("{}", updated_files[file_name]);
        }
    }
    assert!(all_success_scenarios);
    
}

fn get_cleanups_for_code_base_new(args: PiranhaArguments) -> HashMap<PathBuf, String> {
    let mut flag_cleaner = FlagCleaner::new(args);

    flag_cleaner.cleanup();

    flag_cleaner
        .relevant_files
        .iter()
        .map(|(k, x)| (k.clone(), x.code.clone()))
        .collect()
}

pub fn has_name(dir_entry: &DirEntry, extension: &str) -> bool {
    dir_entry
        .path()
        .file_name()
        .map(|e| e.eq(extension))
        .unwrap_or(false)
}

pub fn get_file_with_name(input_dir: PathBuf, name: &String) -> PathBuf {
    fs::read_dir(input_dir)
        .unwrap()
        .filter_map(|d| d.ok())
        .filter(|de| has_name(de, name))
        .next()
        .unwrap()
        .path()
}
