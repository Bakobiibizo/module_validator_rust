use std::path::PathBuf;
use crate::validator::Validator;

#[test]
fn test_validator_creation() {
    let validator = Validator::new("test_subnet").unwrap();
    assert_eq!(validator.subnet_name, "test_subnet");
    assert_eq!(validator.env_dir, PathBuf::from(".test_subnet"));
    assert_eq!(validator.module_dir, PathBuf::from("subnets").join("test_subnet"));
}

#[test]
fn test_validator_find_script() {
    let mut validator = Validator::new("test_subnet").unwrap();
    
    // Create a mock validator script
    std::fs::create_dir_all(&validator.module_dir).unwrap();
    std::fs::File::create(validator.module_dir.join("validator.py")).unwrap();
    
    assert!(validator.find_validator_script().is_ok());
    assert!(validator.validator_path.is_some());
}

#[tokio::test]
async fn test_validator_launch() {
    let mut validator = Validator::new("test_subnet").unwrap();
    
    // Create a mock validator script
    std::fs::create_dir_all(&validator.module_dir).unwrap();
    let script_content = r#"
print("Validator launched successfully")
"#;
    std::fs::write(validator.module_dir.join("validator.py"), script_content).unwrap();
    
    validator.find_validator_script().unwrap();
    let result = validator.launch(None);
    assert!(result.is_ok());
}