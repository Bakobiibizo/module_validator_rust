use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub modules: Vec<ModuleConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub version: String,
    pub entry_point: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }
}