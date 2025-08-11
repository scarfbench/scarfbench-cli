@echo off

:: Define the local installation directory
set PROJECT_DIR=%cd%
set LOCAL_CARGO_DIR=%PROJECT_DIR%\.cargo
set LOCAL_RUSTUP_DIR=%PROJECT_DIR%\.rustup

:: Check if cargo is installed
where cargo > nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Cargo is not installed locally. Installing Rust...

    :: Install Rust locally to the .cargo directory within the project
    set CARGO_HOME=%LOCAL_CARGO_DIR%
    set RUSTUP_HOME=%LOCAL_RUSTUP_DIR%

    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | rustup-init.exe --no-modify-path -y
)

:: Run the cargo command with all passed arguments
cargo %*
