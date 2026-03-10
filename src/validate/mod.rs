mod types;

use anyhow::{Context, anyhow};
use clap::Args;
use kdam::BarExt;
use kdam::term;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::IsTerminal;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use wait_timeout::ChildExt;

use crate::utils::progress_bar::ProgressBar;

use aho_corasick::AhoCorasick;
use regex::Regex;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long, help = "The path to where the agentic conversions are stored")]
    pub conversions_dir: PathBuf,

    #[arg(long, help = "The path where the benchmark directory is stored")]
    pub benchmark_dir: PathBuf,

    #[arg(
        long,
        default_value_t = 5,
        help = "How much time before we hit evaluation timeout (minutes)."
    )]
    pub timeout: u64,
}

// Each worker will send back to progress bar thread one of these
enum UiMessage {
    Tick(usize),
    Log(String),
}
/// Runs the validation pipeline on all the conversions
///
/// Notes
/// -----
/// a. Walk over all the directories in the conversions directory
/// b. Read the innermost metadata.json file. This will give us the layer name, the app name,
/// source and target framework.
/// c. We use this to look through the benchmark directory to copy the makefile, the dockerfile
/// and the smoke.py (or smoke/) to the output directory of the same folder that has the metadata.json
/// d. Then, while inside the output directory, we call make test (with a 300 second) timeout.
/// e. Pipe the contents of the `make logs` command to validation/run.log file and terminate.
/// g. Parallelize the whole pipeline with rayon.
pub fn run(args: ValidateArgs) -> anyhow::Result<i32> {
    let conversions_dir = args.conversions_dir;
    let dirs: Vec<_> = WalkDir::new(conversions_dir)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            // Find the location of the metadata file. This will be our
            // anchor
            entry.file_type().is_file()
                && entry.file_name().to_string_lossy() == "metadata.json"
        })
        .filter_map(|entry| {
            Some(entry.path().parent()?.to_path_buf())
        })
        // De dup
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    log::debug!("Found {} conversions", dirs.len());

    let total = dirs.len();

    let (tx, rx) = mpsc::channel::<UiMessage>();

    // Set up the terminal printer aggregrator that will aggregate the responses
    // from each of my worker and create/update a progress bar
    let tui = thread::spawn(move || -> anyhow::Result<()> {
        // Set up my terminal
        term::init(std::io::stderr().is_terminal());
        term::hide_cursor()?;

        // initialize our progress bar
        let mut pb = total.progress("Evaluating Conversions", "Solutions");

        while let Ok(msg) = rx.recv() {
            match msg {
                UiMessage::Tick(n) => {
                    pb.update(n)?;
                },
                UiMessage::Log(l) => {
                    pb.write(l)?;
                },
            }
        }
        eprint!(""); // GIve ourselves one line after the progress bar in case
        term::show_cursor()?;
        Ok(())
    });

    // Dispatch parallel calls (one for each directory I found above)
    dirs.par_iter().for_each_with(tx.clone(), |tx, dir| {
        let res = copy_validation_harness_and_run_make_test(
            &args.benchmark_dir,
            &dir,
            args.timeout,
        );
        match res {
            Ok(_) => {
                let _ = tx.send(UiMessage::Log(format!(
                    "{}\t{}",
                    format!("{}", "[INFO]".to_string()).bold().bright_cyan(),
                    format!(
                        "Successfully completed validations on {}",
                        dir.to_string_lossy()
                    )
                    .bold()
                    .bright_white()
                )));
            },
            Err(e) => {
                let _ = tx.send(UiMessage::Log(format!(
                    "{}\t{}",
                    format!("{}", "[ERROR]".to_string())
                        .bold()
                        .bright_magenta(),
                    format!("{}", e).bold().bright_magenta()
                )));
            },
        }

        let _ = tx.send(UiMessage::Tick(1));
    });
    // Thats it---discard any stray transmitters
    drop(tx);
    tui.join().map_err(|e| anyhow!("TUI panicked: {:?}", e))??;
    Ok(0)
}

