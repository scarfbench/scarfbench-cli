# 🧣 Scarf CLI

A powerful benchmark CLI tool for framework migration with Docker container support.

## Overview

SCARF is a command-line interface tool designed to facilitate agent-driven code transformation and framework migration. It provides utilities to initialize transformation stubs, test Docker container setups, and run agents for refactoring between different frameworks—with full Docker containerization support.

## Features

- **Template Generation**: Create destination stubs with templates for target frameworks
- **Docker Testing**: Test Docker container setups before running agents
- **Agent Execution**: Run transformation agents in Docker containers with custom agent scripts
- **Port Forwarding**: Built-in port mapping for testing applications in containers
- **Flexible Configuration**: Support for custom Docker images, networks, and environment variables

## Installation
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

### Add to Path

```bash
cargo install --path .
```

This will allow you to use `scarf` without needing the binary.

## Usage

SCARF provides three main commands: `init`, `test`, and `run`.

### 1. Initialize Transformation Stub

Create a destination stub with templates for a target framework:

```bash
scarf init \
  --source-dir <SOURCE_DIR> \
  --target-framework <FRAMEWORK> \
  [--output-dir <OUTPUT_DIR>]
```

**Arguments:**
- `--source-dir`: Path to the source directory containing the original code
- `--target-framework`: Target framework to migrate to
- `--output-dir`: Output directory for generated templates (default: "generated")

**Example:**
```bash
scarf init \
  --source-dir ./my-app \
  --target-framework react \
  --output-dir ./output
```

**What it does:**
- Creates output directory structure
- Copies source files for context
- Creates `.tmp` and `.home` directories for agent use (**these folders are where most agents store their chat sessions**)
- Generates framework-specific templates

### 2. Test Docker Container Setup

Test Docker container configuration without running scarf commands:

```bash
scarf test \
  --from <SOURCE_PATH> \
  [--out <OUTPUT_DIR>] \
  [--docker-image <IMAGE>] \
  [--dockerfile <DOCKERFILE_PATH>] \
  [--docker-build-context <BUILD_CONTEXT>] \
  [--docker-name <CONTAINER_NAME>] \
  [--env <KEY=VALUE>...] \
  [--docker-network <NETWORK>] \
  [--port <HOST:CONTAINER>...] \
  [--docker-no-rm] \
  [--command <COMMAND>]
```

**Arguments:**
- `--from`: Source path containing code to test
- `--out`: Output directory (default: "generated")
- `--docker-image`: Use existing Docker image
- `--dockerfile`: Build from Dockerfile
- `--docker-build-context`: Build context directory
- `--docker-name`: Custom container name
- `--env`: Environment variables (KEY=VALUE)
- `--docker-network`: Network mode (none, bridge, host)
- `--port`: Port forwarding (HOST:CONTAINER)
- `--docker-no-rm`: Keep container after test
- `--command`: Command to run (default: tail -f /dev/null)

**Example:**
```bash
scarf test \
  --from "./whole_applications/jakarta/cargotracker" \
  --dockerfile "./containers/gemini/Dockerfile" \
  --docker-build-context "./containers/gemini" \
  --docker-name "test-container" \
  --env "GEMINI_API_KEY=$GEMINI_API_KEY" \
  --port 8080:8080 \
  --port 4848:4848 \
  --docker-no-rm
```

**What it does:**
- Builds Docker image from Dockerfile
- Creates container with source code mounted
- Copies source to output directory for context
- Starts container in detached mode
- Provides instructions for accessing the container

### 3. Run Transformation Agent

Execute an agent to perform code transformation in Docker:

```bash
scarf run \
  --agent <AGENT_PATH> \
  --from <SOURCE_PATH> \
  --to <TARGET_FRAMEWORK> \
  [--out <OUTPUT_DIR>] \
  [--docker-image <IMAGE>] \
  [--dockerfile <DOCKERFILE_PATH>] \
  [--docker-build-context <BUILD_CONTEXT>] \
  [--docker-name <CONTAINER_NAME>] \
  [--env <KEY=VALUE>...] \
  [--docker-network <NETWORK>] \
  [--port <HOST:CONTAINER>...] \
  [--docker-no-rm] \
  [-- <EXTRA_ARGS>...]
```

**Arguments:**
- `--agent`: Path to the transformation agent script
- `--from`: Source path containing code to transform
- `--to`: Target framework name
- `--out`: Output directory (default: "generated")
- `--docker-image`: Use existing Docker image
- `--dockerfile`: Build from Dockerfile
- `--docker-build-context`: Build context directory
- `--docker-name`: Custom container name
- `--env`: Environment variables (KEY=VALUE)
- `--docker-network`: Network mode (none, bridge, host)
- `--port`: Port forwarding (HOST:CONTAINER)
- `--docker-no-rm`: Keep container after run
- `-- <EXTRA_ARGS>`: Additional arguments passed to the agent

**Example:**
```bash
scarf run \
  --agent "./agents/run_gemini.sh" \
  --from "./whole_applications/jakarta/cargotracker" \
  --to "micronaut" \
  --dockerfile "./containers/gemini/Dockerfile" \
  --docker-build-context "./containers/gemini" \
  --env "GEMINI_API_KEY=$GEMINI_API_KEY" \
  --env "GEMINI_PROMPT=$(cat ./prompts/jakarta_to_micronaut_agentic.txt)" \
  --docker-network "host" \
  --docker-no-rm
```

**What it does:**
- Builds Docker image from Dockerfile
- Creates container with agent script and source code mounted
- Copies source to output directory for context
- Runs the agent script inside the container
- Agent transforms code from source framework to target framework

## Project Structure

```
scarf/
├── Cargo.toml              # Project configuration and dependencies
├── src/
│   ├── main.rs             # Main CLI entry point with command definitions
│   ├── ui.rs               # Terminal UI utilities and styling
│   └── commands/
│       ├── mod.rs          # Command module exports
│       ├── init.rs         # Initialize command (local template generation)
│       ├── test.rs         # Test Docker container setup
│       └── run.rs          # Run agent in Docker container
├── Cargo.lock             # Dependency lock file
└── README.md              # This file
```

## Docker Container Features

### **Port Forwarding**
All Docker-enabled commands support port forwarding:
```bash
--port 8080:8080    # Map host port 8080 to container port 8080
--port 4848:4848    # Map host port 4848 to container port 4848
```

### **Network Modes**
```bash
--docker-network none    # No network access (default)
--docker-network bridge  # Docker bridge network
--docker-network host    # Host network (full network access)
```

### **Environment Variables**
```bash
--env "API_KEY=value"           # Single environment variable
--env "API_KEY=value" --env "DEBUG=true"  # Multiple variables
```

### **Container Management**
```bash
--docker-name "my-container"     # Custom container name
--docker-no-rm                   # Keep container after execution
```

## License

This project is licensed under the terms specified in the `LICENSE` file.

## Status

✅ **Production Ready**: All core functionality is implemented and tested.

- ✅ **`scarf init`**: Local template generation
- ✅ **`scarf test`**: Docker container testing with port forwarding
- ✅ **`scarf run`**: Full agent execution in Docker containers
- ✅ **Docker support**: Image building, container management, port forwarding
- ✅ **Agent framework**: Support for custom transformation scripts

---

*Made with ❤️ for the developer community from IBM Research*

