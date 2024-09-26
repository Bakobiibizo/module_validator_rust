use std::error::Error;
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use regex::Regex;


use crate::inference::python_executor::PythonExecutor;

/// Represents a validator for subnet modules.
pub struct Validator {
    subnet_name: String,
    env_dir: PathBuf,
    module_dir: PathBuf,
    validator_path: PathBuf
}

impl Validator {
    /// Creates a new Validator instance.
    ///
    /// # Arguments
    ///
    /// * `subnet_name` - The name of the subnet module to validate.
    ///
    /// # Returns
    ///
    /// A new Validator instance.
    pub fn new(subnet_name: &str) -> Result<Self, Box<dyn Error>> {
        let env_dir = PathBuf::from(format!(".{}", subnet_name));
        let module_dir = PathBuf::from("subnets").join(subnet_name);
        
        Ok(Self {
            subnet_name: subnet_name.to_string(),
            env_dir,
            module_dir,
            validator_path: PathBuf::new()
        })
    }

    fn parse_validator_script(module_dir: &PathBuf) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(module_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.file_name().unwrap() == "validator.py" {
                    return Some(path);
                } else if path.is_dir() {
                    if let Some(found_path) = Self::parse_validator_script(&path) {
                        println!("Validator path: {:?}", found_path);
                        return Some(found_path);
                    }
                }
            }
        }
        println!("No validator script found in {:?}", module_dir);
        None

    }

    // Prompt user for validator_path
    fn prompt_user_for_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut validator_path = String::new();
        println!("Enter the path to the validator script: ");
        std::io::stdin().read_line(&mut validator_path).unwrap();
        let validator_path = PathBuf::from(validator_path.trim());
        Ok(validator_path)
    }
    /// Launches the validator for the subnet module.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the validator launch.
    pub fn launch(&mut self, args: Vec<String>) -> Result<(), Box<dyn Error>> {
        println!("Launching validator for subnet: {}", self.subnet_name);
        self.validator_path = Self::parse_validator_script(&self.module_dir).unwrap_or_else(|| self.prompt_user_for_path().unwrap());
        if self.validator_path.exists() {
            return Err(format!("Validator script not found at: {}", self.validator_path.display()).into());
        }
        let inference_command = format!("def forward():\n    import subprocess\n    result = subprocess.run(['python3', '{}', '{}'], capture_output=True, text=True)\n    return result.stdout", self.validator_path.to_str().unwrap(), args[0]);
        self.identify_and_replace_inference(&inference_command)?;
        self.execute_modified_script()?;

        println!("Validator launched successfully");
        Ok(())
    }

    fn identify_and_replace_inference(&mut self, inference_command: &str) -> Result<(), Box<dyn Error>> {
        let forward_regex = Regex::new(r"(?s)def\s+forward\s*\([^)]*\)\s*:(.*?)(?:\n\S|\z)")?;
        
        let file_content = fs::read_to_string(&self.validator_path)?;
        
        if let Some(captures) = forward_regex.captures(&file_content) {
            let full_match = captures.get(0).unwrap().as_str();
            let modified_content = file_content.replace(full_match, inference_command);
            fs::write(&self.validator_path, modified_content)?;
            Ok(())
        } else {
            Err("Could not find the forward function in the validator script".into())
        }
    }

    fn execute_modified_script(&self) -> Result<(), Box<dyn Error>> {
        let temp_script_path = self.module_dir.join("modified_validator.py");
        fs::write(&temp_script_path, self.validator_path.to_str().unwrap())?;

        let python_executor = PythonExecutor::new(self.subnet_name.clone(), "subnet".to_string(), , self.validator_path, )

        // Add the module directory to PYTHONPATH
        let current_path = std::env::var("PYTHONPATH").unwrap_or_default();
        let new_path = format!("{}:{}", self.module_dir.display(), current_path);

        // Run the validator script
        let output = Command::new("bash")
            .arg("-c")
            .arg(format!(
                "source {} && PYTHONPATH={} python {} --logging.debug",
                activate_script.to_str().unwrap(),
                new_path,
                temp_script_path.to_str().unwrap()
            ))
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to execute modified validator script: {}", error).into());
        }

        fs::remove_file(temp_script_path)?;

        println!("Modified validator script executed successfully");
        Ok(())
    }
}