use std::error::Error;
use std::process::Command;
use url::Url;
use std::path::PathBuf;
use std::collections::HashSet;
use dialoguer::{MultiSelect, Confirm};
use crate::modules::inference_module::InferenceModule;

pub struct SubnetModule {
    pub name: String,
    pub url: String,
    pub required_inference_modules: HashSet<String>,
}

impl SubnetModule {
    pub fn new(url: impl AsRef<str>) -> Result<Self, Box<dyn Error>> {
        let url = url.as_ref();
        let parsed_url = Url::parse(url)?;
        let name = parsed_url.path_segments()
            .and_then(|segments| segments.last())
            .ok_or("Invalid URL: cannot extract subnet name")?
            .to_string();

        Ok(SubnetModule { 
            name, 
            url: url.to_string(),
            required_inference_modules: HashSet::new(),
        })
    }

    pub async fn install(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Installing subnet module: {}", self.name);

        let module_dir = PathBuf::from("subnets").join(&self.name);

        // Check if the module is already installed
        if module_dir.exists() {
            println!("Subnet module {} is already installed.", self.name);
            return Ok(());
        }

        // Create a new environment for the subnet
        let env_dir = PathBuf::from(format!(".{}", self.name));
        Command::new("python")
            .args(&["-m", "venv", env_dir.to_str().unwrap()])
            .output()?;

        // Clone the repository
        let output = Command::new("git")
            .args(&["clone", &self.url, &module_dir.to_string_lossy()])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to clone repository: {}", error).into());
        }

        println!("Repository cloned successfully");

        // Run setup script if it exists
        let setup_script = module_dir.join("setup.sh");
        if setup_script.exists() {
            let output = Command::new("bash")
                .arg(&setup_script)
                .current_dir(&module_dir)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to run setup script: {}", error).into());
            }

            println!("Setup script executed successfully");
        }

        // Install Python requirements if requirements.txt exists
        let requirements_file = module_dir.join("requirements.txt");
        if requirements_file.exists() {
            let output = Command::new("pip")
                .args(&["install", "-r", "requirements.txt"])
                .current_dir(&module_dir)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to install Python requirements: {}", error).into());
            }

            println!("Python requirements installed successfully");
        }

        self.prompt_for_inference_modules().await?;

        println!("Subnet module installed successfully");
        Ok(())
    }

    async fn prompt_for_inference_modules(&mut self) -> Result<(), Box<dyn Error>> {
        let available_modules = vec!["translation", "embedding"]; // This list should be dynamically generated in the future

        let selections = MultiSelect::new()
            .with_prompt("Select required inference modules")
            .items(&available_modules)
            .interact()?;

        for &index in selections.iter() {
            self.required_inference_modules.insert(available_modules[index].to_string());
        }

        if !self.required_inference_modules.is_empty() {
            let install = Confirm::new()
                .with_prompt("Do you want to install the selected inference modules?")
                .default(true)
                .interact()?;

            if install {
                for selected_module in &self.required_inference_modules {
                    let inference_module = InferenceModule::new(selected_module)?;
                    inference_module.install().await?;
                    println!("Inference module {} installed successfully", selected_module);
                }
            }
        }

        Ok(())
    }
}