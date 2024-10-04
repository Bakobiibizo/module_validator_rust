//! miner module for subnet modules in the Module miner application.
//!
//! This module provides functionality for validating and launching subnet modules.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use crate::inference::python_executor::PythonExecutor;
use regex::Regex;

/// Represents a miner for subnet modules.
pub struct Miner {
    pub subnet_name: String,
    pub env_dir: PathBuf,
    pub module_dir: PathBuf,
    pub miner_path: Option<PathBuf>,
}

impl Miner {
    /// Creates a new miner instance for a given subnet.
    ///
    /// # Arguments
    ///
    /// * `subnet_name` - The name of the subnet to validate.
    ///
    /// # Returns
    ///
    /// A Result containing the miner if successful, or an error if creation fails.
    pub fn new(subnet_name: &str) -> Result<Self, Box<dyn Error>> {
        println!("Creating new miner for subnet: {}", subnet_name);
        let env_dir = PathBuf::from(format!(".{}", subnet_name));
        let module_dir = PathBuf::from("subnets").join(subnet_name);
        
        let mut miner = Self {
            subnet_name: subnet_name.to_string(),
            env_dir,
            module_dir,
            miner_path: None,
        };

        
        miner.find_miner_script()?;
        Ok(miner)
    }

    /// Prompts the user for the path to the miner script.
    ///
    /// # Returns
    ///
    /// A Result containing the PathBuf of the miner script if successful, or an error if the operation
    #[allow(dead_code)]
    pub fn prompt_user_for_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut miner_path = String::new();
        println!("Enter the path to the miner script: ");
        std::io::stdin().read_line(&mut miner_path).unwrap();
        let miner_path = PathBuf::from(miner_path.trim());
        Ok(miner_path)
    }

    /// Finds the miner script in the module directory.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of finding the miner script.
    pub fn find_miner_script(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Finding miner script in: {:?}", self.module_dir);
        fn find_script(module_dir: &PathBuf) -> Option<PathBuf> {
            if let Ok(entries) = fs::read_dir(module_dir) {
                println!("Entries: {:?}", entries);
                for entry in entries.flatten() {
                    if entry.file_name().to_str().unwrap().contains("stream_tutorial") {
                        println!("Skipping stream_tutorial");
                        continue;
                    }
                    println!("Checking entry: {:?}", entry.path());
                    let path = entry.path();
                    if path.is_file() && path.file_name().unwrap() == "miner.py" {
                        return Some(path);
                    } else if path.is_dir() {
                        if let Some(found_path) = find_script(&path) {
                            return Some(found_path);
                        }
                    }
                }
            }
            None
        }

        if let Some(script_path) = find_script(&self.module_dir) {
            self.miner_path = Some(script_path);
            Ok(())
        } else {
            Err("Could not find the miner script".into())
        }
    }

    /// Identifies and prepares the inference for the subnet.
    ///
    /// # Arguments
    ///
    /// * `_args` - The arguments for the inference (currently unused).
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the preparation.
    pub fn identify_and_prepare_inference(&mut self, _args: &String) -> Result<(), Box<dyn Error>> {
        println!("Preparing inference for subnet: {}", self.subnet_name);
        
        let script_path = self.miner_path.as_ref().ok_or("miner script not found")?;
        println!("Original script path: {:?}", script_path);

        // Adjust the path to be relative to the subnet folder
        let relative_script_path = script_path.strip_prefix(&self.module_dir)?;
        self.miner_path = Some(relative_script_path.to_path_buf());
        
        println!("Adjusted miner path: {:?}", self.miner_path);
        Ok(())
    }

    /// Identifies the inference type in the miner script.
    ///
    /// # Returns
    ///
    /// A Result containing the inference type if successful, or an error if the operation fails.
    pub fn identify_inference_type(&self) -> Result<String, Box<dyn Error>> {
        let miner_path = self.miner_path.as_ref().ok_or("miner path not set")?;
        let content = fs::read_to_string(miner_path)?;
        for module in fs::read_dir("modules")? {
            let module = module.unwrap();
            if content.contains(module.file_name().to_str().unwrap()) {
                return Ok(module.file_name().to_str().unwrap().to_string());
            }
        }
        Err("Inference type not found".into())
    }
    

    /// Launches the miner for the subnet.
    ///
    /// # Arguments
    ///
    /// * `args` - Optional arguments to pass to the miner.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the miner launch.
    pub fn launch(&self, args: Option<&String>) -> Result<(), Box<dyn Error>> {
        println!("Launching miner for subnet: {}", self.subnet_name);
        let miner_path = self.miner_path.as_ref().ok_or("miner path not set")?;
        
        println!("miner path: {:?}", miner_path);
        
        let executor = PythonExecutor::new(
            self.subnet_name.clone(),
            "subnet".to_string(),
            miner_path.to_str().unwrap().to_string(),
        )?;

        println!("Executing Python command...");
        let output = match args {
            Some(arg_str) => executor.run_command(arg_str.to_string())?,
            None => executor.run_command(String::new())?,
        };
        println!("Raw output from Python execution: {:?}", output);

        if output.trim().is_empty() {
            println!("Warning: The miner produced no output.");
        } else {
            println!("miner output: {}", output);
        }

        Ok(())
    }

    /// Replaces the forward function in the miner script.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the replacement.
    pub fn replace_forward(&self) -> Result<(), Box<dyn Error>> {
        let miner_path = self.miner_path.as_ref().ok_or("Miner script not found")?;
        println!("Attempting to replace forward function in: {:?}", miner_path);
        
        let content = fs::read_to_string(miner_path)?;
        println!("File content read successfully. Length: {} characters", content.len());

        let inference_type = self.identify_inference_type()?;
        let re = Regex::new(r"(?s)def forward\(.*?\):.*?(\n\S|\z)")?;

        let new_forward = format!(r#"def forward(self, x: dict) -> dict:
    return subprocess.run(["python", "modules/{}/{}.py"], capture_output=True, text=True)
"#, inference_type, inference_type);

        let new_content = re.replace_all(&content, |caps: &regex::Captures| {
            format!("{}{}", new_forward, &caps[1])
        }).to_string();

        println!("New content created. Length: {} characters", new_content.len());

        fs::write(miner_path, new_content)?;
        println!("Forward function replaced in miner script");

        Ok(())
    }
}
