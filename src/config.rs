use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub update: Option<String>,
    pub build: Option<String>,
    pub install: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

impl Config {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Option<Self> {
        let contents = match fs::read_to_string(filepath) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("no package config found");
                return None;
            }
        };
        let config: Config = toml::from_str(&contents).expect("failed to parse config");
        Some(config)
    }
}
