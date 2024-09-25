# Module Validator

Module Validator is a Rust-based application that manages and executes Python modules dynamically. It provides a flexible system for installing, loading, and running Python modules within a Rust environment.

## Features

- Dynamic installation of Python modules
- Automatic creation of Python virtual environments
- Module management through a registry system
- Database integration for persistent module information
- Cross-platform support (Linux, macOS, Windows)
- Support for both subnet and inference modules
- Command-line interface for easy interaction

## Prerequisites

- Rust (latest stable version)
- Python 3.7+
- PostgreSQL

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/module-validator.git
   cd module-validator
   ```

2. Set up the database:
   ```
   ./scripts/setup_database.sh
   ```

3. Build the project:
   ```
   cargo build --release
   ```

## Usage

The application provides a command-line interface with the following commands:

- Install a module: `cargo run -- install <URL>`
- List installed modules: `cargo run -- list`
- Run inference: `cargo run -- run-inference <MODULE_NAME> <INPUT>`
- Uninstall a module: `cargo run -- uninstall <MODULE_NAME>`
- Parse module config: `cargo run -- parse-config <MODULE_NAME>`
- Launch validator: `cargo run -- launch-validator <SUBNET_NAME>`

Example:
```
cargo run -- install https://github.com/example/module.git
cargo run -- list
cargo run -- run-inference translation "Hello, world!"
```

## Project Structure

- `src/`: Contains the Rust source code
  - `main.rs`: Entry point of the application
  - `lib.rs`: Library root, exports public modules
  - `modules/`: Contains the InferenceModule and SubnetModule implementations
  - `registry.rs`: Implements the ModuleRegistry for managing modules
  - `config.rs`: Handles configuration loading
  - `database.rs`: Handles database operations
  - `validator.rs`: Implements the Validator for subnet modules
- `tests/`: Contains integration tests
- `scripts/`: Contains utility scripts for setup and maintenance

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.