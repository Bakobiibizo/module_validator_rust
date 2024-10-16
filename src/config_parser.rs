use std::fs;
use std::path::Path;
use std::error::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use dialoguer::Input;
use std::io::Write;

/// Represents the configuration of an argument in a command.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArgConfig {
    pub type_: String,
    pub default: Option<String>,
    pub help: Option<String>,
}

/// Represents the configuration of a command.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandConfig {
    pub function: String,
    pub args: HashMap<String, ArgConfig>,
}

/// Represents the overall configuration of a module.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub env_vars: HashMap<String, String>,
    pub commands: HashMap<String, CommandConfig>,
}

/// Provides functionality for parsing and manipulating module configurations.
pub struct ConfigParser;

    /// Parses the commands and environment variables from a module's directory.
    ///
    /// # Arguments
    ///
    /// * `file_dir` - The directory containing the module's files.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed ModuleConfig if successful, or an error if parsing fails.
impl ConfigParser {
    pub fn parse_commands(file_dir: &Path) -> Result<ModuleConfig, Box<dyn Error>> {
        println!("Parsing commands from directory: {:?}", file_dir);
        
        let mut config = ModuleConfig {
            env_vars: HashMap::new(),
            commands: HashMap::new(),
        };

        // Parse .env file
        let env_example_file = file_dir.join(".env.example");
        println!("Checking for .env file: {:?}", env_example_file);
        if env_example_file.exists() {
            println!(".env.example file found, parsing...");
            let env_content = fs::read_to_string(&env_example_file)?;
            for line in env_content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    config.env_vars.insert(key.trim().to_string(), value.trim().to_string());
                    println!("Added env var: {} = {}", key.trim(), value.trim());
                }
            }
        } else {
            println!(".env.example file not found");
        }

        // Parse Python files
        println!("Searching for Python files in {:?}", file_dir);
        for entry in fs::read_dir(file_dir)? {
            let entry = entry?;
            let path = entry.path();
            println!("Examining file: {:?}", path);
            if path.is_file() && path.extension().map_or(false, |ext| ext == "py") {
                println!("Parsing Python file: {:?}", path);
                Self::parse_python_file(&path, &mut config)?;
            } else {
                println!("Skipping non-Python file: {:?}", path);
            }
        }

        println!("Parsing complete. Found {} env vars and {} commands", 

                 config.env_vars.len(), config.commands.len());

        Ok(config)
    }

    fn parse_python_file(file_path: &Path, config: &mut ModuleConfig) -> Result<(), Box<dyn Error>> {
        println!("Parsing Python file: {:?}", file_path);
        let content = fs::read_to_string(file_path)?;

        // Parse argparse arguments
        let argparse_regex = Regex::new(r#"parser\.add_argument\(['"](--[\w-]+)['"].*?(?:default=(.*?))?(?:,|\))"#)?;
        for cap in argparse_regex.captures_iter(&content) {
            let key = cap[1].trim_start_matches("--").replace("-", "_");
            let default = cap.get(2).map(|m| m.as_str().trim().trim_matches(|c| c == '\'' || c == '"').to_string());
            config.env_vars.insert(key.clone(), default.clone().unwrap_or_default());
            println!("Found argparse argument: {} = {:?}", key, default);
        }

        // Parse typer commands
        let typer_regex = Regex::new(r#"@app\.command\(['"]([\w-]+)['"].*?\)\s*def\s+(\w+)\((.*?)\):"#)?;
        for cap in typer_regex.captures_iter(&content) {
            let command_name = cap[1].to_string();
            let function_name = cap[2].to_string();
            let args = &cap[3];

            println!("Found typer command: {} (function: {})", command_name, function_name);

            let mut command_config = CommandConfig {
                function: function_name,
                args: HashMap::new(),
            };

            let arg_regex = Regex::new(r#"(\w+):\s*(?:Optional\[)?(\w+)(?:\])?\s*=\s*(?:typer\.(?:Argument|Option)\((.*?)\))?"#)?;
            for arg_cap in arg_regex.captures_iter(args) {
                let arg_name = arg_cap[1].to_string();
                let arg_type = arg_cap[2].to_string();
                let arg_options = arg_cap.get(3).map(|m| m.as_str()).unwrap_or("");

                println!("  Argument: {} (type: {})", arg_name, arg_type);

                let mut arg_config = ArgConfig {
                    type_: arg_type,
                    default: None,
                    help: None,
                };

                if !arg_options.is_empty() {
                    // Parse additional options like default value, help text, etc.
                    let default_regex = Regex::new(r#"default\s*=\s*['"](.*?)['"]"#)?;
                    if let Some(default_cap) = default_regex.captures(arg_options) {
                        arg_config.default = Some(default_cap[1].to_string());
                        println!("    Default value: {:?}", arg_config.default);
                    }

                    let help_regex = Regex::new(r#"help\s*=\s*['"](.*?)['"]"#)?;
                    if let Some(help_cap) = help_regex.captures(arg_options) {
                        arg_config.help = Some(help_cap[1].to_string());
                        println!("    Help text: {:?}", arg_config.help);
                    }
                }

                command_config.args.insert(arg_name, arg_config);
            }

            config.commands.insert(command_name, command_config);
        }
        // Self::prompt_for_env_vars(config)?;

        println!("Finished parsing file: {:?}", file_path);
        Ok(())
    }

    /// Prompts the user for values for environment variables.
    ///
    /// # Arguments
    ///
    /// * `config` - The ModuleConfig containing the environment variables to prompt for.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the prompting operation.
    pub fn prompt_for_env_vars(config: &mut ModuleConfig) -> Result<(), Box<dyn Error>> {
        for (key, value) in &mut config.env_vars {
            let default = value.clone();
            let prompt = format!("Enter value for {} (default: {})", key, default);
            let input: String = Input::new()
                .with_prompt(&prompt)
                .default(default.clone())
                .interact_text()?;
            *value = input;
        }
        Ok(())
    }

    pub fn save_config(config: &ModuleConfig, module_dir: &Path) -> Result<(), Box<dyn Error>> {
        let env_file_path = module_dir.join(".env");
        if !env_file_path.exists() {
            let mut save_file = fs::File::create(env_file_path)?;
            for (key, value) in &config.env_vars {
                writeln!(save_file, "{}={}", key, value)?;
            }
        } else {
            let mut save_file = fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(env_file_path)?;
            for (key, value) in &config.env_vars {
                writeln!(save_file, "{}={}", key, value)?;
            }
        }
        println!("Configuration saved to: {:?}", module_dir.join(".env"));
        Ok(())
    }
}
