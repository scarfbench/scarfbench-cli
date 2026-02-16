@echo off
setlocal

REM Resolve repo root from this script location (independent of current directory)
set "ROOT=%~dp0"
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"

set "CARGO_HOME=%ROOT%\.cargo"
set "RUSTUP_HOME=%ROOT%\.rustup"
set "CARGO_TARGET_DIR=%ROOT%\target"
set "CARGO_INSTALL_ROOT=%CARGO_HOME%"
set "PATH=%CARGO_HOME%\bin;%PATH%"

if not exist "%CARGO_HOME%\bin\rustup.exe" (
    echo [rustupw] missing local rustup at %CARGO_HOME%\bin\rustup.exe
    echo [rustupw] run .\cargow --version to install local rustup/cargo first
    exit /b 1
)

"%CARGO_HOME%\bin\rustup.exe" %*
set "RC=%ERRORLEVEL%"
endlocal & exit /b %RC%
