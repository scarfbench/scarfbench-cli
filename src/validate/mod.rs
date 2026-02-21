mod types;

use anyhow::{Context, anyhow};
use clap::Args;
use jwalk::WalkDir;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long, help = "The path to where the agentic conversions are stored")]
    pub conversions_dir: PathBuf,

    #[arg(long, help = "The path where the benchmark directory is stored")]
    pub benchmark_dir: PathBuf,

    #[arg(
        long,
        help = "How much time before we hit evaluation timeout (minutes)."
    )]
    pub make_timeout: Option<u64>,
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
        .min_depth(2)
        .max_depth(2)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            if !entry.file_type().is_dir() {
                return false;
            }
            let Some(name) = entry
                .parent_path
                .parent()
                .and_then(|p| p.file_name())
                .map(|name| name.to_string_lossy())
            else {
                return false;
            };

            let split: Vec<_> = name.split("__").collect();

            split.len() == 4
        })
        .map(|d| d.path())
        .collect::<Vec<_>>();

    // Dispatch parallel calls
    ThreadPoolBuilder::new().build().unwrap().install(|| {
        dirs.par_iter().for_each(|dir| {
            copy_validation_harness_and_run_make_test(
                &args.benchmark_dir,
                dir,
                args.timeout,
            )
            .with_context(|| format!("Failed to run make tests on {:?}", dir))
            .unwrap();
        })
    });
    Ok(0)
}

/// Run `make test` on the deployed directory
fn copy_validation_harness_and_run_make_test(
    benchmark_dir: &PathBuf,
    conversions_dir: &PathBuf,
    timeout_in_minutes: Option<u64>,
) -> anyhow::Result<()> {
    // --- First we will copy over the test harness from the benchmark directory ---
    let (layer, app, framework) = read_metadata_json(conversions_dir)?;
    let src = benchmark_dir.join(layer).join(app).join(framework);
    let dst = conversions_dir.join("output");
    log::debug!("Reading contents from {:?}", src);
    fs::read_dir(&src)?.for_each(|entry| {
        let path: PathBuf =
            entry.map(|f| PathBuf::from(f.file_name())).unwrap();
        if path.is_file()
            && matches!(
                path.to_str(),
                Some("Makefile" | "makefile" | "Dockerfile")
            )
        {
            let _ = fs::copy(&path, dst.join(&path));
        }
    });
    // --- Now we will run make test ---
    let timeout = match timeout_in_minutes {
        Some(minutes) => Duration::from_mins(minutes),
        _ => Duration::from_mins(5),
    };

    // Spawn command
    let mut child = Command::new("make")
        .current_dir(conversions_dir)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .with_context(|| {
            format!("Failed to run make tests on {:?}", conversions_dir)
        })?;

    // Wait for the command to timeout or succeed
    match child
        .wait_timeout(timeout)
        .context("couldn't spawn chile with wait_timeout")?
    {
        Some(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow!(
                    "make tests in {} failed!",
                    conversions_dir.display()
                ))
            }
        },
        None => {
            // Kill the process because we didn't get a status back...
            child.kill()?;
            child.wait()?; // Wait for the process to finish killing
            Err(anyhow::anyhow!(
                "make tests timed out after {:?} in {}",
                timeout,
                conversions_dir.display()
            ))?
        },
    }
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
