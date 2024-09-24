use clap::Parser;
mod modules;
mod cli;

use dotenv::dotenv;
use module_validator::{Config, ModuleRegistry};
use std::env;
use cli::{Cli, Commands};

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
    }

    Ok(())
}