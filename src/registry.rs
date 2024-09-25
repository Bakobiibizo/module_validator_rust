use std::collections::HashMap;
use std::error::Error as StdError;
use crate::database::Database;

pub struct ModuleRegistry {
    db: Database,
    modules: HashMap<String, String>,
}

impl ModuleRegistry {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn StdError>> {
        let db = Database::new(database_url).await?;

        Ok(Self {
            db,
            modules: HashMap::new(),
        })
    }

    pub async fn register_module(&mut self, name: String, module_type: String) -> Result<(), Box<dyn StdError>> {
        self.db.register_module(&name, &module_type.clone()).await?;
        self.modules.insert(name.clone(), module_type.clone());
        println!("Module {} registered successfully in the database", name);
        Ok(())
    }

    pub async fn get_module(&self, name: &str) -> Result<Option<(String, String)>, Box<dyn StdError>> {
        match self.db.get_module(name).await? {
            Some(module_type) => Ok(Some((name.to_string(), module_type))),
            None => Ok(None),
        }
    }

    pub async fn list_modules(&self) -> Result<Vec<(String, String)>, Box<dyn StdError>> {
        self.db.list_modules().await
    }

    pub async fn unregister_module(&mut self, module_name: &str) -> Result<(), Box<dyn StdError>> {
        self.db.unregister_module(module_name).await?;
        self.modules.remove(module_name);
        Ok(())
    }
}