use anyhow::Result;
use crate::ui::{ error, info };
use std::path::{ Path };

pub fn run_agent(
    agent: &Path,
    from: &Path,
    to_fw: &str,
    out_root: &Path,
    extra: &[String]
) -> Result<()> {
    error("Implement agent runner.");
    info("Run CHECK.sh in the destination to verify.");
    Ok(())
}
