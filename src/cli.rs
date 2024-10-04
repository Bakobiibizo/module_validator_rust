//! Command-line interface module for the Module Validator application.
//!
//! This module defines the structure and available commands for the CLI.

use clap::{Parser, Subcommand};

/// Represents the command-line interface for the Module Validator application.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

/// Enumerates the available commands for the Module Validator CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// Install a new module
    Install {
        /// URL of the module to install
        url: String,
    },
    /// Run a module
    RunInference {
        /// Name of the module to run
        name: String,
        /// Input text for the module
        input: String,
    },
    /// Parse and display the configuration of an installed module
    ParseConfig {
        /// Name of the module to parse
        name: String,
    },
    /// Launch a validator for a subnet module
    LaunchValidator {
        /// Name of the subnet module to launch validator for
        name: String,
        /// Arguments to pass to the validator (optional)
        #[clap(default_value = "")]
        args: String,
    },
    /// Launch a miner for a subnet module
    LaunchMiner {
        /// Name of the subnet module to launch miner for
        name: String,
        /// Arguments to pass to the miner (optional)
        #[clap(default_value = "")]
        args: String,
    },
    
    /// Start the Translation API
    StartTranslationAPI,
}