use dotenv::dotenv;
use module_validator::{Config, ModuleRegistry};
use crate::modules::inference_module::InferenceModule;
use std::env;

mod modules;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let _config = Config::from_file("config.yaml")?;
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut registry = ModuleRegistry::new(&database_url).await?;

    // Use the full URL to create the InferenceModule
    let inference_module = InferenceModule::new("https://registrar-agentartificial.ngrok.dev/modules/translation")?;
    inference_module.install().await?;

    // Use the name extracted from the URL to register the module
    registry.register_module(inference_module.name.clone(), &inference_module.name).await?;

    let processed_text = registry.process(&inference_module.name, "Hello, world!").await?;
    println!("Processed text: {}", processed_text);

    registry.unregister_module(&inference_module.name).await?;

    Ok(())
}