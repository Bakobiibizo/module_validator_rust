//! Inference module for the Module Validator application.
//!
//! This module provides functionality for installing and managing inference modules.

use crate::inference::python_executor::activate_env;
use base64;
use reqwest;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use url::Url;

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
            let name = parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .ok_or("Invalid URL: cannot extract inference name")?
                .to_string();
            (name, input.to_string())
        } else {
            // If input is just a module name
            (
                input.to_string(),
                format!(
                    "https://registrar-agentartificial.ngrok.dev/modules/{}",
                    input
                ),
            )
        };

        let root_dir = PathBuf::from(".");
        Ok(InferenceModule {
            name,
            url,
            root_dir,
        })
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

            let url = format!(
                "https://registrar-agentartificial.ngrok.dev/modules/{}",
                self.name
            );
            let response = reqwest::get(&url).await?.text().await?;

            println!("Received response: {:?}", response);

            let cleaned_response = response
                .trim_matches('"')
                .replace("\\", "")
                .replace("\"", "");

            let decoded_content = match base64::decode(cleaned_response.clone()) {
                Ok(content) => String::from_utf8(content)
                    .map_err(|e| format!("Failed to convert decoded bytes to UTF-8: {}", e))?,
                Err(_) => cleaned_response.to_string(),
            };

            println!("Decoded/cleaned script content:\n{}", decoded_content);

            let script_name = format!("setup_{}.py", self.name);
            let script_path = module_dir.join(&script_name);
            fs::write(&script_path, &decoded_content).map_err(|e| {
                format!("Failed to write script to {}: {}", script_path.display(), e)
            })?;

            println!("Script saved to: {}", script_path.display());
        }
        let env_path = PathBuf::from(format!(".{}", self.name));

        let python_executable = activate_env(&env_path)?;

        // Run setup_MODULE_NAME.py
        let setup_script = module_dir.join(format!("setup_{}.py", self.name.clone()));
        if setup_script.exists() {
            self.run_command_with_output(&python_executable, &[setup_script.to_str().unwrap()])?;
            println!("Setup_{}.py executed successfully", self.name.clone());
        } else {
            println!("Setup_{}.py not found", self.name.clone());
        }

        // make install_MODULE_NAME.sh executable and run it
        let install_script = module_dir.join(format!("install_{}.sh", self.name.clone()));
        if install_script.exists() {
            self.run_command_with_output("chmod", &["+x", install_script.to_str().unwrap()])?;
            self.run_command_with_output("bash", &[install_script.to_str().unwrap()])?;
            println!("install_{}.sh executed successfully", self.name.clone());
        } else {
            println!("install_{}.sh not found", self.name.clone());
        }

        // Prompt the user for API_PORT and API_HOST
        self.prompt_user(&format!("Enter the API_PORT for {}", self.name));
        self.prompt_user(&format!("Enter the API_HOST for {}", self.name));

        println!("Inference module installed/updated successfully");
        Ok(())
    }

    fn prompt_user(&self, prompt: &str) -> () {
        use std::fs::OpenOptions;
        use std::io::{self, Write};

        print!("{}: ", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let mut result_string = input.trim().to_string();

        if prompt.contains("PORT") {
            result_string = format!("{}_API_PORT", self.name.to_uppercase());
        }
        if prompt.contains("HOST") {
            result_string = format!("{}_API_HOST", self.name.to_uppercase());
        }
        let env_file = PathBuf::from(".env".to_string());
        if env_file.exists() {
            let mut file = OpenOptions::new().append(true).open(env_file).unwrap();
            writeln!(file, "{}", result_string).unwrap();
        } else {
            let mut file = fs::File::create(env_file).unwrap();
            writeln!(file, "{}", result_string).unwrap();
        }
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
