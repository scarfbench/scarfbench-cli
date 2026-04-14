pub mod types;

use anyhow::{anyhow, Context};
use clap::Args;
use kdam::term;
use kdam::BarExt;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::IsTerminal;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use wait_timeout::ChildExt;
use walkdir::WalkDir;

use crate::utils::progress_bar::ProgressBar;

use regex::Regex;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long, help = "The path to where the agentic conversions are stored")]
    pub conversions_dir: PathBuf,

    #[arg(long, help = "The path where the benchmark directory is stored")]
    pub validations_dir: PathBuf,

    #[arg(
        long,
        default_value_t = 10,
        help = "How much time before we hit evaluation timeout (minutes)."
    )]
    pub timeout: u64,

    #[arg(
        long,
        default_value_t = false,
        help = "Only reanalyze existing run.log files without running make test again"
    )]
    pub reanalyze: bool,

    #[arg(
        long,
        help = "Path to model costs CSV file for cost calculation"
    )]
    pub costs_csv: Option<PathBuf>,

    #[arg(
        long,
        help = "Output cost summary as JSON to the specified file"
    )]
    pub cost_json_output: Option<PathBuf>,
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
///     source and target framework.
/// c. We use this to look through the benchmark directory to copy the makefile, the dockerfile
///     and the smoke.py (or smoke/) to the output directory of the same folder that has the metadata.json
/// d. Then, while inside the output directory, we call make test (with a 300 second or user specified) timeout.
/// e. Pipe the contents of the `make logs` command to validation/run.log file and terminate.
/// g. Parallelize the whole pipeline with rayon.
pub fn run(args: ValidateArgs) -> anyhow::Result<i32> {
    let conversions_dir = args.conversions_dir.clone();
    let reanalyze = args.reanalyze;

    let dirs: Vec<_> = WalkDir::new(&conversions_dir)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            // Find the location of the metadata file. This will be our
            // anchor
            entry.file_type().is_dir()
                && entry.file_name().to_string_lossy().starts_with("run_")
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

    // Create cost calculator to track conversion statistics
    let cost_calculator = if let Some(ref costs_path) = args.costs_csv {
        match types::ModelCosts::load_from_csv(costs_path) {
            Ok(costs) => {
                log::info!("Loaded model costs from {}", costs_path.display());
                Arc::new(Mutex::new(types::ConversionCostCalculator::with_costs(costs)))
            },
            Err(e) => {
                log::warn!("Failed to load model costs: {}. Continuing without cost calculation.", e);
                Arc::new(Mutex::new(types::ConversionCostCalculator::new()))
            }
        }
    } else {
        Arc::new(Mutex::new(types::ConversionCostCalculator::new()))
    };

    let (tx, rx) = mpsc::channel::<UiMessage>();

    // Set up the terminal printer aggregrator that will aggregate the responses
    // from each of my worker and create/update a progress bar
    let action =
        if reanalyze { "Reanalyzing Logs" } else { "Evaluating Conversions" };
    let tui = thread::spawn(move || -> anyhow::Result<()> {
        // Set up my terminal
        term::init(std::io::stderr().is_terminal());

        // initialize our progress bar
        let mut pb = total.progress(action, " Eval");

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

        Ok(())
    });

    // Dispatch parallel calls (one for each directory I found above)
    dirs.par_iter().for_each_with((tx.clone(), cost_calculator.clone()), |(tx, calc), dir| {
        let res: anyhow::Result<()> =  fs::read_dir(dir)
            .with_context(|| format!("Failed to read sub directories of {}", dir.display()))
            .and_then(|entries| {
                entries
                    .filter_map(Result::ok) // ignore any subdirs that are not readable. This shouldn't happen but still...
                    .map(|e| e.path())
                    .filter(|p| p.is_dir() && p.file_name().and_then(|n| n.to_str()).map_or(false, |s| s.starts_with("run_")))
                    .try_for_each(|subdir| {
                        let result = if reanalyze {
                            reanalyze_existing_logs(&subdir)
                        } else {
                            copy_validation_harness_and_run_make_test(
                                &args.validations_dir,
                                &subdir,
                                args.timeout,
                            )
                        };

                        // After processing, collect statistics from metadata
                        if result.is_ok() {
                            if let Ok(metadata_content) = fs::read_to_string(subdir.join("metadata.json")) {
                                if let Ok(metadata) = serde_json::from_str::<types::Metadata>(&metadata_content) {
                                    // Look for agent.out file in validation directory
                                    let agent_out_path = subdir.join("validation").join("agent.out");
                                    if let Ok(mut calc_lock) = calc.lock() {
                                        calc_lock.add_conversion(&metadata, &agent_out_path);
                                    }
                                }
                            }
                        }

                        result
                    })
            });

        match res {
            Ok(_) => {
                let action = if reanalyze { "reanalyzed" } else { "validated" };
                let _ = tx.send(UiMessage::Log(format!(
                    "{}\t{}",
                    format!("{}", "[INFO]".to_string()).bold().bright_cyan(),
                    format!(
                        "Successfully {} {}",
                        action,
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

    // Finalize costs and print summary after all validations are complete
    if let Ok(mut calc) = cost_calculator.lock() {
        calc.finalize_costs();
        calc.print_summary();
        
        // Output JSON if requested
        if let Some(ref json_path) = args.cost_json_output {
            let json_output = calc.to_json();
            fs::write(json_path, json_output)
                .with_context(|| format!("Failed to write cost JSON to {}", json_path.display()))?;
            log::info!("Cost summary written to {}", json_path.display());
        }
    }

    Ok(0)
}

/// Reanalyze existing run.log files without running make test
fn reanalyze_existing_logs(conversions_dir: &PathBuf) -> anyhow::Result<()> {
    let log_path = conversions_dir.join("validation").join("run.log");

    if !log_path.exists() {
        return Err(anyhow::anyhow!(
            "No run.log found at {}. Run without --reanalyze first.",
            log_path.display()
        ));
    }

    parse_run_log_and_update_metadata(&log_path)?;
    Ok(())
}

/// Run `make test` on the deployed directory
fn copy_validation_harness_and_run_make_test(
    validations_dir: &PathBuf,
    conversions_dir: &PathBuf,
    timeout_in_minutes: u64,
) -> anyhow::Result<()> {
    // --- First we will copy over the test harness from the benchmark directory ---
    let (layer, app, framework) = read_metadata_json(conversions_dir)?;
    let src = validations_dir.join(layer).join(app).join(framework);
    let dst = conversions_dir.join("output");
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

    // Analyze compile, deploy, and test outcomes
    let (compile_outcome, compile_reason, compile_category) =
        analyze_compile(&log);
    let (deploy_outcome, deploy_reason, deploy_category) =
        analyze_deploy(&log, &compile_outcome);
    let (test_outcome, test_reason, test_category, inconclusive) =
        analyze_tests(&log, &deploy_outcome);

    // Update metadata
    metadata.compile_ok = compile_outcome;
    metadata.deploy_ok = deploy_outcome;
    metadata.test_pass_percent = test_outcome;
    metadata.inconclusive = inconclusive;

    // Set failure reason and category
    let mut reasons = Vec::new();
    if let Some(r) = compile_reason {
        reasons.push(r);
    }
    if let Some(r) = deploy_reason {
        reasons.push(r);
    }
    if let Some(r) = test_reason {
        reasons.push(r);
    }

    if !reasons.is_empty() {
        metadata.failure_reason = Some(reasons.join("; "));
    }

    // Priority: test > deploy > compile category
    metadata.failure_category =
        test_category.or(deploy_category).or(compile_category);

    let failed_pattern = Regex::new(r"(\d+)\s+failed")?;
    let passed_pattern = Regex::new(r"(\d+)\s+passed")?;

    let failed: f64 = failed_pattern
        .captures(&log)
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(0.0);

    let passed: f64 = passed_pattern
        .captures(&log)
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(0.0);

    let total = passed + failed;
    metadata.test_pass_percent = if total == 0.0 {
        String::from("UNK")
    } else {
        let frac = (passed / total * 100.0 * 100.0).round() / 100.0;
        format!("{} out of {} tests passed ({}%)", passed, total, frac)
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

/// Analyze compile status from log
fn analyze_compile(
    log: &str,
) -> (types::ValidationOutcome, Option<String>, Option<types::FailureCategory>)
{
    // BUILD FAILURE in maven - check this FIRST before BUILD SUCCESS
    if log.contains("BUILD FAILURE") {
        // Check if it's a compile error vs other maven error
        if log.contains("COMPILATION ERROR")
            || log.contains("cannot find symbol")
            || log.contains("package") && log.contains("does not exist")
        {
            let reason =
                "Compilation errors detected in Maven build".to_string();
            return (
                types::ValidationOutcome::False,
                Some(reason),
                Some(types::FailureCategory::CompileError),
            );
        }

        if log.contains("NoPluginFoundForPrefixException")
            || log.contains("No plugin found for prefix")
        {
            let reason =
                "Maven plugin not found - wrong framework build tool used"
                    .to_string();
            return (
                types::ValidationOutcome::False,
                Some(reason),
                Some(types::FailureCategory::BuildConfigError),
            );
        }

        // Check for dependency issues
        if log.contains("Could not resolve dependencies")
            || log.contains("Failed to collect dependencies")
        {
            let reason = "Maven dependency resolution failed".to_string();
            return (
                types::ValidationOutcome::False,
                Some(reason),
                Some(types::FailureCategory::CompileDependency),
            );
        }

        // Generic build failure
        let reason = "Maven build failed".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::BuildFailure),
        );
    }

    // Docker image built successfully = compile succeeded
    if log.contains("naming to docker.io") {
        return (types::ValidationOutcome::True, None, None);
    }

    // Explicit BUILD SUCCESS in maven
    if log.contains("BUILD SUCCESS") {
        return (types::ValidationOutcome::True, None, None);
    }

    // Docker build failed
    if log.contains("docker build")
        && (log.contains("ERROR") || log.contains("failed to"))
        && !log.contains("naming to docker.io")
    {
        let reason = "Docker build failed".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::DockerBuildError),
        );
    }

    // pull access denied means the image from a previous build wasn't available
    if log.contains("pull access denied")
        && !log.contains("naming to docker.io")
    {
        let reason = "Docker image not found (build likely failed in a shared build step)".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::DockerImageMissing),
        );
    }

    // If there's make: *** [build] Error
    if log.contains("make: *** [build]") {
        let reason = "Build step failed".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::BuildFailure),
        );
    }

    (
        types::ValidationOutcome::Unk,
        Some("No clear compile outcome found in logs".to_string()),
        Some(types::FailureCategory::Unknown),
    )
}

/// Analyze deploy status from log
fn analyze_deploy(
    log: &str,
    compile_ok: &types::ValidationOutcome,
) -> (types::ValidationOutcome, Option<String>, Option<types::FailureCategory>)
{
    if matches!(compile_ok, types::ValidationOutcome::False) {
        return (
            types::ValidationOutcome::False,
            Some("Cannot deploy - compilation failed".to_string()),
            Some(types::FailureCategory::CompileDependency),
        );
    }

    // App started and ready
    if log.contains("pplication started and ready.") {
        return (types::ValidationOutcome::True, None, None);
    }

    // If tests ran, deployment must have succeeded
    let test_summary_pattern =
        Regex::new(r"=+ .*(?:passed|failed|error).*=+").unwrap();
    if test_summary_pattern.is_match(log) {
        return (types::ValidationOutcome::True, None, None);
    }

    // If there was pytest output at all
    if log.contains("short test summary info")
        || log.contains("PASSED")
        || log.contains("FAILED smoke.py")
    {
        return (types::ValidationOutcome::True, None, None);
    }

    // Container started but app didn't come up
    if log.contains("docker run -d")
        && (log.contains("Connection refused")
            || log.contains("container exited"))
    {
        let reason =
            "Container started but application failed to start".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::AppStartupFailure),
        );
    }

    // Docker run errors
    if log.contains("pull access denied") {
        let reason = "Docker image not found".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::DockerImageMissing),
        );
    }

    if log.contains("container name")
        && log.contains("already in use")
        && !log.contains("naming to docker.io")
    {
        let reason = "Container name conflict from previous run".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::ContainerConflict),
        );
    }

    // If make up failed
    if log.contains("make: *** [up]") {
        if matches!(compile_ok, types::ValidationOutcome::True) {
            let reason = "Deployment failed after successful build".to_string();
            return (
                types::ValidationOutcome::False,
                Some(reason),
                Some(types::FailureCategory::DeployFailure),
            );
        }
        let reason = "make up failed".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::DeployFailure),
        );
    }

    // Terminated
    if log.contains("Terminated: 15") {
        let reason = "Process was terminated (SIGTERM)".to_string();
        return (
            types::ValidationOutcome::False,
            Some(reason),
            Some(types::FailureCategory::ProcessTerminated),
        );
    }

    // Log ends with "waiting for app to start..." - process was cut short
    // Check if the log contains this message and doesn't have deployment success or test results after it
    if log.contains("waiting for app to start...") {
        let after_waiting =
            log.split("waiting for app to start...").last().unwrap_or("");
        // If there's no meaningful content after "waiting for app to start..." (just whitespace or check comments)
        if !after_waiting.contains("pplication started and ready")
            && !after_waiting.contains("PASSED")
            && !after_waiting.contains("FAILED")
            && !after_waiting.contains("===")
        {
            let reason = "Validation process cut short - log ends at health check wait (rerun needed)".to_string();
            return (
                types::ValidationOutcome::Unk,
                Some(reason),
                Some(types::FailureCategory::ValidationTruncated),
            );
        }
    }

    if matches!(compile_ok, types::ValidationOutcome::True) {
        return (
            types::ValidationOutcome::Unk,
            Some(
                "Compiled successfully but no deploy outcome found".to_string(),
            ),
            Some(types::FailureCategory::Unknown),
        );
    }

    (
        types::ValidationOutcome::Unk,
        Some("No clear deploy outcome found".to_string()),
        Some(types::FailureCategory::Unknown),
    )
}

