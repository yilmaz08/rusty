use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    http: Http,
}

#[derive(Debug, Deserialize)]
pub struct Http {
    routes: HashMap<String, Route>,
    services: HashMap<String, Service>
}

#[derive(Debug, Deserialize)]
pub struct Service {
    module: String,
    action: String
}

#[derive(Debug, Deserialize)]
pub struct Route {
    ports: Vec<String>,
    hosts: Vec<String>,
    force_host: bool,
    force_ssl: bool,

    paths: HashMap<String, Path>
}

#[derive(Debug, Deserialize)]
pub struct Path {
    service: String
}

pub fn parse_toml(toml_path: String) -> Config {
    let toml_str = fs::read_to_string(toml_path).expect("Failed to read config.toml file");
    let parsed_toml: Config = toml::from_str(&toml_str).expect("Failed to deserialize config.toml");

    return parsed_toml;
}