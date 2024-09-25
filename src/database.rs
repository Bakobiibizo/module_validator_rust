use sqlx::postgres::{PgPool, PgRow};
use sqlx::Row;
use std::error::Error as StdError;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn StdError>> {
        let pool = PgPool::connect(database_url).await?;

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

    pub async fn unregister_module(&self, name: &str) -> Result<(), Box<dyn StdError>> {
        sqlx::query!("DELETE FROM modules WHERE name = $1", name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_module(&self, name: &str) -> Result<Option<String>, Box<dyn StdError>> {
        let result = sqlx::query("SELECT module_type FROM modules WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|row: PgRow| row.get("module_type")))
    }

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