use serde::Deserialize;
use std::fs;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub http: Http,
    #[allow(dead_code)]
    pub modules: HashMap<String, String>
}
#[derive(Debug, Deserialize, Clone)]
pub struct Http {
    #[allow(dead_code)]
    pub routes: HashMap<String, Route>,
    #[allow(dead_code)]
    pub services: HashMap<String, Service>,
    #[allow(dead_code)]
    pub max_body_size: usize
}
#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    #[allow(dead_code)]
    pub module: String,
    #[allow(dead_code)]
    pub parameters: HashMap<String, Value>
}
#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    #[allow(dead_code)]
    pub ports: Vec<String>,
    #[allow(dead_code)]
    pub hosts: Vec<String>,
    #[allow(dead_code)]
    pub priority: Option<usize>, // optional
    #[allow(dead_code)]
    pub paths: HashMap<String, String> // Path - Service relation
}

pub fn parse_config(yaml_path: String) -> Config {
    let yaml_str = fs::read_to_string(&yaml_path).expect(&format!("Failed to read {:?} file", &yaml_path));
    let parsed_yaml: Config = serde_yml::from_str(&yaml_str).expect(&format!("Failed to deserialize {:?}", &yaml_path));

    return parsed_yaml;
}

#[derive(Debug, Deserialize, Clone)]
pub struct Data {
    #[allow(dead_code)]
    pub status_codes: HashMap<String, String>,
}

pub fn parse_data(json_path: String) -> Data {
    let json_str = fs::read_to_string(&json_path).expect(&format!("Failed to read {:?} file", &json_path));
    let parsed_json: Data = serde_json::from_str(&json_str).expect(&format!("Failed to deserialize {:?}", &json_path));

    return parsed_json;
}