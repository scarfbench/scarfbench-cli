use anyhow::Result;
use clap::Parser;

use crate::utils::get_or_create_and_get_scarfbench_home_dir;

mod bench;
mod cli;
mod eval;
mod utils;
mod validate;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    // Setup logging mechanism
    init_logging(cli.verbose);
    // Create workdir under $HOME directory. Unless the user overrrides this, here's where the assets will be downloaded to.
    let _ = get_or_create_and_get_scarfbench_home_dir()?;

    let code = match cli.command {
        cli::Commands::Bench(cmd) => bench::run(cmd)?,
        cli::Commands::Eval(cmd) => eval::run(cmd)?,
        cli::Commands::Validate(args) => validate::run(args)?,
    };
    std::process::exit(code);
}

fn init_logging(verbose: u8) {
    // Use `RUST_LOG` if set; otherwise default to `warn`, and let `-v/-vv/-vvv` raise verbosity.
    let default_filter = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(default_filter),
    )
    .init();
}
