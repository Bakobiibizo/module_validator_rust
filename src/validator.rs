use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

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

    pub fn identify_and_replace_inference(
        &mut self,
        args: &String,
    ) -> Result<(), Box<dyn Error>> {
        println!("Identifying and replacing inference");
        println!("Module dir: {:?}", self.module_dir);
        
        let script_path = self.validator_path.as_ref().ok_or("Validator script not found")?;
        println!("Script path: {:?}", script_path);

        let new_script_path_str = script_path.to_str().unwrap().replace(&format!("subnets/{}/", self.subnet_name), "");
        self.validator_path = Some(PathBuf::from(new_script_path_str));
        
        let inference_command = format!(
            "def forward():\n    import subprocess\n    result = subprocess.run(['python3', '{}', '{}'], capture_output=True, text=True)\n    return result.stdout",
            self.validator_path.as_ref().unwrap().to_str().unwrap(),
            args
        );
        
        println!("Validator path: {}", self.validator_path.as_ref().unwrap().to_str().unwrap());
        let forward_regex = Regex::new(r"(?s)def\s+forward\s*\([^)]*\)\s*:(.*?)(?:\n\S|\z)")?;

        let file_content = fs::read_to_string(script_path)?;

        if let Some(captures) = forward_regex.captures(&file_content) {
            let full_match = captures.get(0).unwrap().as_str();
            println!("Replacing: {}\nWith: {}", full_match, inference_command);
            let modified_content = file_content.replace(full_match, &inference_command);
            fs::write(script_path, modified_content)?;
            Ok(())
        } else {
            Err("Could not find the forward function in the validator script".into())
        }
    }

    pub fn launch(
        &self,
        args: &String,
    ) -> Result<(), Box<dyn Error>> {
        println!("Launching validator for subnet: {}", self.subnet_name);
        let validator_path = self.validator_path.as_ref().ok_or("Validator path not set")?;
        let executor = PythonExecutor::new(
            self.subnet_name.clone(),
            "subnets".to_string(),
            validator_path.to_str().unwrap().to_string()
        )?;
        let output = executor.run_command(args.to_string())?;
        println!("Validator output: {}", output);
        Ok(())
    }
}
