use anyhow::{Result, Context, anyhow};
use crate::ui::{error, info, success, warning};
use std::path::Path;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::fs;

use uuid::Uuid;

pub fn run_test(
    from: &Path,
    out_root: &Path,
    docker_image: Option<&str>,
    dockerfile: Option<&Path>,
    docker_build_context: Option<&Path>,
    docker_name: Option<&str>,
    docker_env: &[String],
    docker_network: &str,
    docker_ports: &[String],
    docker_no_rm: bool,
    command: &str,
) -> Result<()> {
    info("Testing Docker container setup...");
    
    // Validate inputs
    validate_inputs(from, out_root, docker_image, dockerfile, docker_build_context)?;
    
    // Determine if we need to use Docker
    let use_docker = docker_image.is_some() || dockerfile.is_some();
    
    if use_docker {
        run_test_in_docker(
            from, out_root,
            docker_image, dockerfile, docker_build_context, docker_name,
            docker_env, docker_network, docker_ports, docker_no_rm, command
        )
    } else {
        Err(anyhow!("Docker is required for test command. Please specify --docker-image or --dockerfile"))
    }
}

fn validate_inputs(
    from: &Path,
    out_root: &Path,
    docker_image: Option<&str>,
    dockerfile: Option<&Path>,
    docker_build_context: Option<&Path>,
) -> Result<()> {
    // Check if source directory exists
    if !from.exists() {
        return Err(anyhow!("Source directory not found: {}", from.display()));
    }
    
    // Check if output directory can be created
    if let Some(parent) = out_root.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory parent: {}", parent.display()))?;
        }
    }
    
    // Validate Docker configuration
    if let Some(dockerfile_path) = dockerfile {
        if !dockerfile_path.exists() {
            return Err(anyhow!("Dockerfile not found: {}", dockerfile_path.display()));
        }
    }
    
    if let Some(build_context) = docker_build_context {
        if !build_context.exists() {
            return Err(anyhow!("Docker build context not found: {}", build_context.display()));
        }
    }
    
    Ok(())
}

fn container_exists(container_name: &str) -> Result<bool> {
    let output = Command::new("docker")
        .args(&["ps", "-a", "--filter", &format!("name=^{}$", container_name), "--format", "{{.Names}}"])
        .output()
        .with_context(|| "Failed to check if container exists")?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == container_name)
}

