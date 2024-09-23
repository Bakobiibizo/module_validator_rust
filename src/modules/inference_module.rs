use reqwest;
use std::error::Error;
use std::fs;
use std::process::Command;

pub struct InferenceModule {
    name: String,
}

impl InferenceModule {
    pub fn new(name: &str) -> Self {
        InferenceModule {
            name: name.to_string(),
        }
    }

    pub fn install(&self) -> Result<(), Box<dyn Error>> {
        println!("Installing inference module: {}", self.name);

        let url = format!("https://registrar-cellium.ngrok.app/api/{}", self.name);
        let response = reqwest::blocking::get(&url)?.text()?;
        // Save the script
        let file_name = format!("setup_{}.sh", self.name);
        let script_path = format!("./modules/{}/{}", self.name, &file_name);
        fs::create_dir_all(format!("./modules/{}", self.name))?;
        fs::write(&script_path, response)?;

        // Make the script executable
        Command::new("chmod")
            .args(&["+x", &script_path])
            .output()?;

        // Execute the script
        Command::new("bash")
            .arg(&script_path)
            .output()?;

        Ok(())
    }
}