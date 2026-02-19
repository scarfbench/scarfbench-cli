![scarfbench-raised](https://github.com/user-attachments/assets/b36885c0-2843-4c23-8550-01115537666d)

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
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/scarfbench/scarf/releases/download/v0.1.0/scarfbench-cli-installer.sh | sh
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

After installation, you can use the SCARF CLI to interact with the SCARF Benchmark. Here are some common commands:

### 1. List Benchmarks
```
❯ scarf bench list --help

 ███████╗  ██████╗  █████╗  ██████╗  ███████╗ ██████╗  ███████╗ ███╗   ██╗  ██████╗ ██╗  ██╗
 ██╔════╝ ██╔════╝ ██╔══██╗ ██╔══██╗ ██╔════╝ ██╔══██╗ ██╔════╝ ████╗  ██║ ██╔════╝ ██║  ██║
 ███████╗ ██║      ███████║ ██████╔╝ █████╗   ██████╔╝ █████╗   ██╔██╗ ██║ ██║      ███████║
 ╚════██║ ██║      ██╔══██║ ██╔══██╗ ██╔══╝   ██╔══██╗ ██╔══╝   ██║╚██╗██║ ██║      ██╔══██║
 ███████║ ╚██████╗ ██║  ██║ ██║  ██║ ██║      ██████╔╝ ███████╗ ██║ ╚████║ ╚██████╗ ██║  ██║
 ╚══════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝      ╚═════╝  ╚══════╝ ╚═╝  ╚═══╝  ╚═════╝ ╚═╝  ╚═╝

List the application(s) in the benchmark.

Usage: scarf bench list [OPTIONS] --benchmark-dir <ROOT>

Options:
      --benchmark-dir <ROOT>    Path to the root of the scarf benchmark.
  -v, --verbose...     Increase verbosity (-v, -vv, -vvv). If RUST_LOG is set, it takes precedence.
      --layer <LAYER>  Application layer to list.
  -h, --help           Print help
```

This should give you something like below
```bash
❯ scarf bench list --benchmark-dir /home/rkrsn/workspace/scarfbench/benchmark --layer business_domain
┌─────────────────┬──────────────┬───────────┬─────────────────────────────────────────────────────────────────────────────────┐
│ Layer           ┆ Application  ┆ Framework ┆ Path                                                                            │
╞═════════════════╪══════════════╪═══════════╪═════════════════════════════════════════════════════════════════════════════════╡
│ business_domain ┆ cart         ┆ jakarta   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/cart/jakarta         │
│ business_domain ┆ cart         ┆ quarkus   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/cart/quarkus         │
│ business_domain ┆ cart         ┆ spring    ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/cart/spring          │
│ business_domain ┆ converter    ┆ jakarta   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/converter/jakarta    │
│ business_domain ┆ converter    ┆ quarkus   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/converter/quarkus    │
│ business_domain ┆ converter    ┆ spring    ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/converter/spring     │
│ business_domain ┆ counter      ┆ jakarta   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/counter/jakarta      │
│ business_domain ┆ counter      ┆ quarkus   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/counter/quarkus      │
│ business_domain ┆ counter      ┆ spring    ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/counter/spring       │
│ business_domain ┆ helloservice ┆ jakarta   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/helloservice/jakarta │
│ business_domain ┆ helloservice ┆ quarkus   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/helloservice/quarkus │
│ business_domain ┆ helloservice ┆ spring    ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/helloservice/spring  │
│ business_domain ┆ standalone   ┆ jakarta   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/standalone/jakarta   │
│ business_domain ┆ standalone   ┆ quarkus   ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/standalone/quarkus   │
│ business_domain ┆ standalone   ┆ spring    ┆ /home/rkrsn/workspace/scarfbench/benchmark/business_domain/standalone/spring    │
└─────────────────┴──────────────┴───────────┴─────────────────────────────────────────────────────────────────────────────────┘
```

### 2. Test Benchmark Layer(s)

You can use the `scarf bench test` command to test specific benchmark layers or the whole benchmark. Here are some examples:

```
❯ scarf bench test --help

 ███████╗  ██████╗  █████╗  ██████╗  ███████╗ ██████╗  ███████╗ ███╗   ██╗  ██████╗ ██╗  ██╗
 ██╔════╝ ██╔════╝ ██╔══██╗ ██╔══██╗ ██╔════╝ ██╔══██╗ ██╔════╝ ████╗  ██║ ██╔════╝ ██║  ██║
 ███████╗ ██║      ███████║ ██████╔╝ █████╗   ██████╔╝ █████╗   ██╔██╗ ██║ ██║      ███████║
 ╚════██║ ██║      ██╔══██║ ██╔══██╗ ██╔══╝   ██╔══██╗ ██╔══╝   ██║╚██╗██║ ██║      ██╔══██║
 ███████║ ╚██████╗ ██║  ██║ ██║  ██║ ██║      ██████╔╝ ███████╗ ██║ ╚████║ ╚██████╗ ██║  ██║
 ╚══════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝      ╚═════╝  ╚══════╝ ╚═╝  ╚═══╝  ╚═════╝ ╚═╝  ╚═╝

Run regression tests (with `make test`) on the benchmark application(s).

Usage: scarf bench test [OPTIONS] --benchmark-dir <DIRECTORY>

Options:
      --benchmark-dir <DIRECTORY>  Path to the root of the scarf benchmark.
  -v, --verbose...                 Increase verbosity (-v, -vv, -vvv).
      --layer <LAYER>              Application layer to test.
      --app <APPLICATION>          Application to run the test on.
      --dry-run                    Use dry run instead of full run.
  -h, --help                       Print help
```

For example, to test the `persistence` layer:

```bash
❯ scarf bench test --benchmark-dir /home/rkrsn/workspace/scarfbench/benchmark --layer persistence
```

This will run `make tests` in all the apps in `persistence` layer and provide a summary of the results.

```bash
┌─────────────────────────────────────────────────────────────────────────────┬─────────┐
│ Application Path                                                            ┆ Result  │
╞═════════════════════════════════════════════════════════════════════════════╪═════════╡
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/order/jakarta        ┆ Failure │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/roster/spring        ┆ Success │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/order/quarkus        ┆ Failure │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/roster/quarkus       ┆ Success │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/roster/jakarta       ┆ Success │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/address-book/spring  ┆ Success │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/address-book/quarkus ┆ Success │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/address-book/jakarta ┆ Success │
│ /home/rkrsn/workspace/scarfbench/benchmark/persistence/order/spring         ┆ Success │
└─────────────────────────────────────────────────────────────────────────────┴─────────┘
```
