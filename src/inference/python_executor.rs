use std::path::PathBuf;
use std::process::Command;
use std::error::Error;
use std::env;
use std::fs;
use std::io::Write;

pub struct PythonExecutor {
    venv_path: PathBuf,
    python: String,
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

        println!("PythonExecutor initialized");
        Ok(Self {
            venv_path,
            python,
            active_module_dir,
            target_script_path,
        })
    }

    pub fn run_inference(&self, args: Vec<String>) -> Result<String, Box<dyn Error>> {
        let activate_script = if cfg!(windows) {
            self.venv_path.join("Scripts").join("activate.bat")
        } else {
            self.venv_path.join("bin").join("activate")
        };
        // Use the activate_env function to get the Python executable path
        let python = activate_env(&self.venv_path)?;

        let mut command = Command::new(&python);
        command.arg("-m")
               .arg(self.target_script_path.to_str().unwrap().replace("/", "."))
               .args(&args)
               .current_dir(&self.active_module_dir);

        println!("Executing command: {:?}", command);

        println!("Activate script path: {:?}", activate_script);

        let command_str = if cfg!(windows) {
            format!("{} && {} -m {}", 
                activate_script.to_str().unwrap(),
                &self.python,
                self.target_script_path.to_str().unwrap().replace("/", "."))
        } else {
            format!("source {} && {} -m {}", 
                activate_script.to_str().unwrap(),
                &self.python,
                self.target_script_path.to_str().unwrap().replace("/", "."))
        };

        println!("Executing command: {}", command_str);
        println!("Working directory: {:?}", &self.active_module_dir);

        let mut command = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", &command_str]);
            cmd
        } else {
            let mut cmd = Command::new("bash");
            cmd.args(&["-c", &command_str]);
            cmd
        };

        command.args(&args)
               .current_dir(&self.active_module_dir);

        let output = command.output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            println!("Error: {}", String::from_utf8_lossy(&output.stderr));
            Err(format!("Failed to run inference: {}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }

//    pub fn run_subnet_script(&self, temp_script_path: &PathBuf) -> Result<String, Box<dyn Error>> {
//        println!("Running subnet script at: {:?}", temp_script_path);
//        let mut command = Command::new(&self.python);
//        command.arg(temp_script_path.to_str().unwrap())
//               .current_dir(&self.active_module_dir);
//        let output = command.output()?;
//        if output.status.success() {
//            Ok(String::from_utf8(output.stdout)?)
//        } else {
//            println!("Error: {}", String::from_utf8_lossy(&output.stderr));
//            Err(format!("Failed to run subnet script: {}", String::from_utf8_lossy(&output.stderr)).into())
//        }
//    }
}
//
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
    let requirements = if PathBuf::from("modules").join(module_name).exists() {
        PathBuf::from("modules").join(module_name).join("requirements.txt")
    } else if PathBuf::from("subnets").join(module_name).exists() {
        PathBuf::from("subnets").join(module_name).join("requirements.txt")
    } else {
        println!("No requirements.txt found for module: {}", module_name);
        return Ok(());
    };

    let output = Command::new(python_executable)
        .args(&["-m", "pip", "install", "-r", requirements.to_str().unwrap()])
        .current_dir(&root_dir)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to install {}: {}", requirements.to_str().unwrap(), error).into());
    }
    println!("Installed {} in virtual environment", requirements.to_str().unwrap());

    Ok(())
}