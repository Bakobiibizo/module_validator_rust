use crate::modules::subnet_module::SubnetModule;
use std::path::PathBuf;

#[test]
fn test_subnet_module_creation() {
    let module = SubnetModule::new("https://github.com/example/subnet_module").unwrap();
    assert_eq!(module.name, "subnet_module");
    assert_eq!(module.url, "https://github.com/example/subnet_module");
    assert!(module.required_inference_modules.is_empty());
}

#[tokio::test]
async fn test_subnet_module_install() {
    let mut module = SubnetModule::new("https://github.com/example/test_subnet").unwrap();
    let result = module.install().await;
    assert!(result.is_ok());
    
    // Check if the module directory was created
    let module_dir = PathBuf::from("subnets").join("test_subnet");
    assert!(module_dir.exists());
    
    // Check if the virtual environment was created
    let venv_dir = PathBuf::from(".test_subnet");
    assert!(venv_dir.exists());
}

#[test]
fn test_subnet_module_with_inference_modules() {
    let mut module = SubnetModule::new("https://github.com/example/test_subnet").unwrap();
    module.required_inference_modules.insert("translation".to_string());
    module.required_inference_modules.insert("embedding".to_string());
    
    assert_eq!(module.required_inference_modules.len(), 2);
    assert!(module.required_inference_modules.contains("translation"));
    assert!(module.required_inference_modules.contains("embedding"));
}