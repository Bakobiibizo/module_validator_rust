use reqwest;
use std::error::Error;
use std::fs;
use std::process::Command;
use url::Url;
use base64;
use std::path::PathBuf;
use pyo3::prelude::*;


/// Represents an inference module that can be installed and managed.
pub struct InferenceModule {
    /// The name of the inference module.
    pub name: String,
    /// The URL from which the module can be downloaded.
    pub url: String,
    /// The root directory for the module.
    pub root_dir: PathBuf,
}

impl InferenceModule {
    /// Creates a new InferenceModule instance.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice that holds either the URL or the name of the module to be installed.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - Returns an InferenceModule instance if successful, or an error if the input is invalid.
    pub fn new(input: impl AsRef<str>) -> Result<Self, Box<dyn Error>> {
        let input = input.as_ref();
        let (name, url) = if input.contains("://") {
            // If input is a full URL
            let parsed_url = Url::parse(input)?;
            let name = parsed_url.path_segments()
                .and_then(|segments| segments.last())
                .ok_or("Invalid URL: cannot extract inference name")?
                .to_string();
            (name, input.to_string())
        } else {
            // If input is just a module name
            (input.to_string(), format!("https://registrar-agentartificial.ngrok.dev/modules/{}", input))
        };

        let root_dir = PathBuf::from(".");
        Ok(InferenceModule { name, url, root_dir })
    }

    /// Installs the inference module.
    ///
    /// This function performs the following steps:
    /// 1. Downloads the module script from the server.
    /// 2. Decodes the script content if it's base64 encoded.
    /// 3. Saves the script to the appropriate directory.
    /// 4. Creates a Python virtual environment if it doesn't exist.
    /// 5. Runs the setup script in the virtual environment.
    /// 6. Executes any additional installation scripts.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Returns Ok(()) if the installation is successful, or an error if any step fails.
    pub async fn install(&self) -> Result<(), Box<dyn Error>> {
        println!("Installing inference module: {}", self.name);

        let module_dir = self.root_dir.join("modules".to_string()).join(&self.name);

        if module_dir.exists() {
            println!("Module directory already exists. Updating requirements...");
        } else {
            fs::create_dir_all(&module_dir)?;

            let url = format!("https://registrar-agentartificial.ngrok.dev/modules/{}", self.name);
            let response = reqwest::get(&url).await?.text().await?;

            println!("Received response: {:?}", response);

            let cleaned_response = response.trim_matches('"').replace("\\", "").replace("\"", "");

            let decoded_content = match base64::decode(cleaned_response.clone()) {
                Ok(content) => String::from_utf8(content)
                    .map_err(|e| format!("Failed to convert decoded bytes to UTF-8: {}", e))?,
                Err(_) => cleaned_response.to_string(),
            };

            println!("Decoded/cleaned script content:\n{}", decoded_content);

            let script_name = format!("setup_{}.py", self.name);
            let script_path = module_dir.join(&script_name);
            fs::write(&script_path, &decoded_content)
                .map_err(|e| format!("Failed to write script to {}: {}", script_path.display(), e))?;

            println!("Script saved to: {}", script_path.display());
        }

        // Create or update Python virtual environment
        let venv_path = PathBuf::from(format!(".{}", self.name.clone()));

        if !venv_path.exists() {
            let output = Command::new("python")
                .args(&["-m", "venv", venv_path.to_str().unwrap()])
                .current_dir(self.root_dir.clone())
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to create virtual environment: {}", error).into());
            }
            println!("Created Python virtual environment");
        }

        // Activate virtual environment and install/update requirements
        let python_executable = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };

        // Run setup_MODULE_NAME.py
        let setup_script = module_dir.join(format!("setup_{}.py", self.name.clone()));
        if setup_script.exists() {
            let output = Command::new(&python_executable)
                .args(&[setup_script.to_str().unwrap()])
                .current_dir(self.root_dir.clone())
                .output()?;
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to run setup_{}.py: {}", self.name.clone(), error).into());
            }
            println!("Setup_{}.py: {:?}", self.name.clone(), output);
        } else {
            println!("Setup_{}.py not found", self.name.clone());
        }

        // Install Python requirements
        let requirements_file = module_dir.join("requirements.txt");
        if requirements_file.exists() {
            let output = Command::new(&python_executable)
                .args(&["-m", "pip", "install", "-r", "requirements.txt"])
                .current_dir(&module_dir)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to install Python requirements: {}", error).into());
            }

            println!("Python requirements installed/updated successfully");
        }

        // make install_MODULE_NAME.sh executable and run it
        let install_script = module_dir.join(format!("install_{}.sh", self.name.clone()));
        if install_script.exists() {
            let output = Command::new("chmod")
                .args(&["+x", install_script.to_str().unwrap()])
                .output()?;
            println!("chmod: {:?}", output);

            let output = Command::new("bash")
                .args([install_script.to_str().unwrap()])
                .output()?;
            println!("install_{}.sh: {:?}", self.name.clone(), output);

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to run install_{}.sh: {}", self.name.clone(), error).into());
            }
            println!("install_{}.sh: {:?}", self.name.clone(), output);
        } else {
            println!("install_{}.sh not found", self.name.clone());
        }

        println!("Inference module installed/updated successfully");
        Ok(())
    }

    /// Runs the inference on the given input.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice containing the input for the inference.
    ///
    /// # Returns
    ///
    /// * `Result<String, Box<dyn Error>>` - Returns the result of the inference as a string if successful, or an error if the inference fails.
    pub fn run_inference(&self, _input: &str) -> Result<String, Box<dyn Error>> {
        let module_dir = self.root_dir.join(&self.name);
        let python_file = module_dir.join(format!("{}.py", self.name));
        let _wrapper_file = self.root_dir.join("src").join("modules").join("module_wrapper.py");
        let python_exec = if cfg!(windows) {
            "python"
        } else {
            "python3"
        };

        let result = Command::new(python_exec)
            .args(&["-m", "module_wrapper", python_file.to_str().unwrap()])
            .output()?;

        if !result.status.success() {
            let error = String::from_utf8_lossy(&result.stderr);
            return Err(format!("Failed to run inference module: {}", error).into());
        }

        let output = String::from_utf8_lossy(&result.stdout);
        Ok(output.to_string())
    }
}
