use std::error::Error;
use std::process::Command;
use url::Url;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use dialoguer::{MultiSelect, Confirm};
use crate::modules::inference_module::InferenceModule;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

/// Represents a subnet module that can be installed and managed.
pub struct SubnetModule {
    /// The name of the subnet module.
    pub name: String,
    /// The URL from which the module can be downloaded.
    pub url: String,
    /// The set of required inference modules for this subnet module.
    pub required_inference_modules: HashSet<String>,
}

impl SubnetModule {
    /// Creates a new SubnetModule instance.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the subnet module repository.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - Returns a SubnetModule instance if successful, or an error if the URL is invalid.
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

    /// Installs the subnet module.
    ///
    /// This function performs the following steps:
    /// 1. Clones the repository from the provided URL.
    /// 2. Runs the setup script if it exists.
    /// 3. Installs Python requirements if a requirements.txt file is present.
    /// 4. Prompts the user to select required inference modules.
    /// 5. Installs the selected inference modules.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Returns Ok(()) if the installation is successful, or an error if any step fails.
    pub async fn install(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Installing subnet module: {}", self.name);

        let module_dir = PathBuf::from("subnets").join(&self.name);

        if module_dir.exists() {
            println!("Subnet module {} is already installed.", self.name);
            return Ok(());
        }

        let env_dir = PathBuf::from(format!(".{}", self.name));
        self.run_command_with_output("python", &["-m", "venv", env_dir.to_str().unwrap()])?;

        let python_executable = self.get_venv_python(&env_dir)?;

        self.run_command_with_output("git", &["clone", &self.url, &module_dir.to_string_lossy()])?;

        println!("Repository cloned successfully");

        let setup_script = module_dir.join("setup.sh");
        match setup_script.exists() {
            true => {
                println!("Running setup script");
                self.run_command_with_output("bash", &[setup_script.to_str().unwrap()])?;
                println!("Setup script executed successfully");
            }
            false => {
                println!("No setup script found");
            }
        }

        let requirements_file = module_dir.join("requirements.txt");
        match self.run_command_with_output(&python_executable.to_path_buf().to_str().unwrap(), &["-m", "pip", "install", "-r", requirements_file.to_str().unwrap()]) {
            Ok(_) => println!("Python requirements installed successfully"),
            Err(e) => {
                eprintln!("Warning: Failed to install Python requirements: {}", e);
                println!("Continuing with installation process...");
            }
        }

        println!("Installing package in editable mode");
        match self.run_command_with_output(&python_executable.to_path_buf().to_str().unwrap(), &["-m", "pip", "install", "-e", "."]) {
            Ok(_) => println!("Package installed in editable mode successfully"),
            Err(e) => {
                eprintln!("Warning: Failed to install package in editable mode: {}", e);
                println!("Continuing with installation process...");
            }
        }

        self.prompt_for_inference_modules().await?;

        println!("Subnet module installed successfully");
        Ok(())
    }

    // New method to get the Python executable path from the virtual environment
    fn get_venv_python(&self, env_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        let python_path = env_dir.join("Scripts").join("python.exe");

        #[cfg(not(target_os = "windows"))]
        let python_path = env_dir.join("bin").join("python");

        if python_path.exists() {
            Ok(python_path)
        } else {
            Err(format!("Python executable not found in virtual environment: {:?}", python_path).into())
        }
    }

    /// Prompts the user to select required inference modules.
    ///
    /// This function displays a multi-select menu for the user to choose from a list of available inference modules.
    /// The selected modules are then added to the `required_inference_modules` set.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Returns Ok(()) if the user selects at least one module, or an error if an issue occurs.
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

    fn run_command_with_output(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        std::thread::spawn(move || {
            stdout_reader.lines().for_each(|line| {
                if let Ok(line) = line {
                    println!("{}", line);
                }
            });
        });

        std::thread::spawn(move || {
            stderr_reader.lines().for_each(|line| {
                if let Ok(line) = line {
                    eprintln!("{}", line);
                }
            });
        });

        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Command failed with exit code: {}", status).into())
        }
    }
}