/// Run `make test` on the deployed directory
fn copy_validation_harness_and_run_make_test(
    benchmark_dir: &PathBuf,
    conversions_dir: &PathBuf,
    timeout_in_minutes: u64,
) -> anyhow::Result<()> {
    // --- First we will copy over the test harness from the benchmark directory ---
    let (layer, app, framework) = read_metadata_json(conversions_dir)?;
    let src = benchmark_dir.join(layer).join(app).join(framework);
    let dst = conversions_dir.join("output");
    println!(
        "Copying validation harness from {} to {}",
        src.display(),
        dst.display()
    );
    fs::read_dir(&src)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                matches!(n, "Makefile" | "makefile" | "Dockerfile" | "smoke.py")
                    || (n == "smoke" && p.is_dir())
            })
        })
        .for_each(|src_path| {
            let dst_path = dst.join(src_path.file_name().unwrap()); // safe because file_name exists
            log::info!(
                "Copying {} -> {}",
                src_path.display(),
                dst_path.display()
            );

            let copy_result = if src_path.is_dir() {
                copy_dir_recursive(&src_path, &dst_path)
            } else {
                fs::copy(&src_path, &dst_path)
                    .map(|_| ())
                    .map_err(anyhow::Error::from)
            };

            if let Err(e) = copy_result {
                log::warn!(
                    "Failed to copy {} -> {}: {e}",
                    src_path.display(),
                    dst_path.display()
                );
            }
        });
    // --- Now we will run make test ---
    let timeout = Duration::from_secs(timeout_in_minutes * 60);
    let log_dir = conversions_dir.join("validation");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log dir {:?}", log_dir))?;

    let log_path = log_dir.join("run.log");
    let log_file = File::create(&log_path)
        .with_context(|| format!("Failed to create log file {:?}", log_path))?;

    // Clone file handle so both stdout and stderr can write to same file
    let log_file_err = log_file
        .try_clone()
        .with_context(|| "Failed to clone log file handle")?;

    // Spawn command
    let mut child = Command::new("make")
        .current_dir(&dst)
        .args(["test"])
        .stdin(Stdio::null())
        .stderr(Stdio::from(log_file_err))
        .stdout(Stdio::from(log_file))
        .spawn()
        .with_context(|| {
            format!("Failed to run make tests on {:?}", conversions_dir)
        })?;

    // Wait for the command to timeout or succeed
    match child
        .wait_timeout(timeout)
        .context("couldn't spawn child with wait_timeout")?
    {
        Some(_) => {
            parse_run_log_and_update_metadata(&log_path)?;
        },
        None => {
            // Kill the process because we didn't get a status back...
            child.kill()?;
            child.wait()?; // Wait for the process to finish killing
            parse_run_log_and_update_metadata(&log_path)?;
        },
    };

    Ok(())
}

/// Look at the log file and determine whether the run passed, or at what stage it failed
fn parse_run_log_and_update_metadata(log_path: &Path) -> anyhow::Result<()> {
    let log = fs::read_to_string(log_path).with_context(|| {
        format!("failed to read run log at {}", log_path.display())
    })?;
    let metadata_path = log_path
        .parent()
        .unwrap()
        .parent()
        .context("log path has no parent directory")?
        .join("metadata.json");

    let mut metadata: types::Metadata = serde_json::from_reader(
        File::open(&metadata_path).with_context(|| {
            format!(
                "failed to open metadata JSON at {}",
                metadata_path.display()
            )
        })?,
    )
    .with_context(|| {
        format!("failed to parse metadata JSON at {}", metadata_path.display())
    })?;

    let compile_fail = AhoCorasick::new([
        "BUILD FAILURE",
        "COMPILATION FAILURE",
        "Compilation failure",
        "No plugin found for prefix",
    ])?;
    let compile_success =
        AhoCorasick::new(["BUILD SUCCESS", "BUILD SUCCESSFUL"])?;

    let deploy_fail = AhoCorasick::new([
        "Container exited before",
        "[ERROR] Container exited before startup success:",
        "Failed to connect to localhost",
        "container is not running",
        "make: *** [Makefile:",
    ])?;
    let deploy_success = AhoCorasick::new(["Application started and ready"])?;

    let test_pass_fail_pattern =
        Regex::new(r"(\d+)\s+failed,\s+(\d+)\s+passed")?;

    metadata.compile_ok =
        match (compile_success.is_match(&log), compile_fail.is_match(&log)) {
            (_, true) => types::ValidationOutcome::False,
            (true, false) => types::ValidationOutcome::True,
            _ => types::ValidationOutcome::Unk,
        };

    metadata.deploy_ok =
        match (deploy_success.is_match(&log), deploy_fail.is_match(&log)) {
            (true, false) => types::ValidationOutcome::True,
            (_, true) => types::ValidationOutcome::False,
            _ => types::ValidationOutcome::Unk,
        };

    metadata.test_pass_percent =
        match test_pass_fail_pattern.captures_iter(&log).last() {
            Some(c) => {
                let failed: f64 = c[1].parse().unwrap_or(0.0);
                let passed: f64 = c[2].parse().unwrap_or(0.0);
                let frac: f64 = passed / (passed + failed) * 100.00;
                format!(
                    "{} out of {} tests passed ({}%)",
                    failed,
                    passed,
                    frac.round()
                )
            },
            None => String::from("UNK"),
        };

    let mut metadata_file =
        File::create(&metadata_path).with_context(|| {
            format!(
                "failed to open metadata JSON for writing at {}",
                metadata_path.display()
            )
        })?;
    metadata_file
        .write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;

    Ok(())
}

/// Parse the run.log and categorize the errors or pass rates

/// Read the metadata.json file. This will give us the layer name, the app name, and the target framework
///
/// Parameters:
/// path: The path to the metadata.json file.
///
/// Returns:
/// (layer name, app name, and the target framework) from the
fn read_metadata_json(path: &Path) -> anyhow::Result<(String, String, String)> {
    let file = File::open(path.join("metadata.json")).with_context(|| {
        format!("failed to open metadata.json in {}", path.display())
    })?;
    let metadata: types::Metadata = serde_json::from_reader(file)
        .with_context(|| {
            format!("failed to parse metadata JSON from {}", path.display())
        })?;
    // Return the 3-tuple (triple?) with the useful information from the metadata.json
    Ok((metadata.layer, metadata.app, metadata.target_framework.to_string()))
}

fn copy_dir_recursive(from: &Path, to: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
