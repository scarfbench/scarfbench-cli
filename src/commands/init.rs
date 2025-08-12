use anyhow::Result;
use crate::ui::{ header, info, success, table };
use std::path::{ Path };

pub fn run_init(source_dir: &Path, target_framework: &str, output_directory: &Path) -> Result<()> {
    header("INIT RESULT");
    success(&format!("Stub initialized at {}", output_directory.display()));
    println!(
        "{}",
        table(
            &["App", "From (fw)", "To (fw)"],
            &[
                vec![
                    source_dir.to_str().unwrap_or("").to_string(),
                    target_framework.to_string(),
                    output_directory.to_str().unwrap_or("").to_string()
                ],
            ]
        )
    );

    info("Successfully initialized transformation artifacts.");
    Ok(())
}
