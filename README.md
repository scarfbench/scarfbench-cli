![scarfbench-raised](https://github.com/user-attachments/assets/b36885c0-2843-4c23-8550-01115537666d)

[![Crates.io](https://img.shields.io/crates/v/scarfbench-cli?style=for-the-badge)](https://crates.io/crates/scarfbench-cli)
[![License](https://img.shields.io/crates/l/scarfbench-cli?style=for-the-badge)](https://crates.io/crates/scarfbench-cli)
[![npm](https://img.shields.io/npm/v/@scarfbench/scarfbench-cli?color=crimson&logo=npm&style=for-the-badge)](https://www.npmjs.com/package/@scarfbench/scarfbench-cli)

ScarfBench CLI: The command line helper tool for scarf bench

This is a companion CLI tool for the [SCARF Benchmark](https://github.com/scarfbench/benchmark). It provides a commandline interface to list and test benchmarks, run agents, submit solutions, view and explore leaderboard among other useful tasks.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Prerequisites](#prerequisites)
  - [Install with shell script](#install-prebuilt-binaries-via-shell-script)
  - [Install with Homebrew](#install-prebuilt-binaries-via-homebrew)
  - [Install with cargo](#install-prebuilt-binaries-via-cargo)
  - [Install with npm](#install-prebuilt-binaries-via-npm)
  - [Build from Source](#build-from-source)
- [Usage](#usage)

## Features

- List available benchmarks
- Test and validate benchmarks
- Run agents on benchmark problems
- Submit solutions (to be added)
- View and explore leaderboards (to be added)

## Installation

### Prerequisites

Before installing the SCARF CLI, ensure you have the following tools installed:

- **Docker** ([Installation Guide](https://docs.docker.com/get-docker/)) - Runs benchmarks in isolated environments
- **Make** - Builds and runs projects as specified in makefiles
- **Python** - If you want to install `scarf` with pip (optional)

### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/scarfbench/scarf/releases/latest/download/scarfbench-cli-installer.sh | sh
```

### Install prebuilt binaries via Homebrew

```sh
brew tap scarfbench/tap
brew install scarfbench-cli
```

### Install prebuilt binaries via cargo

```sh
cargo install scarfbench-cli
```

### Install prebuilt binaries into your npm project

```sh
npm install @scarfbench/scarfbench-cli
```

### Build from Source

1. **Clone the repository:**
   ```bash
   git clone https://github.com/scarfbench/scarf.git
   cd scarf
   ```

2. **Install the project:**
   ```bash
   ./cargow install --path $PWD --root $HOME/.local/```
   
   The compiled binary will be located in `$HOME/.local/bin`. We'll assume that this path is available in your `$PATH`

3. **Run the CLI:**
   ```
   scarf --help
    ███████╗  ██████╗  █████╗  ██████╗  ███████╗ ██████╗  ███████╗ ███╗   ██╗  ██████╗ ██╗  ██╗
    ██╔════╝ ██╔════╝ ██╔══██╗ ██╔══██╗ ██╔════╝ ██╔══██╗ ██╔════╝ ████╗  ██║ ██╔════╝ ██║  ██║
    ███████╗ ██║      ███████║ ██████╔╝ █████╗   ██████╔╝ █████╗   ██╔██╗ ██║ ██║      ███████║
    ╚════██║ ██║      ██╔══██║ ██╔══██╗ ██╔══╝   ██╔══██╗ ██╔══╝   ██║╚██╗██║ ██║      ██╔══██║
    ███████║ ╚██████╗ ██║  ██║ ██║  ██║ ██║      ██████╔╝ ███████╗ ██║ ╚████║ ╚██████╗ ██║  ██║
    ╚══════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝      ╚═════╝  ╚══════╝ ╚═╝  ╚═══╝  ╚═════╝ ╚═╝  ╚═╝

    ScarfBench CLI: The command line helper tool for scarf bench
    
    Usage: scarf [OPTIONS] <COMMAND>
    
    Commands:
    bench  A series of subcommands to run on the benchmark applications.
    eval   Subcommands to run evaluation over the benchmark
    help   Print this message or the help of the given subcommand(s)
    
    Options:
    -v, --verbose...  Increase verbosity (-v, -vv, -vvv).
    -h, --help        Print help
    -V, --version     Print version
   ```
  
## Usage

After installation, use `scarf --help` to explore the command tree.

### Top-level command

```text
Usage: scarf [OPTIONS] <COMMAND>

Commands:
  bench  A series of subcommands to run on the benchmark applications.
  eval   Subcommands to run evaluation over the benchmark
  help   Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Increase verbosity (-v, -vv, -vvv).
  -h, --help        Print help
  -V, --version     Print version
```

### Quick reference

<table border="2" cellpadding="8" cellspacing="0">
  <thead>
    <tr>
      <th><strong>Group</strong></th>
      <th><strong>Command</strong></th>
      <th><strong>Purpose</strong></th>
      <th><strong>Link</strong></th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td rowspan="4"><strong>Benchmark operations</strong></td>
      <td><code>scarf bench</code></td>
      <td>Operate on benchmark apps</td>
      <td><a href="#group-bench">Go to section</a></td>
    </tr>
    <tr>
      <td><code>scarf bench pull</code></td>
      <td>Pull benchmark versions</td>
      <td><a href="#cmd-bench-pull">Command help</a></td>
    </tr>
    <tr>
      <td><code>scarf bench list</code></td>
      <td>List benchmark applications</td>
      <td><a href="#cmd-bench-list">Command help</a></td>
    </tr>
    <tr>
      <td><code>scarf bench test</code></td>
      <td>Run benchmark regression tests</td>
      <td><a href="#cmd-bench-test">Command help</a></td>
    </tr>
    <tr>
      <td rowspan="2"><strong>Agent evaluation</strong></td>
      <td><code>scarf eval</code></td>
      <td>Evaluate agents on benchmark</td>
      <td><a href="#group-eval">Go to section</a></td>
    </tr>
    <tr>
      <td><code>scarf eval run</code></td>
      <td>Run agent evaluation jobs</td>
      <td><a href="#cmd-eval-run">Command help</a></td>
    </tr>
    <tr>
      <td><strong>Submission validation</strong></td>
      <td><code>scarf validate</code></td>
      <td>Validate converted submissions</td>
      <td><a href="./docs/DEVELOPER.md#cmd-validate">Go to section</a></td>
    </tr>
  </tbody>
</table>

<a id="group-bench"></a>

### 1. `scarf bench` series of commands for operations on the benchmark

THese commands allow you to interact with the benchmark applications, including pulling the latest versions, listing available applications, and running tests. These are mostly for developers and maintainers of the benchmark as well as for users who want to explore the benchmark applications.

```text
Usage: scarf bench [OPTIONS] <COMMAND>

Commands:
  pull  Pull the latest (or user specified) version of the benchmark.
  list  List the application(s) in the benchmark.
  test  Run regression tests (with `make test`) on the benchmark application(s).
  help  Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Increase verbosity (-v, -vv, -vvv).
  -h, --help        Print help
```

<a id="cmd-bench-pull"></a>

#### `scarf bench pull --help`

Pull the latest (or user specified) version of the benchmark.

```text
Usage: scarf bench pull [OPTIONS] --dest <DIR>

Options:
  -d, --dest <DIR>         Path to where the benchmark is to be saved.
  -v, --verbose...         Increase verbosity (-v, -vv, -vvv).
      --version <VERSION>  Version of scarfbench to pull.
  -h, --help               Print help
```

<a id="cmd-bench-list"></a>

#### `scarf bench list --help`

List the application(s) in the benchmark.

```text
Usage: scarf bench list [OPTIONS] --benchmark-dir <BENCHMARK_DIR>

Options:
      --benchmark-dir <BENCHMARK_DIR>  Path to the root of the scarf benchmark.
  -v, --verbose...                     Increase verbosity (-v, -vv, -vvv).
      --layer <LAYER>                  Application layer to list.
  -h, --help                           Print help
```

<a id="cmd-bench-test"></a>

#### `scarf bench test --help`

Run regression tests (with `make test`) on the benchmark application(s).

```text
Usage: scarf bench test [OPTIONS] --benchmark-dir <DIRECTORY>

Options:
      --benchmark-dir <DIRECTORY>  Path to the root of the scarf benchmark.
  -v, --verbose...                 Increase verbosity (-v, -vv, -vvv).
      --layer <LAYER>              Application layer to test.
      --app <APPLICATION>          Application to run the test on.
      --dry-run                    Use dry run instead of full run.
      --logs-dest                  Where to save the logs.
  -h, --help                       Print help
```

<a id="group-eval"></a>

### 2. `scarf eval` series of commands for evaluating agents

These are the key evaluation commands that you will use to run and evaluate agents on the benchmark. 

```text
Usage: scarf eval [OPTIONS] <COMMAND>

Commands:
  run   Evaluate an agent on Scarfbench
  help  Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Increase verbosity (-v, -vv, -vvv).
  -h, --help        Print help
```

<a id="cmd-eval-run"></a>

#### `scarf eval run --help`

Run the evaluation of an agent on the benchmark. This expects the agent to be implemented in the `--agent-dir` directory as per the [agent harness specification](./docs/AGENT_HARNESS.md)
.
```text
Usage: scarf eval run [OPTIONS] --benchmark-dir <DIR> --agent-dir <DIR> --source-framework <FRAMEWORK> --target-framework <FRAMEWORK> --eval-out <EVAL_OUT>

Options:
      --benchmark-dir <DIR>           Path (directory) to the benchmark.
  -v, --verbose...                    Increase verbosity (-v, -vv, -vvv).
      --agent-dir <DIR>               Path (directory) to agent implementation harness.
      --layer <LAYER>                 Application layer to run agent on.
      --app <APP>                     Application to run the agent on. If layer is specified, this app must lie within that layer.
      --source-framework <FRAMEWORK>  The source framework for conversion.
      --target-framework <FRAMEWORK>  The target framework for conversion.
  -p, --pass-at-k <K>                 Value of K to run for generating an Pass@K value. [default: 1]
      --eval-out <EVAL_OUT>           Output directory where the agent runs and evaluation output are stored.
  -j, --jobs <JOBS>                   Number of parallel jobs to run. [default: 1]
      --prepare-only                  Prepare the evaluation harness to run agents. Think of this as a dry run before actually deploying the agents.
  -h, --help                          Print help
```

### Quick examples

```bash
# Pull benchmark into a directory
scarf bench pull --dest /path/to/bench

# List apps in one layer
scarf bench list --benchmark-dir /path/to/bench --layer persistence

# Run benchmark tests for one layer
scarf bench test --benchmark-dir /path/to/bench --layer persistence

# Run evaluation
scarf eval run \
  --benchmark-dir /path/to/bench \
  --agent-dir /path/to/agent-harness \
  --source-framework spring \
  --target-framework quarkus \
  --eval-out /path/to/eval-output

# Validate converted submissions (hidden command)
scarf validate \
  --conversions-dir /path/to/conversions \
  --benchmark-dir /path/to/bench
```