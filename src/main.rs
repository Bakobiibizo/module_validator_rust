use dotenv::dotenv;
use module_validator::{Config, ModuleRegistry};
use module_validator::modules::inference_module::InferenceModule;
use std::env;
use tokio;
use std::error::Error;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file
    dotenv().ok();

    // Load configuration
    let _config = Config::from_file("config.yaml")?;

    // Get database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create ModuleRegistry
    let mut registry = ModuleRegistry::new(&database_url).await?;

    // Register InferenceModule
    registry.register_module("inference".to_string(), "inference_module").await?;

    // Install Inference Module
    let inference_module = InferenceModule::new("inference");
    inference_module.install()?;

    // Use InferenceModule
    let processed_text = registry.process("inference", "Hello, world!").await?;
    println!("Processed text: {}", processed_text);

    // Unregister InferenceModule
    registry.unregister_module("inference").await?;

    // Create a new Python interpreter
    Python::with_gil(|py| -> PyResult<()> {
        // Set up the Python path
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (".",))?;

        // Import the module
        let module = py.import("modules.module_wrapper")?;
        
        // Use the module...

        Ok(())
    })?;

    Ok(())
}