use std::error::Error;
use tokio;
use pyo3::prelude::*;
use std::env;

mod subnet_module;
use subnet_module::SubnetModule;

#[derive(Debug)]
pub struct CustomError(String);

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CustomError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get the current directory as the root path
    let root_path = env::current_dir()?;

    // Perform async operations
    let module = SubnetModule::new(
        "sylliba_subnet",
        "https://github.com/example/sylliba_subnet",
        root_path.to_str().unwrap()
    );
    module.install().await?;

    // Perform sync operations in a separate thread
    tokio::task::spawn_blocking(|| {
        handle_python_operations()
    }).await??;

    Ok(())
}

fn handle_python_operations() -> Result<(), CustomError> {
    Python::with_gil(|py| -> PyResult<()> {
        // Set up the Python path
        let sys = py.import("sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", (".",))?;

        // Import the module
        let _module = py.import("modules.sylliba_subnet.module_wrapper")?;
        
        // Use the module...

        Ok(())
    }).map_err(|e| CustomError(e.to_string()))
}