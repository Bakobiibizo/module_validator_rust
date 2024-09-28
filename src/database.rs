//! Database module for the Module Validator application.
//!
//! This module provides a Database struct for interacting with the PostgreSQL database,
//! including operations for registering, unregistering, and querying modules.

use sqlx::postgres::{PgPool, PgRow};
use sqlx::Row;
use std::error::Error as StdError;
use dotenv::dotenv;
use std::env;

/// Represents a connection to the database and provides database operations.
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Creates a new Database instance.
    ///
    /// # Arguments
    ///
    /// * `is_test` - A boolean flag indicating whether to use the test database URL.
    ///
    /// # Returns
    ///
    /// A Result containing the Database if successful, or an error if the connection fails.
    pub async fn new(is_test: bool) -> Result<Self, Box<dyn StdError>> {
        dotenv().ok(); // Load .env file, ignoring errors if file doesn't exist

        let env_var_name = if is_test { "TEST_DATABASE_URL" } else { "DATABASE_URL" };
        let database_url = env::var(env_var_name)
            .or_else(|_| env::var("DATABASE_URL")) // Fallback to DATABASE_URL if TEST_DATABASE_URL is not set
            .unwrap_or_else(|_| "postgres://localhost/default_db".to_string()); // Default URL

        let pool = PgPool::connect(&database_url).await?;

        // Create the modules table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS modules (
                name TEXT PRIMARY KEY,
                module_type TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Registers a module in the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to register.
    /// * `module_type` - The type of the module.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the registration.
    pub async fn register_module(&self, name: &str, module_type: &str) -> Result<(), Box<dyn StdError>> {
        sqlx::query!(
            "INSERT INTO modules (name, module_type) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET module_type = $2",
            name,
            module_type
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Unregisters a module from the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to unregister.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the unregistration.
    pub async fn unregister_module(&self, name: &str) -> Result<(), Box<dyn StdError>> {
        sqlx::query!("DELETE FROM modules WHERE name = $1", name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Retrieves information about a module from the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing an Option with the module type if found, or None if not found.
    pub async fn get_module(&self, name: &str) -> Result<Option<String>, Box<dyn StdError>> {
        let result = sqlx::query("SELECT module_type FROM modules WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|row: PgRow| row.get("module_type")))
    }

    /// Lists all modules in the database.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of tuples with module names and types.
    pub async fn list_modules(&self) -> Result<Vec<(String, String)>, Box<dyn StdError>> {
        let results = sqlx::query("SELECT name, module_type FROM modules")
            .fetch_all(&self.pool)
            .await?;

        Ok(results
            .into_iter()
            .map(|row: PgRow| (row.get("name"), row.get("module_type")))
            .collect())
    }
}