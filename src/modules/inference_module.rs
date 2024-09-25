use reqwest;
use std::error::Error;
use std::fs;
use std::process::Command;
use url::Url;
use base64;
use std::path::PathBuf;

/// Represents an inference module that can be installed and managed.
pub struct InferenceModule {
    /// The name of the inference module.
    pub name: String,
    pub root_dir: PathBuf,
}

impl InferenceModule {
    /// Creates a new InferenceModule instance.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL of the module to be installed.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - Returns an InferenceModule instance if successful, or an error if the URL is invalid.
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let parsed_url = Url::parse(url)?;
        let name = parsed_url.path_segments()
            .and_then(|segments| segments.last())
            .ok_or("Invalid URL: cannot extract inference name")?
            .to_string();
        let root_dir = PathBuf::from(".");
        Ok(InferenceModule { name, root_dir })
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
        

        let url = format!("https://registrar-agentartificial.ngrok.dev/modules/{}", self.name);
        let response = reqwest::get(&url).await?.text().await?;

        println!("Received response: {:?}", response);

        // Remove surrounding quotes if present
        let cleaned_response = response.trim_matches('"').replace("\\", "").replace("\"", "");

        // Attempt to decode as base64
        let decoded_content = match base64::decode(cleaned_response.clone()) {
            Ok(content) => String::from_utf8(content)
                .map_err(|e| format!("Failed to convert decoded bytes to UTF-8: {}", e))?,
            Err(_) => {
                // If base64 decoding fails, assume it's already decoded
                cleaned_response.to_string()
            }
        };

        println!("Decoded/cleaned script content:\n{}", decoded_content);

        // Create the module directory
        let module_dir = PathBuf::from("modules").join(&self.name);
        fs::create_dir_all(&module_dir)?;

        // Save the script
        let script_name = format!("setup_{}.py", self.name);
        let script_path = module_dir.join(&script_name);
        fs::write(&script_path, &decoded_content)
            .map_err(|e| format!("Failed to write script to {}: {}", script_path.display(), e))?;

        println!("Script saved to: {}", script_path.display());

        // Create Python virtual environment
        let venv_path = PathBuf::from(".venv");
        if !venv_path.exists() {
            let output = Command::new("python")
                .args(&["-m", "venv", ".venv"])
                .current_dir(".")
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to create virtual environment: {}", error).into());
            }
            println!("Created Python virtual environment");
        }

        // Activate virtual environment and run the Python script
        let python_executable = if cfg!(windows) {
            ".venv\\Scripts\\python.exe"
        } else {
            ".venv/bin/python"
        };

        let output = Command::new(python_executable)
            .arg(&script_path)
            .current_dir(".")
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to run Python script: {}", error).into());
        }

        println!("Python script executed successfully");

        let bash_script_path = self.root_dir.join("modules").join(self.name.clone()).join(format!("install_{}.sh", self.name.clone()));
        let bash_script_string = format!("{}", bash_script_path.display());

        // Make the script executable
        let output = Command::new("bash")
            .current_dir(".")
            .arg(&bash_script_string)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to run bash script: {}", error).into());
        }

        println!("Bash script executed successfully");
        println!("Module installed successfully");
        Ok(())
    }
}