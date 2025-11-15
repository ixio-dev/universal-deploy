use clap::{Arg, ArgAction, Command};

/// Builds the CLI command structure
pub fn build_command() -> Command {
    Command::new("universal-deploy")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool to run deployment based on configuration files")
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_parser(clap::value_parser!(clap_complete::Shell))
                        .help("Shell to generate completions for"),
                ),
        )
        .arg(
            Arg::new("config")
                .value_name("FILE")
                .help("Path to the configuration file")
                .required(true),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("keep-checkout")
                .long("keep-checkout")
                .help("Keep the checkout directory after deployment (only applies to clean mode)")
                .action(ArgAction::SetTrue),
        )
}

/// Builds a CLI command for the completion subcommand
/// This variant doesn't require the config argument
pub fn build_command_for_completion() -> Command {
    Command::new("universal-deploy")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool to run deployment based on configuration files")
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_parser(clap::value_parser!(clap_complete::Shell))
                        .help("Shell to generate completions for"),
                ),
        )
        .arg(
            Arg::new("config")
                .value_name("FILE")
                .help("Path to the configuration file")
                .required(false), // Not required for completion generation
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("keep-checkout")
                .long("keep-checkout")
                .help("Keep the checkout directory after deployment (only applies to clean mode)")
                .action(ArgAction::SetTrue),
        )
}

/// Checks if the first command-line argument is "completion"
pub fn is_completion_command() -> bool {
    std::env::args()
        .nth(1)
        .map(|arg| arg == "completion")
        .unwrap_or(false)
}

/// Generates shell completion scripts
pub fn generate_completion(shell: clap_complete::Shell) {
    let mut cmd = build_command();
    clap_complete::generate(shell, &mut cmd, "ud", &mut std::io::stdout());
}
