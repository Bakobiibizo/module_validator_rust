//! Main module for the Module Validator application.
//! This application manages the installation, listing, and execution of various modules.

use clap::Parser;
mod modules;
mod cli;
mod config_parser;
mod utils;
mod registry;
mod database;
mod validator;

use dotenv::dotenv;
use std::env;
use cli::{Cli, Commands};
use std::path::Path;
use dialoguer::MultiSelect;
use std::path::PathBuf;
use crate::config_parser::ConfigParser;
use crate::modules::subnet_module::SubnetModule;
use crate::modules::inference_module::InferenceModule;
use crate::registry::ModuleRegistry;
use crate::validator::Validator;

/// Main entry point for the Module Validator application.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Get database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // Initialize the module registry
    let mut registry = ModuleRegistry::new(&database_url).await?;
    let mut module_type = String::new();
    let mut module_name = String::new();

    // Match the command from CLI and execute corresponding logic
    match &cli.command {
        Commands::Install { url } => {
            // Determine module type and name based on the URL
            if url.contains("://") || url.contains('/') {
                let module_info = registry.get_module(url).await?;
                module_name = url.split("/").last().unwrap().to_string();
                module_type = if url.contains("github.com") {
                    "subnet".to_string()
                } else {
                    "inference".to_string()
                };
            } else {
                module_name = url.to_string();
                module_type = "inference".to_string();
            }

            if module_type == "subnet" {
                // Install and register subnet module
                let mut subnet_module = SubnetModule::new(url)?;
                subnet_module.install().await?;
                registry.register_module(module_name.clone(), module_type.clone()).await?;
                println!("{} module installed and registered successfully", module_name);
                
                // Parse and configure module
                let module_dir = Path::new(&module_type).join(&module_name);
                let mut config = ConfigParser::parse_commands(&module_dir)?;
                ConfigParser::prompt_for_env_vars(&mut config)?;
                print_config(&config);

            } else {
                // Install and register inference module
                let inference_module = InferenceModule::new(url)?;
                inference_module.install().await?;
                registry.register_module(module_name.clone(), module_type.clone()).await?;
                println!("{} module installed and registered successfully", module_name);
            }
        }
        Commands::List => {
            // List all installed modules
            let modules = registry.list_modules().await?;
            println!("Installed modules:");
            for (name, module_type) in modules {
                println!("- {} ({})", name, module_type);
            }
        }
        Commands::RunInference { name, input } => {
            // Run inference on a specific module
            let inference_module = InferenceModule::new(name)?;
            let result = inference_module.run_inference(input)?;
            println!("Result: {}", result);
        }
        Commands::Uninstall { name } => {
            // Uninstall a module
            registry.unregister_module(name).await?;
            println!("Module uninstalled successfully");
        }
        Commands::ParseConfig { name } => {
            // Construct the full path to the module directory
            let module_dir = PathBuf::from("subnets").join(name);
            if module_dir.exists() {
                println!("Parsing configuration for subnet module: {}", module_dir.display());
                match ConfigParser::parse_commands(&module_dir) {
                    Ok(mut config) => {
                        // Configuration parsed successfully
                        println!("Parsed configuration for module '{}':", name);
                        print_config(&config);
                        
                        // Optionally, prompt for environment variables
                        if let Err(e) = ConfigParser::prompt_for_env_vars(&mut config) {
                            println!("Error prompting for environment variables: {}", e);
                        }
                    },
                    Err(e) => {
                        println!("Error parsing configuration: {}", e);
                    }
                }
            } else {
                println!("Module directory '{}' not found", module_dir.display());
            }
        },
        Commands::LaunchValidator { name } => {
            // Launch validator for a subnet module
            let module_name = name.to_string();
            let module_list = registry.list_modules().await?;
            if module_list.contains(&(module_name.clone(), "subnet".to_string())) {
                let validator = Validator::new(name);
                validator.launch()?;
            } else {
                println!("'{}' is not a valid subnet module", name);
            }
        }
    }

    Ok(())
}

/// Prints the configuration of a module.
///
/// # Arguments
///
/// * `config` - The module configuration to print.
fn print_config(config: &config_parser::ModuleConfig) {
    println!("Environment variables:");
    for (key, value) in &config.env_vars {
        println!("  {}: {}", key, value);
    }
    println!("Commands:");
    for (command_name, command_config) in &config.commands {
        println!("  {}:", command_name);
        println!("    Function: {}", command_config.function);
        println!("    Arguments:");
        for (arg_name, arg_config) in &command_config.args {
            println!("      {}: {:?}", arg_name, arg_config);
        }
    }
}