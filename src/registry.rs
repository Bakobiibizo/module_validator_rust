//! Module registry for the Module Validator application.
//!
//! This module provides a ModuleRegistry struct for managing the registration,
//! unregistration, and querying of modules.

use std::collections::HashMap;
use std::error::Error as StdError;
use crate::database::Database;

/// Manages the registration and retrieval of modules.
pub struct ModuleRegistry {
    db: Database,
    modules: HashMap<String, String>,
}

impl ModuleRegistry {
    /// Creates a new ModuleRegistry instance.
    ///
    /// # Arguments
    ///
    /// * `is_test` - A boolean flag indicating whether to use the test database.
    ///
    /// # Returns
    ///
    /// A Result containing the ModuleRegistry if successful, or an error if the database connection fails.
    pub async fn new(is_test: bool) -> Result<Self, Box<dyn StdError>> {
        let db = Database::new(is_test).await?;

        Ok(Self {
            db,
            modules: HashMap::new(),
        })
    }

    /// Registers a new module in the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module.
    /// * `module_type` - The type of the module (e.g., "subnet" or "inference").
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the registration.
    pub async fn register_module(&mut self, name: String, module_type: String) -> Result<(), Box<dyn StdError>> {
        self.db.register_module(&name, &module_type.clone()).await?;
        self.modules.insert(name.clone(), module_type.clone());
        println!("Module {} registered successfully in the database", name);
        Ok(())
    }

    /// Retrieves information about a module.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing an Option with the module name and type if found, or None if not found.
    pub async fn get_module(&self, name: &str) -> Result<Option<(String, String)>, Box<dyn StdError>> {
        match self.db.get_module(name).await? {
            Some(module_type) => Ok(Some((name.to_string(), module_type))),
            None => Ok(None),
        }
    }

    /// Lists all registered modules.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of tuples with module names and types.
    pub async fn list_modules(&self) -> Result<Vec<(String, String)>, Box<dyn StdError>> {
        self.db.list_modules().await
    }

    /// Unregisters a module from the registry.
    ///
    /// # Arguments
    ///
    /// * `module_name` - The name of the module to unregister.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the unregistration.
    pub async fn unregister_module(&mut self, module_name: &str) -> Result<(), Box<dyn StdError>> {
        self.db.unregister_module(module_name).await?;
        self.modules.remove(module_name);
        Ok(())
    }
}