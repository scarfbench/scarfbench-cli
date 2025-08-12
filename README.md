# 🧣 Scarf CLI

A benchmark CLI tool for framework migration.

## Overview

SCARF is a command-line interface tool designed to facilitate benchmarking of agent-driven code transformation and framework migration. It provides utilities to initialize transformation stubs and run agents for refactoring between different frameworks.

## Features

- **Template Generation**: Create destination stubs with templates for target frameworks
- **Agent Execution**: Run transformation agents with custom agent scripts (provided via `--agent-executable`)
- **Flexible Configuration**: Support for custom output directories and extra arguments

## Installation (Global)
To install SCARF globally, you can use Cargo, the Rust package manager. This will allow you to run `scarf` from anywhere in your terminal.

### Prerequisites

Ensure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).
### Install SCARF
Once you have Rust and Cargo set up, you can install SCARF globally using the following
```bash
cargo install scarf
```

### CARGO Wrapper

I have included a `cargow` wrapper script to build this tool without installing cargo globally. This script is a simple wrapper around `cargo` that works on both Unix and Windows systems.

### Build from Source

```bash
git clone <repository-url>
cd scarf
./cargow clean build --release
```

The binary will be available at `target/release/scarf`.

## Usage

SCARF provides two main commands: `init` and `run`.

### Initialize Transformation Stub

Create a destination stub with templates for a target framework:

```bash
./target/release/scarf init --source-dir <SOURCE_DIR> --target-framework <FRAMEWORK> [--output-dir <OUTPUT_DIR>]
```

**Arguments:**
- `--source-dir`: Path to the source directory containing the original code
- `--target-framework`: Target framework to migrate to
- `--output-dir`: Output directory for generated templates (default: "generated")

**Example:**
```bash
./target/release/scarf init --source-dir ./my-app --target-framework react --output-dir ./output
```

### Run Transformation Agent

Execute an agent to perform code transformation:
```bash
./target/release/scarf run --agent <AGENT_PATH> --from <SOURCE_PATH> --to <TARGET_FRAMEWORK> [--out <OUTPUT_DIR>] [-- <EXTRA_ARGS>...]
```

**Arguments:**
- `--agent`: Path to the transformation agent
- `--from`: Source path containing code to transform
- `--to`: Target framework name
- `--out`: Output directory (default: "generated")
- `-- <EXTRA_ARGS>`: Additional arguments passed to the agent

**Example:**
```bash
./target/release/scarf run --agent ./agents/react-to-vue --from ./src --to vue -- --typescript --strict
```

## Project Structure

```
scarf/
├── Cargo.toml          # Project configuration and dependencies
├── src/
│   ├── main.rs         # Main CLI entry point
│   ├── ui.rs           # Terminal UI utilities and styling
│   └── commands/
│       ├── mod.rs      # Command module exports
│       ├── init.rs     # Initialize command implementation
│       └── run.rs      # Run command implementation
├── cargow              # Cargo wrapper (Unix)
├── cargow.bat          # Cargo wrapper (Windows)
└── README.md           # This file
```

## Development

### Running in Development

```bash
./cargow run -- init --source-dir ./example --target-framework react
./cargow run -- run --agent ./agent --from ./src --to vue
```

### Building

```bash
./cargow build
```

### Testing

```bash
./cargow test
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the terms specified in the `LICENSE` file.

## Status

⚠️ **Development Status**: This project is in active development. The `run` command is currently a stub and needs implementation.

---

*Made with ❤️ for the developer community from IBM Research*

