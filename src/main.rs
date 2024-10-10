//! Main module for the Module Validator application.
//!
//! This application manages the installation, listing, and execution of various modules.
//! It provides a command-line interface for interacting with the module system.

use clap::Parser;
mod cli;
mod config_parser;
mod inference;
mod modules;
mod utils;
mod validator;
mod miner;
mod proxy;
mod api;
use crate::api::API;

use cli::{Cli, Commands};
use dotenv::dotenv;
use std::path::Path;
use std::path::PathBuf;

use crate::miner::Miner;
use crate::config_parser::ConfigParser;
use crate::inference::python_executor::{activate_env, PythonExecutor};
use crate::modules::inference_module::InferenceModule;
use crate::modules::subnet_module::SubnetModule;
use crate::validator::Validator;
use crate::inference::translation::TranslationAPI;

/// Main entry point for the Module Validator application.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize the module registry with the test flag
    let mut module_type = String::new();
    let mut module_name = String::new();

    // Match the command from CLI and execute corresponding logic
    match &cli.command {
        Commands::Install { url } => {
            // Determine module type and name based on the URL
            if url.contains("://") || url.contains('/') {
                module_name = url.split("/").last().unwrap().to_string();
                module_type = if url.contains("github.com") {
                    "subnets".to_string()
                } else {
                    "inference".to_string()
                };
            } else {
                module_name = url.to_string();
                module_type = "inference".to_string();
            }
            activate_env(&PathBuf::from(format!(".{}", module_name)))?;

            if module_type == "subnets" {
                // Install and register subnet module
                let mut subnet_module = SubnetModule::new(url, &module_name)?;
                subnet_module.install().await?;
                println!(
                    "{} module installed and registered successfully",
                    module_name
                );

                // Parse and configure module
                let module_dir = Path::new(&module_type).join(&module_name);
                let mut config = ConfigParser::parse_commands(&module_dir)?;
                print_config(&config);
            } else {
                // Install and register inference module
                let inference_module = InferenceModule::new(url)?;
                inference_module.install().await?;
                println!(
                    "{} module installed and registered successfully",
                    module_name
                );
            }
        }
        Commands::RunInference { name, input } => {
            println!("Running inference for module: {}", name);
            let module_name = name.clone();
            let module_type = "inference".to_string();
            let target_script_path = format!("{}/{}/{}.py", "modules", &module_name, &module_name);

            println!("Target script path: {}", target_script_path);
            let python_executor =
                PythonExecutor::new(module_name.clone(), module_type, target_script_path)?; // Use the ? operator to propagate the error

            // Split the input string into a vector of arguments
            let args = input.to_string();

            match python_executor.run_command(args) {
                Ok(result) => println!("Inference result: {}", result),
                Err(e) => println!("Error running inference: {}", e),
            }
        }
        Commands::ParseConfig { name } => {
            let module_dir = PathBuf::from("subnets").join(name);
            println!("Attempting to parse config from: {:?}", module_dir);
            if module_dir.exists() {
                println!("Module directory found. Parsing configuration...");
                match ConfigParser::parse_commands(&module_dir) {
                    Ok(mut config) => {
                        println!("Successfully parsed configuration:");
                        print_config(&config);

                        // Prompt for environment variables
                        if let Err(e) = ConfigParser::prompt_for_env_vars(&mut config) {
                            println!("Error prompting for environment variables: {}", e);
                        }

                        // Save the configuration
                        match ConfigParser::save_config(&config, &module_dir) {
                            Ok(_) => println!("Configuration saved successfully."),
                            Err(e) => println!("Error saving configuration: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("Error parsing configuration: {}", e);
                    }
                }
            } else {
                println!("Module directory not found: {:?}", module_dir);
            }
        }
        Commands::LaunchValidator { name, args } => {
            let mut validator = Validator::new(&name).unwrap();

            validator.identify_and_prepare_inference(&args)?;
            let output = validator.launch(if args.is_empty() { None } else { Some(&args) })?;
            println!("Validator output: {:?}", output);
        }
        Commands::LaunchMiner { name, args } => {
            let mut miner = Miner::new(&name).unwrap();

            miner.identify_and_prepare_inference(&args)?;
            let output = miner.launch(if args.is_empty() { None } else { Some(&args) })?;
            println!("Miner output: {:?}", output);
        }
        Commands::StartTranslationAPI => {
            let mut translation_api = TranslationAPI::new();
            translation_api.start_with_pm2()?;
        }
        Commands::LaunchProxy { ip, port, target_url } => {
            let proxy = proxy::Proxy::new(ip.to_string(), *port, target_url.to_string());
            proxy.run().await?;
        }
        Commands::StartAPI { port } => {
            API::start("127.0.0.1".to_string(), *port).await?;
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
