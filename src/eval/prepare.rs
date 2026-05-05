use std::io::Write;
use std::{
    collections::HashMap,
    fs::{self, File, create_dir_all},
    path::{Path, PathBuf},
};

use crate::eval::run::EvalRunArgs;
use crate::eval::types::{
    AgentConfig, EvalGroup, EvalInstance, EvalKey, EvalLayout, RunMetaData,
};
use anyhow::Result;
use walkdir::WalkDir;

/// The public facing prepare harness that sets up the evaluation environment
///
/// Parameters:
/// - args: All the arguments passed by the user
pub fn prepare_harness(args: &EvalRunArgs) -> Result<EvalLayout> {
    let eval_layout = EvalLayout::new(initialize_evals(args)?);
    Ok(eval_layout)
}

/// Populate the evals data structure
fn initialize_evals(args: &EvalRunArgs) -> Result<HashMap<EvalKey, EvalGroup>> {
    let mut evals: HashMap<EvalKey, EvalGroup> = HashMap::new();

    // agent.toml is the single source of truth for the agent's identity, model, and entrypoint.
    let agent_toml_path = args.agent_dir.join("agent.toml");
    let agent_config: AgentConfig = match fs::read_to_string(&agent_toml_path) {
        Ok(s) => toml::from_str(&s).map_err(|e| {
            anyhow::anyhow!("failed to parse {}: {}", agent_toml_path.display(), e)
        })?,
        Err(_) => anyhow::bail!(
            "agent.toml not found at {}. See https://scarfbench.info/quickstart/#agenttoml-file",
            agent_toml_path.display()
        ),
    };
    let agent_name = agent_config.solution_name.clone();
    log::debug!("Using agent name: {}", &agent_name);

    // Iterate over all the selected layers and pick the apps chosen by the user
    // if not all apps will be chosen.
    let apps: Vec<_> = (if !args.layer.is_empty() {
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
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .filter(|e| {
                if args.app.is_empty() {
                    true
                } else {
                    e.file_name()
                        .to_str()
                        .map(|n| args.app.iter().any(|a| a == n))
                        .unwrap_or(false)
                }
            })
            .map(|e| e.path().to_path_buf())
    })
    .collect();

    // If the user provided some --app(s) but they weren't any of the layer(s) the user provided then...
    if apps.is_empty() {
        anyhow::bail!(
            "The app(s) provided with the --app flag were not found for the specified --layer(s)."
        );
    }

    for app_path in apps.iter() {
        log::debug!(
            "Preparing eval for application at path: {}",
            app_path.display()
        );

        // Build an evaluation index key for the each of the evaluation instances
        let eval_instance_key = app_path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|app| {
                app_path
                    .parent() // The parent to get the layer
                    .and_then(|p| p.file_name()) // Get the layer folder's filename
                    .and_then(|layer| layer.to_str()) // Covert folder name to string representation
                    .map(|layer| {
                        // Use the later name to generate an key to index all the evaluation instances
                        EvalKey::new(
                            &agent_name,
                            layer,
                            app,
                            &args.source_framework,
                            &args.target_framework,
                        )
                    })
            })
            .unwrap();

        // A container to gather all the evaluation runs
        let mut runs: Vec<EvalInstance> = Vec::new();

        // Repeat for k (pass @ k) loops 1...k
        for run in 1..=args.pass_at_k {
            // Create a directory in the --eval-out/agent_layer_app_source_framework_dest_framework directory
            let eval_instance_dir = args
                .eval_out
                .join(eval_instance_key.repr())
                .join(format!("run_{}", run));

            // Check if CHANGELOG.md already exists in the output directory
            // If it does, skip this run entirely (conversion already completed)
            let changelog_path = eval_instance_dir.join("output").join("CHANGELOG.md");
            if changelog_path.exists() {
                log::info!(
                    "CHANGELOG.md already exists in {}. Skipping this run (already completed).",
                    eval_instance_dir.join("output").display()
                );
                
                // Still need to add this run to the runs vector for consistency
                // but we won't copy files or create directories
                let eval_input_dir: PathBuf = eval_instance_dir.join("input");
                let eval_output_dir: PathBuf = eval_instance_dir.join("output");
                let eval_validation_dir: PathBuf = eval_instance_dir.join("validation");
                
                runs.push(EvalInstance::new(
                    eval_instance_dir,
                    eval_input_dir,
                    eval_output_dir,
                    eval_validation_dir,
                ));
                continue;
            }

            // If the run directory exists but CHANGELOG.md doesn't, delete it for a clean start
            // Double-check that CHANGELOG.md doesn't exist before deleting
            if eval_instance_dir.exists() && !changelog_path.exists() {
                log::info!(
                    "Run directory exists without CHANGELOG.md at {}. Deleting for clean start.",
                    eval_instance_dir.display()
                );
                match fs::remove_dir_all(&eval_instance_dir) {
                    Ok(_) => {
                        log::debug!("Successfully deleted incomplete run directory");
                    },
                    Err(e) => {
                        anyhow::bail!(
                            "Failed to delete incomplete run directory {}: {}",
                            eval_instance_dir.display(),
                            e
                        );
                    },
                }
            }

            // Create the outer eval directory
            match create_dir_all(&eval_instance_dir) {
                Ok(_) => {
                    log::debug!(
                        "Created eval instance directory: {}",
                        eval_instance_dir.display()
                    );
                },
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create eval instance directory {}: {}",
                        eval_instance_dir.display(),
                        e
                    );
                },
            }
            match create_eval_metadata(
                &eval_instance_dir,
                &eval_instance_key,
                &run,
                &agent_config,
            ) {
                Ok(_) => {
                    log::debug!(
                        "Created eval metadata file in: {}",
                        eval_instance_dir.display()
                    );
                },
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create eval metadata file in {}: {}",
                        eval_instance_dir.display(),
                        e
                    );
                },
            }

            // Create the input, output, and validation directories
            let eval_input_dir: PathBuf = eval_instance_dir.join("input");
            match create_dir_all(&eval_input_dir) {
                Ok(_) => {
                    log::debug!(
                        "Created input directory: {} and seeded it with the source framework",
                        eval_instance_dir.join("input").display()
                    );
                },
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create input directory {}: {}",
                        eval_instance_dir.join("input").display(),
                        e
                    );
                },
            }
            // Copy the app files into the input directory
            copy_app_dir(app_path, &args.source_framework, &eval_input_dir)?;

            let eval_output_dir: PathBuf = eval_instance_dir.join("output");
            match create_dir_all(eval_instance_dir.join("output")) {
                Ok(_) => {
                    log::debug!(
                        "Created output directory: {} and seeded it with the source framework",
                        eval_instance_dir.join("output").display()
                    );
                },
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create output directory {}: {}",
                        eval_instance_dir.join("output").display(),
                        e
                    );
                },
            }
            copy_app_dir(app_path, &args.source_framework, &eval_output_dir)?;

            let eval_validation_dir: PathBuf =
                eval_instance_dir.join("validation");
            match create_dir_all(eval_instance_dir.join("validation")) {
                Ok(_) => {
                    log::debug!(
                        "Created validation directory: {}",
                        eval_validation_dir.display()
                    );
                },
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create validation directory {}: {}",
                        eval_validation_dir.display(),
                        e
                    );
                },
            }

            // Append the current run information to runs
            runs.push(EvalInstance::new(
                eval_instance_dir,
                eval_input_dir,
                eval_output_dir,
                eval_validation_dir,
            ));
        }
        // Update evals directory structure.
        evals.insert(
            eval_instance_key.to_owned(),
            EvalGroup::new(args.eval_out.join(eval_instance_key.repr()), runs),
        );
    }
    Ok(evals)
}

