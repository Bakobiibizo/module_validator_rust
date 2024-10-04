//! Module Validator Library
//!
//! This library provides functionality for managing and executing Python modules
//! dynamically within a Rust environment. It includes features for module installation,
//! registration, and execution.

#[cfg(test)]
mod tests;

pub mod config;
pub mod utils;
pub mod modules;
pub mod inference;
pub mod validator;
pub mod miner;
pub mod cli;

pub use config::Config;
pub use crate::modules::inference_module::InferenceModule;
pub use crate::utils::parse_url;