/// Analyze test results from log
fn analyze_tests(
    log: &str,
    deploy_ok: &types::ValidationOutcome,
) -> (String, Option<String>, Option<types::FailureCategory>, bool) {
    if matches!(deploy_ok, types::ValidationOutcome::False) {
        return (
            "0".to_string(),
            Some("Cannot test - deployment failed".to_string()),
            Some(types::FailureCategory::DeployDependency),
            false,
        );
    }

    // Look for pytest summary lines
    let summary_pattern =
        Regex::new(r"=+ (.*?(?:passed|failed|error).*?) =+").unwrap();
    let summaries: Vec<_> = summary_pattern.captures_iter(log).collect();

    if let Some(last_summary) = summaries.last() {
        let summary = &last_summary[1];

        let passed_re = Regex::new(r"(\d+) passed").unwrap();
        let failed_re = Regex::new(r"(\d+) failed").unwrap();
        let error_re = Regex::new(r"(\d+) error").unwrap();

        let passed: i32 = passed_re
            .captures(summary)
            .and_then(|c| c[1].parse().ok())
            .unwrap_or(0);
        let failed: i32 = failed_re
            .captures(summary)
            .and_then(|c| c[1].parse().ok())
            .unwrap_or(0);
        let errors: i32 = error_re
            .captures(summary)
            .and_then(|c| c[1].parse().ok())
            .unwrap_or(0);

        let total = passed + failed + errors;
        if total > 0 {
            let pct = (passed as f64 / total as f64 * 100.0).round();
            let pct_str = format!("{}", pct);

            if failed > 0 || errors > 0 {
                let reason = format!(
                    "{} failed, {} errors, {} passed out of {} tests",
                    failed, errors, passed, total
                );
                return (
                    pct_str,
                    Some(reason),
                    Some(types::FailureCategory::TestFailures),
                    false,
                );
            } else {
                return (pct_str, None, None, false);
            }
        }
    }

    // Check for Error 137 during test (OOM/timeout kill)
    if log.contains("make: *** [test] Error 137") {
        let reason =
            "Test process killed (Error 137 - likely OOM/timeout)".to_string();
        return (
            "UNK".to_string(),
            Some(reason),
            Some(types::FailureCategory::TestTimeoutOom),
            true,
        );
    }

    // make test Error 1
    if log.contains("make: *** [test] Error 1") {
        let reason = "Test step failed with Error 1".to_string();
        return (
            "0".to_string(),
            Some(reason),
            Some(types::FailureCategory::TestFailure),
            true,
        );
    }

    if matches!(deploy_ok, types::ValidationOutcome::True) {
        let reason =
            "App deployed but no test results found in log".to_string();
        return (
            "UNK".to_string(),
            Some(reason),
            Some(types::FailureCategory::NoTestOutput),
            true,
        );
    }

    ("UNK".to_string(), None, None, false)
}

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