fn create_eval_metadata(
    eval_instance_dir: &Path,
    eval_key: &EvalKey,
    run: &u32,
    agent_config: &AgentConfig,
) -> Result<()> {
    let metadata: RunMetaData = RunMetaData::new(
        eval_key.agent(),
        eval_key.layer(),
        eval_key.app(),
        "PREPARED",
        run.to_owned(),
        eval_key.source_framework(),
        eval_key.target_framework(),
        Some(agent_config.model.clone()),
    );
    // Generate a JSON String (that's prettified)
    let json = serde_json::to_string_pretty(&metadata)?;

    let mut file = File::create(eval_instance_dir.join("metadata.json"))?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn copy_app_dir(
    apps: &Path,
    source_framework: &String,
    dest: &Path,
) -> Result<()> {
    for entry in apps
        .join(source_framework)
        .read_dir()
        .expect("Failed to read {source_framework} directory")
    {
        let entry = entry.expect("Failed to read file in app directory");
        log::trace!("Processing entry: {}", entry.path().display());
        let path = entry.path();
        let file_name =
            path.file_name().expect("Failed to get file name").to_owned();

        if matches!(
            &file_name,
            n if n == std::ffi::OsStr::new("smoke.py")
              || n == std::ffi::OsStr::new("smoke")
              || n == std::ffi::OsStr::new("Makefile")
        ) {
            log::trace!("Skipping file {}", file_name.to_string_lossy());
            continue;
        }

        log::trace!("Copying file {}", file_name.to_string_lossy());
        let dest_path = dest.join(&file_name);
        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            dircopy(&path, &dest_path)?;
        } else if meta.is_file() {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

fn dircopy(from: &Path, to: &Path) -> Result<()> {
    create_dir_all(to)?;
    for entry in from.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let to_path = to.join(entry.file_name());
        let meta = fs::metadata(&path)?;

        if meta.is_dir() {
            dircopy(&path, &to_path)?;
        } else if meta.is_file() {
            fs::copy(&path, &to_path)?;
        }
    }
    Ok(())
}
