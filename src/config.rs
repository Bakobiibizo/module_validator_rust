//! Configuration module for the Module Validator application.
//!
//! This module provides structures and methods for loading and saving
//! configuration data for modules and the application itself.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Represents the configuration of a module.
#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub version: String,
    pub entry_point: String,
}

/// Represents the overall configuration of the application.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub modules: Vec<String>,
    pub log_level: String,
    pub max_concurrent_modules: usize,
}

impl Config {
    /// Loads the configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file.
    ///
    /// # Returns
    ///
    /// A Result containing the Config if successful, or an error if loading fails.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    /// Saves the configuration to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the configuration should be saved.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the save operation.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_yaml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}