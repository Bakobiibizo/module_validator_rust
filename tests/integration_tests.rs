use module_validator::{Config, ModuleRegistry, InferenceModule};
use tokio;

#[tokio::test]
async fn test_module_installation_and_registration() {
    let config = Config::from_file("config.yaml").expect("Failed to load config");
    let mut registry = ModuleRegistry::new(&config.database_url).await.expect("Failed to create registry");

    let inference_module = InferenceModule::new("https://registrar-agentartificial.ngrok.dev/modules/translation").expect("Failed to create inference module");
    inference_module.install().await.expect("Failed to install module");

    registry.register_module(inference_module.name.clone(), &inference_module.name).await.expect("Failed to register module");

    let result = registry.process(&inference_module.name, "Hello, world!").await.expect("Failed to process text");
    assert!(!result.is_empty(), "Processed text should not be empty");

    registry.unregister_module(&inference_module.name).await.expect("Failed to unregister module");