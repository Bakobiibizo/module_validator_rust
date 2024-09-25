use std::error::Error;
use std::process::Command;
use std::path::PathBuf;

/// Represents a validator for subnet modules.
pub struct Validator {
    subnet_name: String,
    env_dir: PathBuf,
    module_dir: PathBuf,
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
    pub fn new(subnet_name: &str) -> Self {
        let env_dir = PathBuf::from(format!(".{}", subnet_name));
        let module_dir = PathBuf::from("modules").join(subnet_name);
        Self {
            subnet_name: subnet_name.to_string(),
            env_dir,
            module_dir,
        }
    }

    /// Launches the validator for the subnet module.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the validator launch.
    pub fn launch(&self) -> Result<(), Box<dyn Error>> {
        println!("Launching validator for subnet: {}", self.subnet_name);

        // Activate the virtual environment
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
                "source {} && PYTHONPATH={} python neurons/validator.py --logging.debug",
                activate_script.to_str().unwrap(),
                new_path
            ))
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to launch validator: {}", error).into());
        }

        println!("Validator launched successfully");
        Ok(())
    }
}