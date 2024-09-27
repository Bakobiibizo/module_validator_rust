//! Main module for the Module Validator application.
//! This application manages the installation, listing, and execution of various modules.

use clap::Parser;
mod cli;
mod config_parser;
mod database;
mod inference;
mod modules;
mod registry;
mod utils;
mod validator;

use cli::{Cli, Commands};
use dotenv::dotenv;
use std::env;
use std::path::Path;
use std::path::PathBuf;

use crate::config_parser::ConfigParser;
use crate::inference::python_executor::{activate_env, PythonExecutor};
use crate::modules::inference_module::InferenceModule;
use crate::modules::subnet_module::SubnetModule;
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
                let mut subnet_module = SubnetModule::new(url)?;
                subnet_module.install().await?;
                registry
                    .register_module(module_name.clone(), module_type.clone())
                    .await?;
                println!(
                    "{} module installed and registered successfully",
                    module_name
                );

                // Parse and configure module
                let module_dir = Path::new(&module_type).join(&module_name);
                let mut config = ConfigParser::parse_commands(&module_dir)?;
                ConfigParser::prompt_for_env_vars(&mut config)?;
                print_config(&config);
            } else {
                // Install and register inference module
                let inference_module = InferenceModule::new(url)?;
                inference_module.install().await?;
                registry
                    .register_module(module_name.clone(), module_type.clone())
                    .await?;
                println!(
                    "{} module installed and registered successfully",
                    module_name
                );
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
        Commands::Uninstall { name } => {
            // Uninstall a module
            registry.unregister_module(name).await?;
            println!("Module uninstalled successfully");
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
            validator.identify_and_replace_inference(&args)?;
            let output = validator.launch(args)?;
            println!("Validator output: {:?}", output);
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
