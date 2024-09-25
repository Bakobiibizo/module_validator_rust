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

use crate::config_parser::ConfigParser;
use crate::modules::subnet_module::SubnetModule;
use crate::modules::inference_module::InferenceModule;
use crate::registry::ModuleRegistry;
use crate::validator::Validator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cli = Cli::parse();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut registry = ModuleRegistry::new(&database_url).await?;
    let mut module_type = String::new();
    let mut module_name = String::new();
    match &cli.command {
        Commands::Install { url } => {
            if url.contains("://") || url.contains('/') {
                let module_info = registry.get_module(url).await?;
                module_name = url.split("/").last().unwrap().to_string();
                if url.contains("github.com") {
                    module_type = "subnet".to_string();
                }
                else {
                    module_type = "inference".to_string();
                }
            }
            else {
                module_name = url.to_string();
                module_type = "inference".to_string();
            }
            if module_type == "subnet" {
                let mut subnet_module = SubnetModule::new(url)?;
                subnet_module.install().await?;
                registry.register_module(module_name.clone(), module_type.clone()).await?;
                println!("{} module installed and registered successfully", module_name);


                let available_inference_modules = vec!["translation", "embedding", "none"]; // This list should be dynamically generated in the future
                let selected_inference_modules = loop {
                    match MultiSelect::new()
                        .with_prompt("Select required inference modules")
                        .items(&available_inference_modules)
                        .interact()
                    {
                        Ok(selection) => break selection,
                        Err(_) => {
                            println!("Invalid selection. Please try again.");
                            continue;
                        }
                    }
                };

                if selected_inference_modules.is_empty() {
                    println!("No inference modules selected. Exiting.");
                    return Ok(());
                }

                for &index in available_inference_modules.iter() {
                    let index: usize = index.parse().expect("Failed to parse index as usize");
                    if index == 2 {
                        break;
                    }
                    let selected_inference_module = available_inference_modules[index].to_string();
                    let inference_module = InferenceModule::new(selected_inference_module.clone())?;
                    inference_module.install().await?;
                    registry.register_module(selected_inference_module.clone(), module_type.clone()).await?;
                    println!("Inference module {} installed and registered successfully", selected_inference_module);
                }

                let module_dir = Path::new(&module_type).join(&module_name);
                let mut config = ConfigParser::parse_commands(&module_dir)?;
                ConfigParser::prompt_for_env_vars(&mut config)?;
                print_config(&config);

            } else {
                let inference_module = InferenceModule::new(url)?;
                inference_module.install().await?;
                registry.register_module(module_name.clone(), module_type.clone()).await?;
                println!("{} module installed and registered successfully", module_name);
            }
        }
        Commands::List => {
            let modules = registry.list_modules().await?;
            println!("Installed modules:");
            for (name, module_type) in modules {
                println!("- {} ({})", name, module_type);
            }
        }
        Commands::RunInference { name, input } => {
            let inference_module = InferenceModule::new(name)?;
            let result = inference_module.run_inference(input)?;
            println!("Result: {}", result);
        }
        Commands::Uninstall { name } => {
            registry.unregister_module(name).await?;
            println!("Module uninstalled successfully");
        }
        Commands::ParseConfig { name } => {
            let module_dir = Path::new("modules").join(name);
            if module_dir.exists() {
                let mut config = ConfigParser::parse_commands(&module_dir)?;
                ConfigParser::prompt_for_env_vars(&mut config)?;
                println!("Parsed configuration for module '{}':", name);
                print_config(&config);
            } else {
                println!("Module '{}' not found", name);
            }
        }
        Commands::LaunchValidator { name } => {
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