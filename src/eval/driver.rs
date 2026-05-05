use crate::eval::types::{AgentConfig, EvalInstance, EvalLayout, RunMetaData};
use crate::validate::types::{ConversionCostCalculator, Metadata};
use std::io::ErrorKind;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

/// The main helper to dispatch calls to the user defined agent
pub fn dispatch_agent(
    agent_dir: &Path,
    eval_layout: &EvalLayout,
    cost_calculator: Option<Arc<Mutex<ConversionCostCalculator>>>,
) -> anyhow::Result<()> {
    // Load agent.toml from the agent_dir. This gives us the proper agent name (matched
    // against metadata.json's solution_name) and the entrypoint script to invoke.
    let agent_toml_path = agent_dir.join("agent.toml");
    let agent_config: AgentConfig = match fs::read_to_string(&agent_toml_path) {
        Ok(s) => toml::from_str(&s).map_err(|e| {
            anyhow::anyhow!("failed to parse {}: {}", agent_toml_path.display(), e)
        })?,
        Err(e) if e.kind() == ErrorKind::NotFound => anyhow::bail!(
            "agent.toml not found at {}. See https://scarfbench.info/quickstart/#agenttoml-file",
            agent_toml_path.display()
        ),
        Err(e) => anyhow::bail!(
            "failed to read {}: {}",
            agent_toml_path.display(),
            e
        ),
    };

    for (eval_key, eval_group) in eval_layout {
        // Check how many runs are already complete before starting
        let total_runs = eval_group.runs().len();
        let completed_runs: Vec<&EvalInstance> = eval_group
            .into_iter()
            .filter(|eval_instance| {
                eval_instance.output().join("CHANGELOG.md").exists()
            })
            .collect();
        
        if !completed_runs.is_empty() {
            println!("✓ Found {} of {} runs already completed (CHANGELOG.md exists)",
                     completed_runs.len(), total_runs);
            for eval_instance in &completed_runs {
                println!("  ✓ Skipping: {}", eval_instance.output().display());
            }
        }

        let pool = ThreadPoolBuilder::new()
            .num_threads(std::cmp::min(
                eval_group.runs().len(),
                std::thread::available_parallelism()
                    .map(|t| t.get())
                    .unwrap_or(1),
            ))
            .build()?;

        let _: anyhow::Result<Vec<()>> = pool.install(|| {
            eval_group
                .par_iter()
                .map(|eval_instance| -> anyhow::Result<()> {
                    // Check if CHANGELOG.md already exists - if so, skip execution
                    let changelog_path = eval_instance.output().join("CHANGELOG.md");
                    if changelog_path.exists() {
                        // Already logged above, just skip silently
                        return Ok(());
                    }

                    // Read the current eval metadata
                    let mut run_metadata: RunMetaData = fs::read_to_string(
                        eval_instance.root().join("metadata.json"),
                    )
                    .map_err(anyhow::Error::from)
                    .and_then(|metadata_file| {
                        serde_json::from_str::<RunMetaData>(&metadata_file)
                            .map_err(anyhow::Error::from)
                    })?;

                    let result = Command::new("bash")
                        .arg("-lc")
                        .arg(format!("./{}", agent_config.entrypoint))
                        .current_dir(agent_dir)
                        .env("SCARF_WORK_DIR", eval_instance.output())
                        .env("SCARF_WORKDIR", eval_instance.output())
                        .env(
                            "SCARF_FRAMEWORK_FROM",
                            run_metadata.source_framework(),
                        )
                        .env(
                            "SCARF_FROM_FRAMEWORK",
                            run_metadata.source_framework(),
                        )
                        .env(
                            "SCARF_FRAMEWORK_TO",
                            run_metadata.target_framework(),
                        )
                        .env(
                            "SCARF_TO_FRAMEWORK",
                            run_metadata.target_framework(),
                        )
                        .env(
                            "SCARF_SOURCE_FRAMEWORK",
                            run_metadata.source_framework(),
                        )
                        .env(
                            "SCARF_TARGET_FRAMEWORK",
                            run_metadata.target_framework(),
                        )
                        .stderr(
                            File::create(
                                eval_instance.validation().join("agent.err"),
                            )?
                            .try_clone()?,
                        )
                        .stdout(
                            File::create(
                                eval_instance.validation().join("agent.out"),
                            )?
                            .try_clone()?,
                        )
                        .output()?;

                    if result.status.success() {
                        log::debug!(
                            "Agent {} execution complete",
                            eval_key.agent()
                        );
                        run_metadata.set_status(String::from("CONVERTED"));
                        update_eval_metadata(
                            eval_instance.root(),
                            &run_metadata,
                        )?;
                    } else {
                        run_metadata.set_status(String::from("FAILED"));
                        update_eval_metadata(
                            eval_instance.root(),
                            &run_metadata,
                        )?;
                    }

                    // Write individual cost file for this run if calculator is provided
                    if let Some(ref calc) = cost_calculator {
                        if let Ok(metadata_content) = fs::read_to_string(eval_instance.root().join("metadata.json")) {
                            if let Ok(metadata) = serde_json::from_str::<Metadata>(&metadata_content) {
                                let agent_out_path = eval_instance.validation().join("agent.out");
                                
                                // Create a temporary calculator for this single run
                                if let Ok(calc_lock) = calc.lock() {
                                    if let Some(model_costs) = calc_lock.get_model_costs() {
                                        let mut run_calc = ConversionCostCalculator::with_costs(model_costs.clone());
                                        run_calc.add_conversion(&metadata, &agent_out_path);
                                        run_calc.finalize_costs();
                                        
                                        // Write cost file to run directory
                                        let cost_file = eval_instance.root().join("costs.json");
                                        let json_output = run_calc.to_json();
                                        if let Err(e) = fs::write(&cost_file, json_output) {
                                            log::warn!("Failed to write cost file to {}: {}", cost_file.display(), e);
                                        } else {
                                            log::debug!("Cost file written to {}", cost_file.display());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Ok(())
                })
                .collect()
        });
    }
    Ok(())
}

fn update_eval_metadata(
    eval_instance_dir: PathBuf,
    run_metadata: &RunMetaData,
) -> anyhow::Result<()> {
    match File::create(eval_instance_dir.join("metadata.json")) {
        Ok(mut f) => {
            f.write_all(serde_json::to_string_pretty(run_metadata)?.as_bytes())?
        },
        Err(e) => {
            anyhow::bail!("Failed to update metadata.json {}", e);
        },
    };
    Ok(())
}
