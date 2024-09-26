use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::inference::python_executor::PythonExecutor;

pub struct Validator {
    subnet_name: String,
    env_dir: PathBuf,
    module_dir: PathBuf,
    validator_path: PathBuf,
}

impl Validator {
    pub fn new(subnet_name: &str) -> Result<Self, Box<dyn Error>> {
        let env_dir = PathBuf::from(format!(".{}", subnet_name));
        let module_dir = PathBuf::from("subnets").join(subnet_name);

        Ok(Self {
            subnet_name: subnet_name.to_string(),
            env_dir,
            module_dir,
            validator_path: PathBuf::new(),
        })
    }

    pub fn prompt_user_for_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut validator_path = String::new();
        println!("Enter the path to the validator script: ");
        std::io::stdin().read_line(&mut validator_path).unwrap();
        let validator_path = PathBuf::from(validator_path.trim());
        Ok(validator_path)
    }

    pub fn find_validator_script(&self, module_dir: &PathBuf) -> Option<PathBuf> {
        fn find_script(module_dir: &PathBuf) -> Option<PathBuf> {
            if let Ok(entries) = fs::read_dir(module_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.file_name().unwrap() == "validator.py" {
                        println!("Found validator script: {}", path.to_str().unwrap());
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
        if let Some(script_path) = find_script(module_dir) {
            return Some(script_path);
        }
        None
    }

    pub fn identify_and_replace_inference(
        &mut self,
        inference_command: &str,
    ) -> Result<(), Box<dyn Error>> {
        let script_path = self.find_validator_script(&self.module_dir).unwrap();

        let forward_regex = Regex::new(r"(?s)def\s+forward\s*\([^)]*\)\s*:(.*?)(?:\n\S|\z)")?;

        let file_content = fs::read_to_string(&script_path)?;

        self.validator_path = PathBuf::from(script_path.to_str().unwrap().replace("subnets/{}/", ""));

        if let Some(captures) = forward_regex.captures(&file_content) {
            let full_match = captures.get(0).unwrap().as_str();
            println!("Replacing: {}\nWith: {}", full_match, inference_command);
            let modified_content = file_content.replace(full_match, inference_command);
            fs::write(&script_path, modified_content)?;
            Ok(())
        } else {
            Err("Could not find the forward function in the validator script".into())
        }

    }

    pub fn launch(
        &mut self,
        args: Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        println!("Launching validator for subnet: {}", self.subnet_name);

        let inference_command = format!("def forward():\n        import subprocess\n        result = subprocess.run(['python3', '{}', '{}'], capture_output=True, text=True)\n        return result.stdout", self.validator_path.to_str().unwrap(), args[0]);
        self.identify_and_replace_inference(&inference_command)?;

        if !self.validator_path.exists() {
            println!("Validator path: {}", self.validator_path.to_str().unwrap());
            self.validator_path = self.prompt_user_for_path()?;
        }
        //if script_path.contains(self.module_dir.to_str().unwrap()) {
        //    let local_path = script_path.replace(self.module_dir.to_str().unwrap(), "");
        //    self.validator_path = PathBuf::from(local_path);
        //} else {
        //    self.validator_path = PathBuf::from(script_path);
        //}
        //self.validator_path = Self::parse_validator_script(&PathBuf::from(script_path)).unwrap_or_else(|| self.prompt_user_for_path().unwrap());
        //if !self.validator_path.exists() {
        //    self.validator_path = PathBuf::from(script_path);
        //}
        //if !self.validator_path.exists() {
        //    return Err(format!("Validator script not found at: {}", self.validator_path.display()).into());
        //}

        //self.execute_modified_script()?;

        println!("Validator launched successfully");
        Ok(())
    }

    //pub fn execute_modified_script(&self) -> Result<(), Box<dyn Error>> {
    //    let script_contents = fs::read_to_string(&self.validator_path)?;
    //    let temp_script_path = self.module_dir.join("modified_validator.py");
    //    fs::write(&temp_script_path, script_contents)?;
//
    //    let python_executor_result = PythonExecutor::new(
    //        self.subnet_name.clone(),
    //        "subnet".to_string(),
    //        temp_script_path.to_str().unwrap().to_string(),
    //    )
    //    .unwrap()
    //    .run_subnet_script(&temp_script_path)?;
//
    //    if !python_executor_result.is_empty() {
    //        let error = python_executor_result;
    //        return Err(format!("Failed to execute modified validator script: {}", error).into());
    //    }
//
    //    fs::remove_file(&temp_script_path)?;
//
    //    println!("Modified validator script executed successfully");
    //    Ok(())
    //}
}
