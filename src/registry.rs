use pyo3::prelude::*;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::Row;
use std::collections::HashMap;
use std::error::Error as StdError;

pub struct ModuleRegistry {
    db: PgPool,
    modules: HashMap<String, Py<PyAny>>,
}

impl ModuleRegistry {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn StdError>> {
        let db = PgPool::connect(database_url).await?;
        Ok(Self {
            db,
            modules: HashMap::new(),
        })
    }

    pub async fn register_module(&mut self, name: String, module_name: &str) -> Result<(), Box<dyn StdError>> {
        Python::with_gil(|py| -> Result<(), Box<dyn StdError>> {
            let sys = py.import("sys")?;
            sys.getattr("path")?.call_method1("append", ("./",))?;

            let module_wrapper = PyModule::import(py, "modules.module_wrapper")?
                .getattr("ModuleWrapper")?
                .call1((module_name,))?;
            self.modules.insert(name.clone(), module_wrapper.into());
            Ok(())
        })?;

        // Store module info in the database
        sqlx::query!(
            "INSERT INTO modules (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
            name
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_module(&self, name: &str) -> Result<Option<String>, Box<dyn StdError>> {
        let result = sqlx::query("SELECT name FROM modules WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.db)
            .await?;

        Ok(result.map(|row: PgRow| row.get("name")))
    }

    pub async fn list_modules(&self) -> Result<Vec<String>, Box<dyn StdError>> {
        let results = sqlx::query("SELECT name FROM modules")
            .fetch_all(&self.db)
            .await?;

        Ok(results.into_iter().map(|row: PgRow| row.get("name")).collect())
    }

    pub async fn update_module(&self, name: &str, new_name: &str) -> Result<(), Box<dyn StdError>> {
        sqlx::query("UPDATE modules SET name = $1 WHERE name = $2")
            .bind(new_name)
            .bind(name)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    pub async fn unregister_module(&mut self, module_name: &str) -> Result<(), Box<dyn StdError>> {
        if let Some(module) = self.modules.remove(module_name) {
            Python::with_gil(|py| -> Result<(), Box<dyn StdError>> {
                module.call_method0(py, "unload")?;
                Ok(())
            })?;

            // Remove module info from the database
            sqlx::query!("DELETE FROM modules WHERE name = $1", module_name)
                .execute(&self.db)
                .await?;

            Ok(())
        } else {
            Err("Module not found".into())
        }
    }

    pub async fn load_module(&self, module_name: &str) -> Result<(), Box<dyn StdError>> {
        if let Some(module) = self.modules.get(module_name) {
            Python::with_gil(|py| -> Result<(), Box<dyn StdError>> {
                module.call_method0(py, "load")?;
                Ok(())
            })?;
            Ok(())
        } else {
            Err("Module not found".into())
        }
    }

    pub async fn process(&self, module_name: &str, input: &str) -> Result<String, Box<dyn StdError>> {
        if let Some(module) = self.modules.get(module_name) {
            let result: String = Python::with_gil(|py| -> Result<String, Box<dyn StdError>> {
                let result = module.call_method1(py, "process", (input,))?;
                Ok(result.extract::<String>(py)?)
            })?;
            Ok(result)
        } else {
            Err("Module not found".into())
        }
    }
}