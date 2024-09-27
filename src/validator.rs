use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use crate::inference::python_executor::PythonExecutor;

pub struct Validator {
    subnet_name: String,
    env_dir: PathBuf,
    module_dir: PathBuf,
    validator_path: Option<PathBuf>,
}

impl Validator {
    pub fn new(subnet_name: &str) -> Result<Self, Box<dyn Error>> {
        println!("Creating new validator for subnet: {}", subnet_name);
        let env_dir = PathBuf::from(format!(".{}", subnet_name));
        let module_dir = PathBuf::from("subnets").join(subnet_name);
        
        let mut validator = Self {
            subnet_name: subnet_name.to_string(),
            env_dir,
            module_dir,
            validator_path: None,
        };

        validator.find_validator_script()?;
        Ok(validator)
    }

    #[allow(dead_code)]
    pub fn prompt_user_for_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut validator_path = String::new();
        println!("Enter the path to the validator script: ");
        std::io::stdin().read_line(&mut validator_path).unwrap();
        let validator_path = PathBuf::from(validator_path.trim());
        Ok(validator_path)
    }

    fn find_validator_script(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Finding validator script in: {:?}", self.module_dir);
        fn find_script(module_dir: &PathBuf) -> Option<PathBuf> {
            if let Ok(entries) = fs::read_dir(module_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.file_name().unwrap() == "validator.py" {
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
            self.validator_path = Some(script_path);
            Ok(())
        } else {
            Err("Could not find the validator script".into())
        }
    }

    pub fn identify_and_prepare_inference(&mut self, _args: &String) -> Result<(), Box<dyn Error>> {
        println!("Preparing inference for subnet: {}", self.subnet_name);
        
        let script_path = self.validator_path.as_ref().ok_or("Validator script not found")?;
        println!("Original script path: {:?}", script_path);

        // Adjust the path to be relative to the subnet folder
        let relative_script_path = script_path.strip_prefix(&self.module_dir)?;
        self.validator_path = Some(relative_script_path.to_path_buf());
        
        println!("Adjusted validator path: {:?}", self.validator_path);
        Ok(())
    }

    pub fn launch(&self, args: Option<&String>) -> Result<(), Box<dyn Error>> {
        println!("Launching validator for subnet: {}", self.subnet_name);
        let validator_path = self.validator_path.as_ref().ok_or("Validator path not set")?;
        
        println!("Validator path: {:?}", validator_path);
        
        let executor = PythonExecutor::new(
            self.subnet_name.clone(),
            "subnet".to_string(),
            validator_path.to_str().unwrap().to_string(),
        )?;

        println!("Executing Python command...");
        let output = match args {
            Some(arg_str) => executor.run_command(arg_str.to_string())?,
            None => executor.run_command(String::new())?,
        };
        println!("Raw output from Python execution: {:?}", output);

        if output.trim().is_empty() {
            println!("Warning: The validator produced no output.");
        } else {
            println!("Validator output: {}", output);
        }

        Ok(())
    }
}
