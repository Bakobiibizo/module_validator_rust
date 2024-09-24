use clap::Parser;
mod modules;
mod cli;
mod config_parser;

use dotenv::dotenv;
use module_validator::{Config, ModuleRegistry};
use std::env;
use cli::{Cli, Commands};
use config_parser::ConfigParser;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cli = Cli::parse();

    let _config = Config::from_file("config.yaml")?;
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut registry = ModuleRegistry::new(&database_url).await?;

    match &cli.command {
        Commands::Install { url } => {
            let (module_name, module_type) = registry.install_module(url).await?;
            registry.register_module(module_name.clone(), &module_type, &module_name).await?;
            println!("{} module installed and registered successfully", module_type);

            // Parse commands and create configuration
            if module_type == "subnet" {
                let module_dir = Path::new("modules").join(&module_name);
                let config = ConfigParser::parse_commands(&module_dir)?;
                print_config(&config);
            }
        }
        Commands::List => {
            let modules = registry.list_modules().await?;
            println!("Installed modules:");
            for module in modules {
                println!("- {}", module);
            }
        }
        Commands::Run { name, input } => {
            let result = registry.process(name, input).await?;
            println!("Result: {}", result);
        }
        Commands::Uninstall { name } => {
            registry.unregister_module(name).await?;
            println!("Module uninstalled successfully");
        }
        Commands::ParseConfig { name } => {
            let module_dir = Path::new("modules").join(name);
            if module_dir.exists() {
                let config = ConfigParser::parse_commands(&module_dir)?;
                println!("Parsed configuration for module '{}':", name);
                print_config(&config);
            } else {
                println!("Module '{}' not found", name);
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