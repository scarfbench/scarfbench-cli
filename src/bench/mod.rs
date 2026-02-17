pub mod list;
pub mod test;

use anyhow::Result;
use clap::Subcommand;

use crate::bench::list::BenchListArgs;
use crate::bench::test::BenchTestArgs;
use crate::utils::logo;

#[derive(Subcommand, Debug)]
pub enum BenchCmd {
    #[command(before_help = logo(), about = "List the application(s) in the benchmark.")]
    List(BenchListArgs),
    #[command(
        before_help = logo(),
        about = "Run regression tests (with `make test`) on the benchmark application(s)."
    )]
    Test(BenchTestArgs),
}

pub fn run(cmd: BenchCmd) -> Result<i32> {
    match cmd {
        BenchCmd::List(args) => list::run(args),
        BenchCmd::Test(args) => test::run(args),
    }
}
