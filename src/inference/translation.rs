use std::{borrow::BorrowMut, env, future::IntoFuture, mem, process::Command};

use crate::inference::python_executor::{self, PythonExecutor};

#[cfg(windows)]
use winapi;

pub struct TranslationAPI {
    executor: PythonExecutor,
    module_path: String,
    target_script: String,
    host_ip: String,
    port: String
}

impl TranslationAPI {
    pub fn new() -> Self {

        let module_path = "modules/translation".to_string();
        let target_script = "translation_api.py".to_string();
        let module_name = "translation".to_string();
        let inference_type = "inference".to_string();
        let host_ip = env::var("TRANSLATION_API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("TRANSLATION_API_PORT").unwrap_or_else(|_| "8090".to_string());
        let executor = PythonExecutor::new(
            module_name,
            inference_type,
            target_script.clone()
        ).unwrap();

        Self {
            executor,
            module_path,
            target_script,
            host_ip,
            port
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::process::{Command, Child};
        

        // Get the host and port from environment variables
        let host = env::var("TRANSLATION_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("TRANSLATION_API_PORT").unwrap_or_else(|_| "8070".to_string());

        // Construct the command
        let mut cmd = Command::new("pm2");
        cmd.args(&[
            "start",
            format!("python -m modules.translation.translation_api --port {} --host {}", &port, &host).as_str()
        ]);

        // Spawn the child process
        let child = Command::new("your_command")
            .args(&["arg1", "arg2"])
            .spawn()
            .expect("Failed to spawn child process");

        // On Windows, disassociate the child process from the parent's console
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            child.spawn_config.creation_flags |= winapi::um::winbase::DETACHED_PROCESS;
        }

        // Don't wait for the child process
        std::mem::forget(child);

        println!("Translation API started on {}:{}", host, port);

        Ok(())
    }

    pub fn start_with_pm2(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;

        let result = self.executor.source_env()?;
        println!("{}", result);
        

        // Get the host and port from environment variables
        let host = env::var("TRANSLATION_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("TRANSLATION_API_PORT").unwrap_or_else(|_| "8070".to_string());

        // Construct the command
        let mut cmd = Command::new("pm2");
        cmd.args(&[
            "start",
            "python",
            "-n", "translation",
            "--",
            "-m", "modules.translation.translation_api",
            "--port", &port,
            "--host", &host,
        ]);

        // Spawn the child process
        let child = cmd.spawn()?;

        // On Windows, disassociate the child process from the parent's console
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(winapi::um::winbase::DETACHED_PROCESS);
        }

        // Don't wait for the child process
        std::mem::forget(child);

        println!("Translation API started on {}:{}", host, port);

        Ok(())
    }
}