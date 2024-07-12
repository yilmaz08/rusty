use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub http: Http,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Http {
    #[allow(dead_code)]
    pub routes: HashMap<String, Route>,
    #[allow(dead_code)]
    pub services: HashMap<String, Service>
}
#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    #[allow(dead_code)]
    pub module: String,
    #[allow(dead_code)]
    pub parameters: HashMap<String, String>
}
#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    #[allow(dead_code)]
    pub ports: Vec<String>,
    #[allow(dead_code)]
    pub hosts: Vec<String>,
    #[allow(dead_code)]
    pub force_host: Option<bool>, // optional
    #[allow(dead_code)]
    pub force_ssl: Option<bool>, // optional
    #[allow(dead_code)]
    pub priority: Option<usize>, // optional
    #[allow(dead_code)]
    pub paths: HashMap<String, Path>
}
#[derive(Debug, Deserialize, Clone)]
pub struct Path {
    #[allow(dead_code)]
    pub service: String
}

pub fn parse_config(toml_path: String) -> Config {
    let toml_str = fs::read_to_string(&toml_path).expect(&format!("Failed to read {:?} file", &toml_path));
    let parsed_toml: Config = toml::from_str(&toml_str).expect(&format!("Failed to deserialize {:?}", &toml_path));

    return parsed_toml;
}

#[derive(Debug, Deserialize, Clone)]
pub struct Data {
    #[allow(dead_code)]
    pub status_codes: HashMap<String, String>,
}

pub fn parse_data(toml_path: String) -> Data {
    let toml_str = fs::read_to_string(&toml_path).expect(&format!("Failed to read {:?} file", &toml_path));
    let parsed_toml: Data = toml::from_str(&toml_str).expect(&format!("Failed to deserialize {:?}", &toml_path));

    return parsed_toml;
}