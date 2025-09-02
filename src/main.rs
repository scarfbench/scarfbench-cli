mod ui;
mod commands;

use anyhow::Result;
use clap::{ Parser, Subcommand };
use owo_colors::OwoColorize;


#[derive(Parser)]
#[command(name = "scarf", version, about = "🧣 SCARF benchmark CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// At runtime, clap will match the first keyword after scarf [init|run|test] to the following variants
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
        /// Optional: Dockerfile to build the container from
        #[arg(long)]
        dockerfile: Option<std::path::PathBuf>,
        /// Optional: Docker build context directory
        #[arg(long)]
        docker_build_context: Option<std::path::PathBuf>,
        /// Optional: Custom container name (default: auto-generated)
        #[arg(long)]
        docker_name: Option<String>,
        /// Optional: environment variables for Docker (KEY=VALUE). Can repeat.
        #[arg(long = "env", value_parser = clap::builder::NonEmptyStringValueParser::new())]
        docker_env: Vec<String>,
        /// Docker network mode (default: none). Examples: none, bridge, host
        #[arg(long, default_value = "none")]
        docker_network: String,
        /// Port forwarding (HOST:CONTAINER). Can repeat for multiple ports. Example: 8080:8080
        #[arg(long = "port", value_parser = clap::builder::NonEmptyStringValueParser::new())]
        docker_ports: Vec<String>,
        /// Keep container after run (default: false -> --rm)
        #[arg(long)]
        docker_no_rm: bool,
        /// Extra args to pass to agent after `--`
        #[arg(last = true)]
        extra: Vec<String>,
    },
    /// Test Docker container setup without running scarf commands
    Test {
        #[arg(long)]
        from: std::path::PathBuf,
        #[arg(long, default_value = "generated")]
        out: std::path::PathBuf,
        /// Optional: Docker image to test
        #[arg(long)]
        docker_image: Option<String>,
        /// Optional: Dockerfile to build the container from
        #[arg(long)]
        dockerfile: Option<std::path::PathBuf>,
        /// Optional: Docker build context directory
        #[arg(long)]
        docker_build_context: Option<std::path::PathBuf>,
        /// Optional: Custom container name (default: auto-generated)
        #[arg(long)]
        docker_name: Option<String>,
        /// Optional: environment variables for Docker (KEY=VALUE). Can repeat.
        #[arg(long = "env", value_parser = clap::builder::NonEmptyStringValueParser::new())]
        docker_env: Vec<String>,
        /// Docker network mode (default: none). Examples: none, bridge, host
        #[arg(long, default_value = "none")]
        docker_network: String,
        /// Port forwarding (HOST:CONTAINER). Can repeat for multiple ports. Example: 8080:8080
        #[arg(long = "port", value_parser = clap::builder::NonEmptyStringValueParser::new())]
        docker_ports: Vec<String>,
        /// Keep container after run (default: false -> --rm)
        #[arg(long)]
        docker_no_rm: bool,
        /// Command to run in the container (default: tail -f /dev/null to keep container alive)
        #[arg(long, default_value = "tail -f /dev/null")]
        command: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{} {}", ui::SCARF, "Welcome to SCARF!".bright_magenta().bold());

    match cli.command {
        Commands::Init { source_dir, target_framework, output_dir } =>
            commands::init::run_init(&source_dir, &target_framework, &output_dir),
        Commands::Run { agent, from, to, out, docker_image, dockerfile, docker_build_context, docker_name, docker_env, docker_network, docker_ports, docker_no_rm, extra } =>
            commands::run::run_agent(&agent, &from, &to, &out, docker_image.as_deref(), dockerfile.as_deref(), docker_build_context.as_deref(), docker_name.as_deref(), &docker_env, &docker_network, &docker_ports, docker_no_rm, &extra),
        Commands::Test { from, out, docker_image, dockerfile, docker_build_context, docker_name, docker_env, docker_network, docker_ports, docker_no_rm, command } =>
            commands::test::run_test(&from, &out, docker_image.as_deref(), dockerfile.as_deref(), docker_build_context.as_deref(), docker_name.as_deref(), &docker_env, &docker_network, &docker_ports, docker_no_rm, &command),
    }
}
