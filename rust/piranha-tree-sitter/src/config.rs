
use colored::Colorize;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
    pub flag_name: String,
    pub flag_value: String,
    pub flag_namespace: String,
}

// pub struct PiranhaArguments {
//     pub flag_name: String,
//     pub flag_value: bool,
//     pub flag_namespace: String,
// }

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub queries: Vec<String>,
    pub replace: String,
    pub and_then: Option<Vec<Rule>>,
    pub scope: String
}

impl Config {
    pub fn read_config(
        language: &str,
        flag_name: &str,
        flag_namespace: &str,
        flag_value: &str
    ) -> Config {
        println!("{}", format!("Loading Configs").purple());
        let path_buf_to_java_toml = match language {
            "Java" => Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("configurations")
                .join("java_rules.toml"),
            _ => panic!(),
        };
        let flag_val = flag_value.eq("true");
        let treated = format!("{}", flag_val);
        let treated_c = format!("{}", !flag_val);
    
        println!("{}", 
            format!("Piranha arguments are :\n (i) flag_name : {flag_name}\n (ii) Value: {treated} \n (iii) flag_namespace : {flag_namespace}").purple());
    
        let substitutions = HashMap::from([
            (String::from("[stale_flag_name]"), flag_name),
            (String::from("[treated]"),&treated ),
            (String::from("[namespace]"), flag_namespace),
            (String::from("[treated_complement]"), &treated_c)]);
        
            
        let path_to_config_toml = path_buf_to_java_toml.as_path();
        
        // Perform the replacement on the entire Config file
        let file_content = fs::read_to_string(&path_to_config_toml);
        if file_content.is_err(){
            panic!("{}", format!("Could not load configuration file - \n {:?}", file_content.err().unwrap()).red());
        }
        let mut content = file_content.unwrap();
        for (k,v) in &substitutions{
            content = content.replace(k,v);
        }
      
        let config: Config = toml::from_str(content.as_str()).unwrap();
    
        return config;
    }
}


