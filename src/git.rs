use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

use crate::config::Resource;

/// Clones/updates a git repository and copies resources into it
///
/// # Arguments
/// * `config_path` - Path to the configuration file (used to locate resources)
/// * `repo_url` - Git repository URL to clone
/// * `branch` - Branch name to checkout
/// * `clean` - If true, creates a new directory; if false, uses current directory
/// * `merge` - If true, fetches and merges latest changes from upstream
/// * `verbose` - Enable verbose logging
/// * `resources` - List of resources to copy into the cloned repository
///
/// # Returns
/// Path to the cloned repository on success
pub fn checkout_repository(
    config_path: &str,
    repo_url: &str,
    branch: &str,
    clean: bool,
    merge: bool,
    verbose: bool,
    resources: &[Resource],
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let target_path = determine_target_path(clean)?;

    if clean {
        // Clean mode: always clone fresh
        clone_repository(repo_url, branch, &target_path, verbose)?;
    } else {
        // Non-clean mode: use existing or clone if missing
        if target_path.join(".git").exists() {
            if verbose {
                println!("Found existing repository at {}", target_path.display());
            }
            // Repository exists, optionally update it
            if merge {
                update_repository(branch, &target_path, verbose)?;
            }
        } else {
            // No repository exists, clone it
            clone_repository(repo_url, branch, &target_path, verbose)?;
        }
    }

    // Optionally merge updates in clean mode too
    if clean && merge {
        update_repository(branch, &target_path, verbose)?;
    }

    copy_resources(config_path, &target_path, resources, verbose)?;

    Ok(target_path)
}

/// Determines where to clone the repository based on clean flag
fn determine_target_path(clean: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if clean {
        // Create a UUID-named directory in the current working directory
        let uuid = Uuid::new_v4();
        let dir_name = uuid.to_string();
        Ok(std::env::current_dir()?.join(&dir_name))
    } else {
        // Use current directory directly - no cloning into subdirectory
        Ok(std::env::current_dir()?)
    }
}

/// Clones a git repository to the specified path
fn clone_repository(
    repo_url: &str,
    branch: &str,
    target_path: &Path,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!(
            "Cloning repository {} branch {} to {}",
            repo_url,
            branch,
            target_path.display()
        );
    }

    let status = Command::new("git")
        .arg("clone")
        .arg("--branch")
        .arg(branch)
        .arg("--progress")
        .arg(repo_url)
        .arg(target_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(format!(
            "Git clone failed with exit code: {}",
            status.code().unwrap_or(1)
        )
        .into());
    }

    Ok(())
}

/// Updates an existing repository by fetching and merging from upstream
fn update_repository(
    branch: &str,
    repo_path: &Path,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!(
            "Updating repository in {} (branch: {})",
            repo_path.display(),
            branch
        );
    }

    // Check for uncommitted changes
    let status_output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(repo_path)
        .output()?;

    if !status_output.stdout.is_empty() {
        return Err(
            "Repository has uncommitted changes. Commit or stash changes before updating.".into(),
        );
    }

    // Fetch latest changes
    if verbose {
        println!("Fetching latest changes from origin...");
    }

    let fetch_status = Command::new("git")
        .arg("fetch")
        .arg("--progress")
        .arg("origin")
        .arg(branch)
        .current_dir(repo_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !fetch_status.success() {
        return Err(format!(
            "Git fetch failed with exit code: {}",
            fetch_status.code().unwrap_or(1)
        )
        .into());
    }

    // Merge changes
    if verbose {
        println!("Merging changes from origin/{}...", branch);
    }

    let merge_status = Command::new("git")
        .arg("merge")
        .arg(format!("origin/{}", branch))
        .current_dir(repo_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !merge_status.success() {
        return Err(format!(
            "Git merge failed with exit code: {}",
            merge_status.code().unwrap_or(1)
        )
        .into());
    }

    if verbose {
        println!("Repository updated successfully");
    }

    Ok(())
}

/// Copies resources from config directory to target repository
fn copy_resources(
    config_path: &str,
    target_path: &Path,
    resources: &[Resource],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    for resource in resources {
        copy_single_resource(config_dir, target_path, resource, verbose)?;
    }

    Ok(())
}

/// Copies a single resource file
fn copy_single_resource(
    config_dir: &Path,
    target_path: &Path,
    resource: &Resource,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = config_dir.join("resources").join(&resource.file);
    let dest_path = target_path.join(resource.copy_path.as_ref().unwrap_or(&resource.file));

    // Validate paths to prevent traversal attacks
    validate_path(&source_path, &config_dir.join("resources"))?;
    validate_path(&dest_path, target_path)?;

    if verbose {
        println!(
            "Copying resource: {} -> {}",
            source_path.display(),
            dest_path.display()
        );
    }

    // Create destination directory if it doesn't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&source_path, &dest_path)?;

    Ok(())
}

/// Validates that a path doesn't escape its intended base directory
fn validate_path(path: &Path, base: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let canonical_path = path.canonicalize().or_else(|_| {
        // If path doesn't exist yet, validate parent
        if let Some(parent) = path.parent() {
            parent.canonicalize()
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Invalid path",
            ))
        }
    })?;

    let canonical_base = base.canonicalize()?;

    if !canonical_path.starts_with(&canonical_base) {
        return Err(format!(
            "Path traversal attempt detected: {} is outside {}",
            path.display(),
            base.display()
        )
        .into());
    }

    Ok(())
}

/// Executes a deployment tool in the repository directory
///
/// # Arguments
/// * `tool_name` - Name or path of the tool to execute
/// * `arguments` - Arguments to pass to the tool
/// * `repo_path` - Path to the repository where the tool should run
/// * `verbose` - Enable verbose logging
///
/// # Returns
/// Exit code of the tool execution
pub fn execute_tool(
    tool_name: &str,
    arguments: &[String],
    repo_path: &Path,
    verbose: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    if tool_name.is_empty() {
        return Ok(0); // Nothing to execute
    }

    if verbose {
        if arguments.is_empty() {
            println!(
                "Executing tool '{}' in {}",
                tool_name,
                repo_path.display()
            );
        } else {
            println!(
                "Executing tool '{}' with args {:?} in {}",
                tool_name,
                arguments,
                repo_path.display()
            );
        }
    }

    let status = Command::new(tool_name)
        .args(arguments)
        .current_dir(repo_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    let exit_code = status.code().unwrap_or(1);

    if verbose {
        println!("Tool '{}' exited with code: {}", tool_name, exit_code);
    }

    Ok(exit_code)
}
