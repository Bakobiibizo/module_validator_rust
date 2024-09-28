use crate::database::Database;
use sqlx::postgres::PgPool;
use tokio;

async fn setup_test_db() -> Database {
    let database_url = "postgres://user:pass@localhost/testdb";
    Database::new(true).await.unwrap()
}

#[tokio::test]
async fn test_register_and_get_module() {
    let db = setup_test_db().await;
    
    db.register_module("test_module", "inference").await.unwrap();
    
    let module_type = db.get_module("test_module").await.unwrap();
    assert_eq!(module_type, Some("inference".to_string()));
}

#[tokio::test]
async fn test_unregister_module() {
    let db = setup_test_db().await;
    
    db.register_module("test_module", "inference").await.unwrap();
    db.unregister_module("test_module").await.unwrap();
    
    let module_type = db.get_module("test_module").await.unwrap();
    assert_eq!(module_type, None);
}

#[tokio::test]
async fn test_list_modules() {
    let db = setup_test_db().await;
    
    db.register_module("module1", "inference").await.unwrap();
    db.register_module("module2", "subnet").await.unwrap();
    
    let modules = db.list_modules().await.unwrap();
    assert_eq!(modules.len(), 2);
    assert!(modules.contains(&("module1".to_string(), "inference".to_string())));
    assert!(modules.contains(&("module2".to_string(), "subnet".to_string())));
}