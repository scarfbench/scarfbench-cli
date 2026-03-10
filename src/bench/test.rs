use crate::utils::get_or_create_and_get_scarfbench_home_dir;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{ArgAction, Args};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::mpsc};
use wait_timeout::ChildExt;
use walkdir::WalkDir;

#[derive(Args, Debug)]
pub struct BenchTestArgs {
    #[arg(
        long,
        help = "Path to the root of the scarf benchmark.",
        value_name = "DIRECTORY"
    )]
    pub benchmark_dir: PathBuf,

    #[arg(long, help = "Application layer to test.", action=ArgAction::Append, value_name="LAYER")]
    pub layer: Vec<String>,

    #[arg(long, help = "Application to run the test on.", action = ArgAction::Append, value_name="APPLICATION")]
    pub app: Vec<String>,

    #[arg(
        long = "dry-run",
        action = ArgAction::SetTrue,
        help = "Use dry run instead of full run."
    )]
    pub dry_run: bool,

    #[arg(
        long = "logs-dest",
        action = ArgAction::SetTrue,
        help = "Where to save the logs."
    )]
    pub maybe_logs: Option<PathBuf>,
}

/// Create a container to hold command run result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunResult {
    pub timestamp: DateTime<Utc>,
    pub path: PathBuf,
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Run the make -n test command on the makefile in the provided directory
fn run_makefile(path: &PathBuf, dry_run: bool) -> Result<RunResult> {
    // Check to see if there is a makefile in the provided directory.
    if !path.join("Makefile").exists() {
        return Err(anyhow::anyhow!(
            "No Makefile found in the provided directory: {}",
            path.display()
        ));
    }
    let mut cmd = Command::new("make");

    if dry_run {
        cmd.arg("-n");
    }

    let mut child = cmd
        .arg("test")
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // This is the key part - use timeout with 10 minutes (600 seconds)
    let dur = Duration::from_secs(600); // 10 minutes in seconds

    // Wiat for the command to timeout (or succeed)

    match child.wait_timeout(dur)? {
        Some(_) => {
            // Command completed within timeout
            let output = child.wait_with_output()?;
            Ok(RunResult {
                timestamp: Utc::now(),
                path: path.clone(),
                ok: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        },
        None => {
            child.kill()?;
            let output = child.wait_with_output()?;
            Ok(RunResult {
                timestamp: Utc::now(),
                path: path.clone(),
                ok: false,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        },
    }
}

/// The test subcommand that runs make test on all the applications to ensure they work as expected
pub fn run(args: BenchTestArgs) -> Result<i32> {
    log::info!(
        "Running tests to ensure functionality of benchmark applications..."
    );

    if !args.benchmark_dir.exists() {
        anyhow::bail!(
            "The benchmark dir {} doesnt seem to exist.",
            &args.benchmark_dir.display(),
        )
    }

    // Obtain the benchmark root from the repository root
    let bench_root = std::fs::canonicalize(&args.benchmark_dir)
        .context(format!(
            "Failed to canonicalize the benchmark root path: {}",
            args.benchmark_dir.display()
        ))
        .unwrap();

    if bench_root.exists() {
        log::debug!("Benchmark directory: {}", bench_root.display());
    } else {
        anyhow::bail!(
            "The benchmark directory {} does not exist?",
            bench_root.display()
        );
    }

    // Iterate over all the selected layers and pick the apps chosen by the user
    // if not all apps will be chosen.
    let app_dirs: Vec<_> = (if !args.layer.is_empty() {
        args.layer.clone()
    } else {
        WalkDir::new(&args.benchmark_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect()
    })
    .iter()
    .flat_map(|layer| {
        WalkDir::new(args.benchmark_dir.join(layer))
            .min_depth(2)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .filter(|e| {
                if args.app.is_empty() {
                    true
                } else {
                    e.path()
                        .parent()
                        .and_then(|f| f.file_name())
                        .map(|os| os.to_string_lossy().into_owned())
                        .map(|n| args.app.iter().any(|a| a.to_string() == n))
                        .unwrap_or(false)
                }
            })
            .map(|e| e.path().to_path_buf())
    })
    .collect();

    // If the user provided some --app(s) but they weren't any of the layer(s) the user provided then...
    if app_dirs.is_empty() {
        anyhow::bail!(
            "The app(s) provided with the --app flag were not found for the specified --layer(s)."
        );
    }

    // Let's obtain a multi-provider (tx) single channel (rx) to collect results
    let (tx, rx) = mpsc::channel::<(PathBuf, anyhow::Result<RunResult>)>();
    //                              ^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^
    //                                 ▲                  ▲
    //                                 │                  │
    //                                Tx type     Rx Result (Result of makefile run)

    // Each item in the apps_dir will be sent to the following closure such that the closure gets
    // a tx (a transmitter of its own to the common channel) and the reference to the dir to do its work
    app_dirs.par_iter().for_each_with(tx, |tx, dir| {
        // Each worker does its job (i.e., run the makefile and return the result as RunResult)
        let result = run_makefile(dir, args.dry_run);
        log::info!("Completed makefile test in directory: {}.", dir.display(),);
        // Now, clone into an owned directory (using to_path_buf) that each of the worker is
        // using and send that back to the receiver along with the ownership of the result.
        let _ = tx.send((dir.to_path_buf(), result));
    });
    let log_dir = get_or_create_and_get_scarfbench_home_dir()?.join("logs");
    let log_file = log_dir
        .join(format!("benchtest.logs.{}.jsonl", Utc::now().to_rfc3339()));
    // Open a log file to store results

    let mut results: Vec<[String; 2]> = Vec::new();
    //              Only iterate as many times as we have directories
    //                                       │
    //                                       ▼
    //                             |```````````````````|
    // for (dir, res) in rx.iter().take(app_dirs.len()) {

    for (dir, res) in rx.iter() {
        let mut file =
            OpenOptions::new().create(true).append(true).open(&log_file)?;
        match res {
            Ok(res) => {
                let status = if res.ok { "Success" } else { "Failure" };

                results.push([
                    dir.to_string_lossy().into_owned(),
                    status.to_string(),
                ]);

                if res.ok {
                    log::info!(
                        "Makefile test in {} succeeded",
                        res.path.display()
                    );
                } else {
                    log::warn!(
                        "Makefile test in {} failed",
                        res.path.display()
                    );
                }

                let jsonl = serde_json::to_string(&res)?;
                writeln!(file, "{jsonl}")?;
            },
            Err(e) => {
                results.push([
                    dir.to_string_lossy().into_owned(),
                    "Error".to_string(),
                ]);

                log::error!(
                    "Makefile test in {} encountered an error: {}",
                    dir.display(),
                    e
                );
            },
        }
        file.flush()?;
    }

    let header: [String; 2] =
        ["Application Path".to_string(), "Result".to_string()];
    let mut table: comfy_table::Table = comfy_table::Table::new();

    // Tabulate the final results
    table.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED);
    table.set_header(header);
    for rows in results {
        table.add_row(rows.to_vec());
    }
    println!("{}", table);
    Ok(0)
}

// =====[ Unit Tests ]=====
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn _touch_makefile(dir: &PathBuf) -> Result<()> {
        fs::create_dir_all(dir)?;
        let makefile_path = dir.join("Makefile");
        fs::write(makefile_path, "test:\n\techo Hello World\n")?;
        Ok(())
    }

    #[test]
    pub fn test_make_dry_run() {
        let tempfile = tempfile::tempdir().unwrap();
        let app_dir = tempfile.path().join("layer/app/framwork");

        // Create a dummy makefile
        _touch_makefile(&app_dir)
            .expect("Failed to create Makefile in app directory");

        // Run the makefile in dry-run mode
        let result =
            run_makefile(&app_dir, false).expect("Failed to run the makefile");

        // Validate the RunResult captures the makefile directory correctly
        assert_eq!(result.path, app_dir);

        // The output must be okay
        assert!(result.ok);

        // The stdout should contain the echo command
        assert!(result.stdout.contains("echo Hello World"));
    }
}
