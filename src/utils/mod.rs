pub mod progress_bar;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub(crate) fn logo() -> String {
    format!(
        "\x1b[1m\x1b[31m{}\x1b[0m",
        r#"
 ███████╗ ██████╗ █████╗ ██████╗ ███████╗██████╗ ███████╗███╗   ██╗ ██████╗██╗  ██╗
 ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔════╝████╗  ██║██╔════╝██║  ██║
 ███████╗██║     ███████║██████╔╝█████╗  ██████╔╝█████╗  ██╔██╗ ██║██║     ███████║
 ╚════██║██║     ██╔══██║██╔══██╗██╔══╝  ██╔══██╗██╔══╝  ██║╚██╗██║██║     ██╔══██║
 ███████║╚██████╗██║  ██║██║  ██║██║     ██████╔╝███████╗██║ ╚████║╚██████╗██║  ██║
 ╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚═╝  ╚═╝"#
            .bold()
            .red()
    )
}

/// This will create a `.scarfbench` directory at $HOME wherein we have two subfolders
/// ├── benchmarks/
/// └── runs/
///     ├── 2026-03-04T18-22-11Z/
///     │   ├── eval_out/           # what I am currently call eval_out
///     │   └── results.json        # a speculative placeholder to put leaderboard (might remove this later)
pub(crate) fn get_or_create_and_get_scarfbench_home_dir() -> Result<PathBuf> {
    // Find the user home directory and create a directory called .scarfbench there it if doesn't exist.
    // if it exists, then just return that directory
    let scarfbench_home_dir = dirs::home_dir()
        .expect("Unable to find home directory")
        .join(".scarfbench");

    // It doesnt seem to exist, so create it
    // if !scarfbench_home_dir.exists() {
    log::debug!(
        "Creating Scarfbench home directory at {}",
        scarfbench_home_dir.to_string_lossy()
    );
    // Create the base directory at ~/.scarfbench
    std::fs::create_dir_all(&scarfbench_home_dir)
        .expect("Unable to create scarfbench home directory at ~/.scarfbench");
    //
    vec!["benchmark", "evals", "logs"]
        .into_iter()
        .map(|p| scarfbench_home_dir.join(p))
        .try_for_each(std::fs::create_dir_all)?;
    // }
    Ok(scarfbench_home_dir)
}

/// Here's where we'll save the logs
#[allow(unused)]
pub(crate) fn get_logs_dir() -> Result<PathBuf> {
    Ok(get_or_create_and_get_scarfbench_home_dir()?.join("logs"))
}
