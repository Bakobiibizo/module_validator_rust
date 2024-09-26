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
            println!("Running setup script");
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
            println!("Installing Python requirements");
            let output = Command::new("python")
                .args(&["-m", "pip", "install", "-r", "requirements.txt"])
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
        let available_modules = vec!["translation", "embedding", "none"];
    
        let mut selections = Vec::new();
        while selections.is_empty() {
            println!("Please select required inference modules:");
            println!("Use ↑↓ arrows to move, Space to select/deselect, Enter to confirm");
            selections = MultiSelect::new()
                .with_prompt("Select required inference modules")
                .items(&available_modules)
                .interact()?;
    
            if selections.is_empty() {
                println!("No modules selected. Please select at least one option.");
            }
        }
    
        println!("Debug: Selected indices: {:?}", selections);
    
        for &selected_index in &selections {
            let selected_module = available_modules[selected_index];
            println!("Debug: Processing selected module: {}", selected_module);
            if selected_module != "none" {
                self.required_inference_modules.insert(selected_module.to_string());
            }
        }
    
        println!("Debug: Required inference modules: {:?}", self.required_inference_modules);
    
        if self.required_inference_modules.is_empty() {
            println!("Only 'none' was selected. No inference modules will be installed.");
            return Ok(());
        }
    
        let install = Confirm::new()
            .with_prompt("Do you want to install the selected inference modules?")
            .default(true)
            .interact()?;
    
        if install {
            for selected_module in &self.required_inference_modules {
                println!("Installing inference module: {}", selected_module);
                let inference_module = InferenceModule::new(selected_module)?;
                inference_module.install().await?;
                println!("Inference module {} installed successfully", selected_module);
            }
        } else {
            println!("Inference modules were selected but not installed.");
        }
    
        Ok(())
    }
}