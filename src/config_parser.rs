use std::fs;
use std::path::Path;
use std::error::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use dialoguer::Input;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgConfig {
    pub type_: String,
    pub default: Option<String>,
    pub help: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandConfig {
    pub function: String,
    pub args: HashMap<String, ArgConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub env_vars: HashMap<String, String>,
    pub commands: HashMap<String, CommandConfig>,
}

pub struct ConfigParser;

impl ConfigParser {
    pub fn parse_commands(file_dir: &Path) -> Result<ModuleConfig, Box<dyn Error>> {
        let mut config = ModuleConfig {
            env_vars: HashMap::new(),
            commands: HashMap::new(),
        };

        // Parse .env.example file
        if let Ok(env_content) = fs::read_to_string(file_dir.join(".env.example")) {
            for line in env_content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    config.env_vars.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        // Parse Python files for additional configuration
        for entry in fs::read_dir(file_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "py") {
                Self::parse_python_file(&path, &mut config)?;
            }
        }

        Ok(config)
    }

    fn parse_python_file(file_path: &Path, config: &mut ModuleConfig) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;

        // Parse argparse arguments
        let argparse_regex = Regex::new(r#"parser\.add_argument\(['"](--[\w-]+)['"].*?(?:default=(.*?))?(?:,|\))"#)?;
        for cap in argparse_regex.captures_iter(&content) {
            let key = cap[1].trim_start_matches("--").replace("-", "_");
            let default = cap.get(2).map(|m| m.as_str().trim().trim_matches(|c| c == '\'' || c == '"').to_string());
            config.env_vars.insert(key, default.unwrap_or_default());
        }

        // Parse typer commands
        let typer_regex = Regex::new(r#"@app\.command\(['"]([\w-]+)['"].*?\)\s*def\s+(\w+)\((.*?)\):"#)?;
        for cap in typer_regex.captures_iter(&content) {
            let command_name = cap[1].to_string();
            let function_name = cap[2].to_string();
            let args = &cap[3];

            let mut command_config = CommandConfig {
                function: function_name,
                args: HashMap::new(),
            };

            let arg_regex = Regex::new(r#"(\w+):\s*(?:Optional\[)?(\w+)(?:\])?\s*=\s*(?:typer\.(?:Argument|Option)\((.*?)\))?"#)?;
            for arg_cap in arg_regex.captures_iter(args) {
                let arg_name = arg_cap[1].to_string();
                let arg_type = arg_cap[2].to_string();
                let arg_options = arg_cap.get(3).map(|m| m.as_str()).unwrap_or("");

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
                    }

                    let help_regex = Regex::new(r#"help\s*=\s*['"](.*?)['"]"#)?;
                    if let Some(help_cap) = help_regex.captures(arg_options) {
                        arg_config.help = Some(help_cap[1].to_string());
                    }
                }

                command_config.args.insert(arg_name, arg_config);
            }

            config.commands.insert(command_name, command_config);
        }

        Ok(())
    }

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
}