use std::path::PathBuf;
use std::process::Command;
use std::error::Error;
use std::env;
use std::collections::HashMap;

pub struct PythonExecutor {
    venv_path: PathBuf,
    stored_env: Option<HashMap<String, String>>,
    pub python: String,
    active_module_dir: PathBuf,
    target_script_path: PathBuf,
}

impl PythonExecutor {
    pub fn new(active_module_name: String, active_module_type: String, target_script_path: String) -> Result<Self, Box<dyn Error>> {
        let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
        let venv_path = root_dir.join(format!(".{}", &active_module_name));
        let python = activate_env(&venv_path)?;
        
        let active_module_dir = if active_module_type == "inference" {
            root_dir.join("modules").join(&active_module_name)
        } else {
            root_dir.join("subnets").join(&active_module_name)
        };
        
        let target_script_path = if active_module_type == "inference" {
            active_module_dir.join(format!("{}.py", &active_module_name))
        } else {
            PathBuf::from(target_script_path)
        };
        let mut executor = Self {
            venv_path,
            stored_env: None,
            python,
            active_module_dir,
            target_script_path,
        };
        executor.source_env()?;
        Ok(executor)
    }

    pub fn run_command(&self, args: String) -> Result<String, Box<dyn Error>> {
        let target_script_path = self.target_script_path.to_str().unwrap().replace(".py", "").replace("/", ".");
        let command_str = if cfg!(windows) {
            format!("{} && {} -m {}", 
                self.venv_path.join("Scripts").join("activate.bat").to_str().unwrap(),
                &self.python,
                target_script_path)
        } else {
            format!("source {} && {} -m {}", 
                self.venv_path.join("bin").join("activate").to_str().unwrap(),
                &self.python,
                target_script_path)
        };

        println!("Executing command: {}", command_str);

        let mut command = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", &command_str]);
            cmd
        } else {
            let mut cmd = Command::new("bash");
            cmd.args(&["-c", &command_str]);
            cmd
        };

        command.args([&args])
               .current_dir(&self.active_module_dir);

        if let Some(env_vars) = &self.stored_env {
            for (key, value) in env_vars {
                command.env(key, value);
                println!("Debug: Applied env var: {}={}", key, value);
            }
        } else {
            println!("Debug: No stored environment variables found");
        }

        let output = command.output()?;
        println!("Command exit status: {}", output.status);
        println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(format!("Failed to run inference: {}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }

    pub fn source_env(&mut self) -> Result<String, std::io::Error> {
        let activate_script = if cfg!(windows) {
            self.venv_path.join("Scripts").join("activate.bat")
        } else {
            self.venv_path.join("bin").join("activate")
        };
        let dot_env_path = self.active_module_dir.to_str().unwrap();
        let command_str = if cfg!(windows) {
            format!("{} && set", activate_script.to_str().unwrap())
        } else {
            format!("source {} && cat {}/.env", activate_script.to_str().unwrap(), dot_env_path)
        };

        let output = if cfg!(windows) {
            Command::new("cmd")
                .args(&["/C", &command_str])
                .output()?
        } else {
            Command::new("bash")
                .arg("-c")
                .arg(&command_str)
                .output()?
        };

        if output.status.success() {
            let env_output = String::from_utf8_lossy(&output.stdout);
            let mut env_vars = HashMap::new();
    
            for line in env_output.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    env_vars.insert(key.to_string(), value.to_string());
                    println!("Debug: Captured env var: {}={}", key, value);  // Debug line
                }
            }
    
            self.stored_env = Some(env_vars);
            println!("Successfully activated");
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to source environment",
            ))
        }
    }
}

pub fn activate_env(venv_path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));

    if !venv_path.exists() {
        println!("Creating virtual environment at {:?}", venv_path);
        let output = Command::new("python3")
            .args(&["-m", "venv", venv_path.to_str().unwrap()])
            .current_dir(&root_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to create virtual environment: {}", error).into());
        }
        println!("Created Python virtual environment");
    }

    let python_executable = venv_path.join("bin").join("python3");

    // Upgrade pip
    let output = Command::new(&python_executable)
        .args(&["-m", "pip", "install", "--upgrade", "pip"])
        .current_dir(&root_dir)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to upgrade pip: {}", error).into());
    }

    Ok(python_executable.to_string_lossy().into_owned())
}

pub fn install_requirements(venv_path: &PathBuf, python_executable: &str) -> Result<(), Box<dyn Error>> {
    let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let module_name = venv_path.file_name().unwrap().to_str().unwrap().trim_start_matches('.');
    let mut command = Command::new(python_executable);

    let module_path = if PathBuf::from("modules").join(module_name).exists() {
        PathBuf::from("modules").join(module_name)
    } else if PathBuf::from("subnets").join(module_name).exists() {
        PathBuf::from("subnets").join(module_name)
    } else {
        println!("No module found for module: {}", module_name);
        return Ok(());
    };

    let output = command.args(["-m", "pip", "install", "-e", module_path.to_str().unwrap()])
        .current_dir(&root_dir)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to install {}: {}", module_name, error).into());
    }
    println!("Installed {} in virtual environment", module_name);

    let requirements = module_path.join("requirements.txt");

    if !requirements.exists() {
        println!("requirements.txt not found at {:?}", requirements);
    }


    println!("Installing requirements from {:?}", requirements);
    let output = Command::new(python_executable)
        .args(&["-m", "pip", "install", "-r", requirements.to_str().unwrap()])
        .current_dir(&root_dir)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to install {}: {}", module_name, error).into());
    }
    println!("Installed {} in virtual environment", module_name);

    Ok(())
}
