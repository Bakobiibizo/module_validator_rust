use reqwest;
use std::error::Error;
use std::fs;
use std::process::Command;
use url::Url;
use base64;
use std::path::PathBuf;
use crate::inference::python_executor::{activate_env, install_requirements};


/// Represents an inference module that can be installed and managed.
pub struct InferenceModule {
    /// The name of the inference module.
    pub name: String,
    /// The url from which the module can be downloaded.
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
        let env_path = PathBuf::from(format!(".{}", self.name));

        let python_executable = activate_env(&env_path)?;


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
            install_requirements(&env_path, &python_executable)?;
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
}
