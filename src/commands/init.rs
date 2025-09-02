use anyhow::{Context, Result};
use std::path::Path;
use crate::ui::{info, success, header, table};

pub fn run_init(
    source_dir: &Path,
    target_framework: &str,
    output_directory: &Path,
) -> Result<()> {
    info("Initializing destination stub...");
    
    // Validate inputs
    validate_inputs(source_dir, output_directory)?;
    
    // Always run locally - no Docker needed for template generation
    run_init_locally(source_dir, target_framework, output_directory)
}

fn validate_inputs(
    source_dir: &Path,
    output_directory: &Path,
) -> Result<()> {
    // Check if source directory exists
    if !source_dir.exists() {
        return Err(anyhow::anyhow!("Source directory not found: {}", source_dir.display()));
    }
    
    // Check if output directory can be created
    if let Some(parent) = output_directory.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory parent: {}", parent.display()))?;
        }
    }
    
    Ok(())
}

fn run_init_locally(source_dir: &Path, target_framework: &str, output_directory: &Path) -> Result<()> {
    info("Running init locally...");
    
    // Create output directory
    std::fs::create_dir_all(output_directory)
        .with_context(|| format!("Failed to create output directory: {}", output_directory.display()))?;
    
    // Copy source files to output directory
    copy_directory_recursive(source_dir, output_directory)
        .with_context(|| format!("Failed to copy source directory: {}", source_dir.display()))?;
    
    // Create .tmp and .home directories in output
    let tmp_dir = output_directory.join(".tmp");
    let home_dir = output_directory.join(".home");
    std::fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("Failed to create .tmp directory: {}", tmp_dir.display()))?;
    std::fs::create_dir_all(&home_dir)
        .with_context(|| format!("Failed to create .home directory: {}", home_dir.display()))?;
    
    // Display results
    display_init_results(source_dir, target_framework, output_directory);
    
    Ok(())
}

fn display_init_results(source_dir: &Path, target_framework: &str, output_directory: &Path) {
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
}

fn copy_directory_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Err(anyhow::anyhow!("Source directory does not exist: {}", src.display()));
    }
    
    if src.is_file() {
        // Copy single file
        std::fs::copy(src, dst)
            .with_context(|| format!("Failed to copy file {} to {}", src.display(), dst.display()))?;
    } else if src.is_dir() {
        // Create destination directory
        std::fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create directory: {}", dst.display()))?;
        
        // Copy directory contents recursively
        for entry in std::fs::read_dir(src)
            .with_context(|| format!("Failed to read directory: {}", src.display()))? {
            let entry = entry
                .with_context(|| format!("Failed to read directory entry in: {}", src.display()))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            copy_directory_recursive(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}
