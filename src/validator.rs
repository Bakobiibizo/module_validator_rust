use std::error::Error;
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use regex::Regex;

/// Represents a validator for subnet modules.
pub struct Validator {
    subnet_name: String,
    env_dir: PathBuf,
    module_dir: PathBuf,
    validator_script: String,
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
        let validator_script = Self::parse_validator_script(&module_dir)?;

        Ok(Self {
            subnet_name: subnet_name.to_string(),
            env_dir,
            module_dir,
            validator_script,
        })
    }

    fn parse_validator_script(module_dir: &PathBuf) -> Result<String, Box<dyn Error>> {
        let validator_path = module_dir.join("eden_subnet").join("validator").join("validator.py");
        let script_content = fs::read_to_string(validator_path)?;
        Ok(script_content)
    }

    /// Launches the validator for the subnet module.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the validator launch.
    pub fn launch(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Launching validator for subnet: {}", self.subnet_name);

        self.identify_and_replace_inference()?;
        self.execute_modified_script()?;

        println!("Validator launched successfully");
        Ok(())
    }

    fn identify_and_replace_inference(&mut self) -> Result<(), Box<dyn Error>> {
        let inference_regex = Regex::new(r"def\s+forward\s*\([^)]*\)\s*:(?s).*?return")?;
        
        self.validator_script = inference_regex.replace(&self.validator_script, |caps: &regex::Captures| {
            let original = caps.get(0).unwrap().as_str();
            format!("{}\n        # Replaced inference call\n        result = self.custom_inference()\n        return", &original[..original.rfind('\n').unwrap_or(original.len())])
        }).to_string();

        Ok(())
    }

    fn execute_modified_script(&self) -> Result<(), Box<dyn Error>> {
        let temp_script_path = self.module_dir.join("modified_validator.py");
        fs::write(&temp_script_path, &self.validator_script)?;

        let activate_script = if cfg!(windows) {
            self.env_dir.join("Scripts").join("activate.bat")
        } else {
            self.env_dir.join("bin").join("activate")
        };

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