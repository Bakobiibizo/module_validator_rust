# Module Validator

Module Validator is a Rust-based application that manages and executes Python modules dynamically. It provides a flexible system for installing, loading, and running Python modules within a Rust environment.

## Features

- Dynamic installation of Python modules
- Automatic creation of Python virtual environments
- Module management through a registry system
- Database integration for persistent module information
- Cross-platform support (Linux, macOS, Windows)

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

1. Run the application:
   ```
   cargo run --release
   ```

2. The application will automatically install the translation module and run a test process.

## Project Structure

- `src/`: Contains the Rust source code
  - `main.rs`: Entry point of the application
  - `lib.rs`: Library root, exports public modules
  - `modules/`: Contains the InferenceModule implementation
  - `registry.rs`: Implements the ModuleRegistry for managing modules
  - `config.rs`: Handles configuration loading
- `modules/`: Contains Python modules and wrappers
- `scripts/`: Contains utility scripts for setup and maintenance

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.