fn run_test_in_docker(
    from: &Path,
    out_root: &Path,
    docker_image: Option<&str>,
    dockerfile: Option<&Path>,
    docker_build_context: Option<&Path>,
    docker_name: Option<&str>,
    docker_env: &[String],
    docker_network: &str,
    docker_ports: &[String],
    docker_no_rm: bool,
    command: &str,
) -> Result<()> {
    info("Testing Docker container setup...");
    
    // Determine the Docker image to use
    let image_name = if let Some(image) = docker_image {
        image.to_string()
    } else if let Some(dockerfile_path) = dockerfile {
        // Build image from Dockerfile
        build_docker_image(dockerfile_path, docker_build_context)?
    } else {
        return Err(anyhow!("Either docker_image or dockerfile must be specified"));
    };
    
    // Generate container name if not provided
    let container_name = docker_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("scarf-test-{}", Uuid::new_v4().simple()));
    
    // Check if container already exists
    if container_exists(&container_name)? {
        if let Some(_) = docker_name {
            // User specified a custom name that conflicts
            warning(&format!("Container '{}' already exists. You can:", container_name));
            warning("  1. Remove the existing container: docker rm <container-name>");
            warning("  2. Use a different name with --docker-name");
            warning("  3. Let scarf generate a unique name (omit --docker-name)");
            return Err(anyhow!("Container name '{}' is already in use", container_name));
        } else {
            // Auto-generated name conflicts (very unlikely, but handle it)
            let new_name = format!("scarf-test-{}", Uuid::new_v4().simple());
            info(&format!("Container name conflict detected. Using new name: {}", new_name));
            return run_test_in_docker(
                from, out_root,
                docker_image, dockerfile, docker_build_context,
                Some(&new_name), docker_env, docker_network, docker_ports, docker_no_rm, command
            );
        }
    }
    
    // Prepare environment variables
    let env_vars = parse_env_vars(docker_env)?;
    
    // Create output directory
    std::fs::create_dir_all(out_root)
        .with_context(|| format!("Failed to create output directory: {}", out_root.display()))?;
    
    // Copy source files to output directory so agent has full context
    copy_directory_recursive(from, out_root)
        .with_context(|| format!("Failed to copy source directory: {}", from.display()))?;
    
    // Create .tmp and .home directories in output
    let tmp_dir = out_root.join(".tmp");
    let home_dir = out_root.join(".home");
    std::fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("Failed to create .tmp directory: {}", tmp_dir.display()))?;
    std::fs::create_dir_all(&home_dir)
        .with_context(|| format!("Failed to create .home directory: {}", home_dir.display()))?;
    
    // Build Docker run command
    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    
    // Container name
    docker_cmd.args(&["--name", &container_name]);
    
    // Network mode
    if docker_network != "none" {
        docker_cmd.args(&["--network", docker_network]);
    }
    
    // Remove container after run (opposite of docker_no_rm)
    if !docker_no_rm {
        docker_cmd.arg("--rm");
    }
    
    // Environment variables
    for (key, value) in &env_vars {
        docker_cmd.args(&["-e", &format!("{}={}", key, value)]);
    }
    
    // Port forwarding
    for port_mapping in docker_ports {
        docker_cmd.args(&["-p", port_mapping]);
    }
    
    // Volume mounts
    let source_abs = from.canonicalize()
        .with_context(|| format!("Failed to canonicalize source path: {}", from.display()))?;
    let output_abs = out_root.canonicalize()
        .with_context(|| format!("Failed to canonicalize output path: {}", out_root.display()))?;
    
    // Mount source directory (read-only)
    docker_cmd.args(&["-v", &format!("{}:/source:ro", source_abs.display())]);
    
    // Mount output directory (read-write)
    docker_cmd.args(&["-v", &format!("{}:/output", output_abs.display())]);
    
    // Mount .tmp directory (canonicalize to absolute path)
    let tmp_abs = tmp_dir.canonicalize()
        .with_context(|| format!("Failed to canonicalize .tmp path: {}", tmp_dir.display()))?;
    docker_cmd.args(&["-v", &format!("{}:/tmp", tmp_abs.display())]);
    
    // Mount .home directory (canonicalize to absolute path)
    let home_abs = home_dir.canonicalize()
        .with_context(|| format!("Failed to canonicalize .home path: {}", home_dir.display()))?;
    docker_cmd.args(&["-v", &format!("{}:/home", home_abs.display())]);
    
    // Set working directory to output
    docker_cmd.args(&["-w", "/output"]);
    
    // Add detached mode flag BEFORE the image name
    docker_cmd.arg("-d");
    
    // Image name
    docker_cmd.arg(&image_name);
    
    // Command to run - use shell for complex commands
    if command.contains(" ") || command.contains("'") || command.contains("\"") {
        docker_cmd.args(&["sh", "-c", command]);
    } else {
        docker_cmd.arg(command);
    }
    
    info(&format!("Testing Docker container with command: {:?}", docker_cmd));
    
    info(&format!("Starting Docker container with command: {:?}", docker_cmd));
    
    let output = docker_cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| "Failed to execute Docker command")?;
    
    if output.status.success() {
        success("Docker container started successfully!");
        info("Container is now running in the background.");
        println!();
        info("To access the container:");
        println!("  docker exec -it {} bash", container_name);
        println!();
        info("To stop the container:");
        println!("  docker stop {}", container_name);
        println!();
        info("To remove the container:");
        println!("  docker rm {}", container_name);
        println!();
        info("Container setup is working correctly!");
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error(&format!("Docker container test failed: {}", error_msg));
        Err(anyhow!("Docker container test failed with exit code: {}", output.status))
    }
}

fn build_docker_image(dockerfile: &Path, build_context: Option<&Path>) -> Result<String> {
    info("Building Docker image from Dockerfile...");
    
    let context_path = build_context
        .unwrap_or_else(|| dockerfile.parent().unwrap_or(Path::new(".")));
    
    let image_name = format!("scarf-test-{}", Uuid::new_v4().simple());
    
    let mut docker_cmd = Command::new("docker");
    docker_cmd
        .arg("build")
        .arg("-f")
        .arg(dockerfile)
        .arg("-t")
        .arg(&image_name)
        .arg(context_path);
    
    info(&format!("Building image with command: {:?}", docker_cmd));
    
    let output = docker_cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| "Failed to build Docker image")?;
    
    if output.status.success() {
        success(&format!("Docker image built successfully: {}", image_name));
        Ok(image_name)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error(&format!("Docker build failed: {}", error_msg));
        Err(anyhow!("Docker build failed with exit code: {}", output.status))
    }
}

fn parse_env_vars(env_vars: &[String]) -> Result<HashMap<String, String>> {
    let mut env_map = HashMap::new();
    
    for env_var in env_vars {
        if let Some(equal_pos) = env_var.find('=') {
            let key = env_var[..equal_pos].to_string();
            let value = env_var[equal_pos + 1..].to_string();
            env_map.insert(key, value);
        } else {
            return Err(anyhow!("Invalid environment variable format: {}. Expected KEY=VALUE", env_var));
        }
    }
    
    Ok(env_map)
}

fn copy_directory_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Err(anyhow!("Source directory does not exist: {}", src.display()));
    }
    
    if src.is_file() {
        // Copy single file
        fs::copy(src, dst)
            .with_context(|| format!("Failed to copy file {} to {}", src.display(), dst.display()))?;
    } else if src.is_dir() {
        // Create destination directory
        fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create directory: {}", dst.display()))?;
        
        // Copy directory contents recursively
        for entry in fs::read_dir(src)
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
