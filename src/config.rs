use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub version: String,
    pub entry_point: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub modules: Vec<String>,
    pub log_level: String,
    pub max_concurrent_modules: usize,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_yaml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}