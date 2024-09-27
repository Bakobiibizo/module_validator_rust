use crate::config::Config;
use std::fs;
use tempfile::NamedTempFile;


#[test]
fn test_config_from_file() {
    let config_content = r#"
    database_url: "postgres://user:pass@localhost/testdb"
    modules:
      - module1
      - module2
    log_level: "info"
    max_concurrent_modules: 5
    "#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), config_content).unwrap();

    let config = Config::from_file(temp_file.path()).unwrap();

    assert_eq!(config.database_url, "postgres://user:pass@localhost/testdb");
    assert_eq!(config.modules, vec!["module1", "module2"]);
    assert_eq!(config.log_level, "info");
    assert_eq!(config.max_concurrent_modules, 5);
}

#[test]
fn test_config_save() {
    let config = Config {
        database_url: "postgres://user:pass@localhost/testdb".to_string(),
        modules: vec!["module1".to_string(), "module2".to_string()],
        log_level: "debug".to_string(),
        max_concurrent_modules: 3,
    };

    let temp_file = NamedTempFile::new().unwrap();
    config.save(temp_file.path()).unwrap();

    let saved_config = Config::from_file(temp_file.path()).unwrap();
    assert_eq!(config.database_url, saved_config.database_url);
    assert_eq!(config.modules, saved_config.modules);
    assert_eq!(config.log_level, saved_config.log_level);
    assert_eq!(config.max_concurrent_modules, saved_config.max_concurrent_modules);
}