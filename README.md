# Module Validator

Module Validator is a Rust-based application that manages the installation, execution, and validation of various modules, including inference modules and subnet modules. It provides a flexible system for dynamically managing Python modules within a Rust environment.

## Features

- Dynamic installation of Python modules (inference modules and subnet modules)
- Automatic creation of Python virtual environments for each module
- Module management through a registry system
- Database integration for persistent module information
- Command-line interface for easy interaction
- Subnet module validation
- Cross-platform support (Linux, macOS, Windows)

## Prerequisites

- Rust (latest stable version)
- Python 3.7+
- PostgreSQL
- Git (for cloning subnet repositories)

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/module-validator.git
   cd module-validator
   ```

2. Set up the database:
   - Create a PostgreSQL database
   - Set the `DATABASE_URL` environment variable in a `.env` file:
     ```
     DATABASE_URL=postgres://username:password@localhost/database_name
     ```

3. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the application using:

`cargo run --release -- COMMAND`

Available commands:

- `install <url>`: Install a new module (inference or subnet)
- `list`: List all installed modules
- `run-inference <name> <input>`: Run an inference module
- `uninstall <name>`: Uninstall a module
- `parse-config <name>`: Parse and display the configuration of an installed module
- `launch-validator <name> [args]`: Launch a validator for a subnet module

For more details on each command, use:

`cargo run --release -- help`

## Project Structure

- `src/`: Contains the Rust source code
  - `main.rs`: Entry point of the application
  - `lib.rs`: Library root, exports public modules
  - `cli.rs`: Defines the command-line interface
  - `config.rs`: Handles configuration loading and saving
  - `config_parser.rs`: Parses module configurations
  - `database.rs`: Manages database operations
  - `registry.rs`: Implements the ModuleRegistry for managing modules
  - `utils.rs`: Contains utility functions
  - `validator.rs`: Implements the Validator for subnet modules
  - `modules/`: Contains module implementations
    - `inference_module.rs`: Implements the InferenceModule
    - `subnet_module.rs`: Implements the SubnetModule
  - `inference/`: Contains inference-related implementations
    - `python_executor.rs`: Manages Python execution environments
    - `inference_requests.rs`: Defines structures for inference requests

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.