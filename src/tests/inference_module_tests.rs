use crate::modules::inference_module::InferenceModule;
use std::path::PathBuf;

#[tokio::test]
async fn test_inference_module_creation() {
    let module = InferenceModule::new("test_module").unwrap();
    assert_eq!(module.name, "test_module");
    assert_eq!(module.url, "https://registrar-agentartificial.ngrok.dev/modules/test_module");
}

#[tokio::test]
async fn test_inference_module_creation_with_url() {
    let module = InferenceModule::new("https://example.com/modules/custom_module").unwrap();
    assert_eq!(module.name, "custom_module");
    assert_eq!(module.url, "https://example.com/modules/custom_module");
}

#[tokio::test]
async fn test_inference_module_install() {
    let module = InferenceModule::new("test_module").unwrap();
    let result = module.install().await;
    assert!(result.is_ok());
    
    // Check if the module directory was created
    let module_dir = PathBuf::from("modules").join("test_module");
    assert!(module_dir.exists());
    
    // Check if the setup script was created
    let setup_script = module_dir.join("setup_test_module.py");
    assert!(setup_script.exists());
}