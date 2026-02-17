mod driver;
mod prepare;
mod types;

pub(crate) mod run;

use crate::utils::logo;
use clap::Subcommand;
use run::EvalRunArgs;

#[derive(Subcommand, Debug)]
pub enum EvalCmd {
    #[command(before_help = logo(), about = "Evaluate an agent on Scarfbench")]
    Run(EvalRunArgs),
}

pub fn run(cmd: EvalCmd) -> anyhow::Result<i32> {
    match cmd {
        EvalCmd::Run(args) => run::run(args),
    }
}
