# Universal Deploy

A tool to run deployment based on configuration files.

## Installation

```bash
cargo build --release
# Binary will be available at target/release/ud
```

## Usage

```bash
ud [OPTIONS] <FILE>
```

### Options
- `-v`, `--verbose`: Enable verbose output
- `--keep-checkout`: Keep the checkout directory after deployment (only applies to clean mode)
- `--help`: Show help information

### Subcommands
- `completion <SHELL>`: Generate shell completion scripts

## Configuration File Format

The configuration file is in YAML format with the following structure:

```yaml
release:
  # Whether to create a clean checkout in a new directory (default: false)
  clean: true

  # Git repository URL (required)
  repository: "https://github.com/user/repo.git"

  # Branch to checkout (required)
  branch: "main"

  # Whether to merge after deployment (not yet implemented) (default: false)
  merge: false

  # Whether to create a git tag (not yet implemented) (default: false)
  tag: false

  # Tool to use for deployment (optional)
  tool:
    command: "deploy-tool"
    arguments: ["--env", "production"]

  # Alternative simple tool format
  # tool: "deploy-tool"

  # List of resource files to copy (optional)
  resources:
    - file: "config.json"
      copy: "dest-config.json"  # optional destination path
    - file: "secrets.env"
```

### Resources
Resource files are expected to be located in a `resources/` directory relative to the configuration file. The `copy` field is optional and specifies the destination path within the cloned repository.

### Tool Configuration
The `tool` section can be specified in two ways:
- **Full configuration**: With separate `command` and `arguments` fields
- **Simple configuration**: A single string with the command name

If no tool is specified, the process will only clone/update the repository and copy resources.