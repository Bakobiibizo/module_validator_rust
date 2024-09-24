use std::error::Error;
use std::process::Command;
use url::Url;
use std::path::PathBuf;

pub struct SubnetModule {
    pub name: String,
    pub url: String,
}

impl SubnetModule {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let parsed_url = Url::parse(url)?;
        let name = parsed_url.path_segments()
            .and_then(|segments| segments.last())
            .ok_or("Invalid URL: cannot extract subnet name")?
            .to_string();

        Ok(SubnetModule { name, url: url.to_string() })
    }

    pub async fn install(&self) -> Result<(), Box<dyn Error>> {
        println!("Installing subnet module: {}", self.name);

        let module_dir = PathBuf::from("subnets").join(&self.name);

        // Clone the repository
        let output = Command::new("git")
            .args(&["clone", &self.url, &module_dir.to_string_lossy()])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to clone repository: {}", error).into());
        }

        println!("Repository cloned successfully");

        // Run setup script if it exists
        let setup_script = module_dir.join("setup.sh");
        if setup_script.exists() {
            let output = Command::new("bash")
                .arg(&setup_script)
                .current_dir(&module_dir)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to run setup script: {}", error).into());
            }

            println!("Setup script executed successfully");
        }

        // Install Python requirements if requirements.txt exists
        let requirements_file = module_dir.join("requirements.txt");
        if requirements_file.exists() {
            let output = Command::new("pip")
                .args(&["install", "-r", "requirements.txt"])
                .current_dir(&module_dir)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to install Python requirements: {}", error).into());
            }

            println!("Python requirements installed successfully");
        }

        println!("Subnet module installed successfully");
        Ok(())
    }
}