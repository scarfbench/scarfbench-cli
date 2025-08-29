mod ui;
mod commands;

use anyhow::Result;
use clap::{ Parser, Subcommand };
use owo_colors::OwoColorize;
use ui::ROCKET;

#[derive(Parser)]
#[command(name = "scarf", version, about = "🧣 SCARF benchmark CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// At runtime, clap will match the first keyword after scarf [init|run] to the following variants
#[derive(Subcommand)]
enum Commands {
    /// Create a destination stub (templates) for a target framework
    Init {
        #[arg(long)]
        source_dir: std::path::PathBuf,
        #[arg(long)]
        target_framework: String,
        #[arg(long, default_value = "generated")]
        output_dir: std::path::PathBuf,
    },
    /// Run an agent to perform a transformation
    Run {
        #[arg(long)]
        agent: std::path::PathBuf,
        #[arg(long)]
        from: std::path::PathBuf,
        #[arg(long)]
        to: String,
        #[arg(long, default_value = "generated")]
        out: std::path::PathBuf,
        /// Optional: Docker image to run the agent in
        #[arg(long)]
        docker_image: Option<String>,
        /// Optional: environment variables for Docker (KEY=VALUE). Can repeat.
        #[arg(long = "env", value_parser = clap::builder::NonEmptyStringValueParser::new())]
        docker_env: Vec<String>,
        /// Docker network mode (default: none). Examples: none, bridge, host
        #[arg(long, default_value = "none")]
        docker_network: String,
        /// Keep container after run (default: false -> --rm)
        #[arg(long)]
        docker_no_rm: bool,
        /// Extra args to pass to agent after `--`
        #[arg(last = true)]
        extra: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{} {}", ui::SCARF, "Welcome to SCARF!".bright_magenta().bold());

    match cli.command {
        Commands::Init { source_dir, target_framework, output_dir } =>
            commands::init::run_init(&source_dir, &target_framework, &output_dir),
        Commands::Run { agent, from, to, out, docker_image, docker_env, docker_network, docker_no_rm, extra } =>
            commands::run::run_agent(&agent, &from, &to, &out, docker_image.as_deref(), &docker_env, &docker_network, docker_no_rm, &extra),
    }
}
