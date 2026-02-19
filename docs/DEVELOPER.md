# Development

### Development Dependencies

Use `make setup` to install and verify tooling (`rustup`, `rustfmt`, `clippy`, `cargo-nextest`, `cargo-llvm-cov`). You can also install them manually:

1. **Clippy** - Linting and code quality checks
   ```bash
   ./rustupw component add clippy
   ```

2. **Rustfmt** - Code formatting
   ```bash
   ./rustupw component add rustfmt
   ```

3. **LLVM Coverage Tools** - Coverage analysis
   ```bash
   ./rustupw component add llvm-tools-preview
   ./cargow install cargo-llvm-cov
   ```

4. **Nextest** - Advanced test runner
   ```bash
   ./cargow install cargo-nextest --locked
   ```

5. **dist** - To bundle and ship releases
  ```bash
  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/download/v0.30.4/cargo-dist-installer.sh | sh
  ```

### Testing

The project follows idiomatic Rust testing practices:

- **Unit tests**: Located within each module under the `#[cfg(test)]` attribute
- **Integration tests**: Place in `tests/` (not currently present) for CLI-level coverage. 
  - For intergation tests, use descriptive names for test files, e.g., `cli_tests.rs`.

### Building and Testing

A [Makefile](Makefile) is provided to streamline development tasks. Run `make help` to see available commands. You can run `make help` to see all available targets:

| Target     | Description                                                      |
|------------|------------------------------------------------------------------|
| `all`      | Run full pipeline (setup → fmt → clippy → build → test → coverage) |
| `setup`    | Check/install rustup, cargo, components, nextest, llvm-cov       |
| `fmt`      | Run `cargo fmt --all`                                            |
| `clippy`   | Run `cargo clippy` with warnings denied                          |
| `build`    | Run `cargo build`                                                |
| `test`     | Run tests using `cargo nextest`                                  |
| `coverage` | Run coverage using `cargo llvm-cov` + nextest                    |
| `clean`    | Run `cargo clean`                                                |
| `help`     | Show help message                                                |

Run the full pipeline with:
```bash
make
```

To build a release binary:
```bash
./cargow build --release
./target/release/scarf --help
```
