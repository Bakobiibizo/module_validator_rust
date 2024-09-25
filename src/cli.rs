use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a new module
    Install {
        /// URL of the module to install
        url: String,
    },
    /// List all installed modules
    List,
    /// Run a module
    RunInference {
        /// Name of the module to run
        name: String,
        /// Input text for the module
        input: String,
    },
    /// Uninstall a module
    Uninstall {
        /// Name of the module to uninstall
        name: String,
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
    },
}