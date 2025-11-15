use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Top-level configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub release: ReleaseConfig,
}

/// Release configuration settings
#[derive(Debug, Deserialize, Serialize)]
pub struct ReleaseConfig {
    /// Whether to create a clean checkout in a new directory
    #[serde(default)]
    pub clean: bool,

    /// Git repository URL
    #[serde(default)]
    pub repository: String,

    /// Branch to checkout
    #[serde(default)]
    pub branch: String,

    /// Whether to merge after deployment (not yet implemented)
    #[serde(default)]
    pub merge: bool,

    /// List of resource files to copy
    #[serde(default)]
    pub resources: Vec<Resource>,

    /// Tool to use for deployment
    #[serde(default)]
    pub tool: ToolConfig,

    /// Whether to create a git tag (not yet implemented)
    #[serde(default)]
    pub tag: bool,
}

/// Tool configuration for deployment
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ToolConfig {
    /// Full configuration with command and arguments
    Full {
        command: String,
        #[serde(default)]
        arguments: Vec<String>,
    },
    /// Simple string for command without arguments
    Simple(String),
}

impl Default for ToolConfig {
    fn default() -> Self {
        ToolConfig::Simple(String::new())
    }
}

impl ToolConfig {
    /// Returns the command name, if any
    pub fn command(&self) -> Option<&str> {
        match self {
            ToolConfig::Simple(cmd) if cmd.is_empty() => None,
            ToolConfig::Simple(cmd) => Some(cmd.as_str()),
            ToolConfig::Full { command, .. } if command.is_empty() => None,
            ToolConfig::Full { command, .. } => Some(command.as_str()),
        }
    }

    /// Returns the arguments list
    pub fn arguments(&self) -> &[String] {
        match self {
            ToolConfig::Full { arguments, .. } => arguments.as_slice(),
            _ => &[],
        }
    }

    /// Checks if tool is configured
    pub fn is_empty(&self) -> bool {
        self.command().is_none()
    }
}

impl std::fmt::Display for ToolConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolConfig::Simple(cmd) => write!(f, "{}", cmd),
            ToolConfig::Full { command, arguments } => {
                write!(f, "{}", command)?;
                if !arguments.is_empty() {
                    write!(f, " {}", arguments.join(" "))?;
                }
                Ok(())
            }
        }
    }
}

/// Resource file to copy into the deployment
#[derive(Debug, Deserialize, Serialize)]
pub struct Resource {
    /// Source file name (relative to resources directory)
    pub file: String,

    /// Optional destination path (defaults to same as file)
    #[serde(rename = "copy")]
    pub copy_path: Option<String>,
}

impl Config {
    /// Loads configuration from a YAML file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration YAML file
    ///
    /// # Returns
    /// Parsed configuration or error
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !Path::new(path).exists() {
            return Err(format!("Config file does not exist: {}", path).into());
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Could not read config file: {}", e))?;

        let config: Config = serde_yaml::from_str(&contents)
            .map_err(|e| format!("Failed to parse YAML config file: {}", e))?;

        Ok(config)
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.release.repository.is_empty() {
            return Err("Repository URL cannot be empty".into());
        }

        if self.release.branch.is_empty() {
            return Err("Branch name cannot be empty".into());
        }

        Ok(())
    }

    /// Prints configuration summary to stdout
    pub fn print_summary(&self, verbose: bool) {
        if verbose {
            println!("Successfully parsed config: {:#?}", self);
        } else {
            println!("Release configuration:");
            println!("  Clean: {}", self.release.clean);
            println!("  Repository: {}", self.release.repository);
            println!("  Branch: {}", self.release.branch);
            println!("  Merge: {}", self.release.merge);
            println!("  Tool: {}", self.release.tool);
            println!("  Tag: {}", self.release.tag);
            println!("  Resources: {} items", self.release.resources.len());
            for (i, resource) in self.release.resources.iter().enumerate() {
                println!("    [{}]: file='{}'", i, resource.file);
                if let Some(copy) = &resource.copy_path {
                    println!("         copy='{}'", copy);
                }
            }
        }
    }
}
