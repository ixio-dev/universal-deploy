mod cli;
mod config;
mod git;

use config::Config;

fn main() {
    // Check if running the completion subcommand
    let cmd = if cli::is_completion_command() {
        cli::build_command_for_completion()
    } else {
        cli::build_command()
    };

    let matches = cmd.get_matches();

    // Handle completion subcommand
    if let Some(("completion", sub_matches)) = matches.subcommand() {
        let shell: clap_complete::Shell = *sub_matches
            .get_one("shell")
            .expect("Shell is required");

        cli::generate_completion(shell);
        return;
    }

    // Handle normal deployment operation
    match run_deployment(&matches) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Executes the deployment workflow
fn run_deployment(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = matches
        .get_one::<String>("config")
        .expect("Config file is required");
    let verbose = matches.get_flag("verbose");
    let keep_checkout = matches.get_flag("keep-checkout");

    if verbose {
        println!("Reading configuration from: {}", config_path);
    }

    // Load and validate configuration
    let config = Config::load(config_path)?;
    config.validate()?;

    if verbose {
        config.print_summary(true);
    } else {
        println!("Configuration loaded successfully from {}", config_path);
        config.print_summary(false);
    }

    // Perform repository checkout
    let repo_path = git::checkout_repository(
        config_path,
        &config.release.repository,
        &config.release.branch,
        config.release.clean,
        config.release.merge,
        verbose,
        &config.release.resources,
    )?;

    if verbose {
        println!(
            "Repository successfully checked out to: {}",
            repo_path.display()
        );
    } else {
        println!("Repository checked out successfully");
    }

    // Execute deployment tool if specified
    let tool_result = if let Some(command) = config.release.tool.command() {
        if verbose {
            println!("Executing tool: '{}'", config.release.tool);
        }
        let exit_code = git::execute_tool(
            command,
            config.release.tool.arguments(),
            &repo_path,
            verbose,
        )?;
        if exit_code != 0 {
            Err(format!(
                "Tool '{}' failed with exit code {}",
                config.release.tool, exit_code
            )
            .into())
        } else {
            if verbose {
                println!("Tool execution completed successfully");
            }
            Ok(())
        }
    } else {
        Ok(())
    };

    // Cleanup checkout directory if in clean mode and not keeping it
    if config.release.clean && !keep_checkout {
        if let Err(e) = std::fs::remove_dir_all(&repo_path) {
            eprintln!("Warning: Failed to remove checkout directory: {}", e);
        } else if verbose {
            println!("Removed checkout directory: {}", repo_path.display());
        }
    } else if config.release.clean && verbose {
        println!("Keeping checkout directory: {}", repo_path.display());
    }

    tool_result
}
