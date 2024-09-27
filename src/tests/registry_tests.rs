use crate::registry::ModuleRegistry;
use tokio;

#[tokio::test]
async fn test_register_and_get_module() {
    let database_url = "postgres://user:pass@localhost/testdb";
    let mut registry = ModuleRegistry::new(database_url).await.unwrap();

    registry.register_module("test_module".to_string(), "inference".to_string()).await.unwrap();

    let module = registry.get_module("test_module").await.unwrap();
    assert_eq!(module, Some(("test_module".to_string(), "inference".to_string())));
}

#[tokio::test]
async fn test_list_modules() {
    let database_url = "postgres://user:pass@localhost/testdb";
    let mut registry = ModuleRegistry::new(database_url).await.unwrap();

    registry.register_module("module1".to_string(), "inference".to_string()).await.unwrap();
    registry.register_module("module2".to_string(), "subnet".to_string()).await.unwrap();

    let modules = registry.list_modules().await.unwrap();
    assert_eq!(modules.len(), 2);
    assert!(modules.contains(&("module1".to_string(), "inference".to_string())));
    assert!(modules.contains(&("module2".to_string(), "subnet".to_string())));
}