# rustified

A command-line Minecraft launcher written in Rust.

`rustified` provides a text-based interface for launching and managing Minecraft instances, designed for automation, scripting, and environments where a graphical launcher isn't suitable or preferred.

## Features

Based on the current implementation and planned features:

*   **Microsoft Account Authentication:** Securely authenticate using the standard Microsoft/Xbox Live flow.
*   **Instance Management:** Create, list, configure, and delete isolated Minecraft instances.
*   **Multi-Version Support:** Launch different Minecraft versions using their specific requirements.
*   **Automatic File Management:** Downloads required game JARs, libraries, and assets with integrity verification (SHA1).
*   **Automatic Java Detection:** Scans for and selects the appropriate Java Runtime Environment (JRE) based on the Minecraft version.
*   **Cross-Platform:** Designed to work on Windows, macOS, and Linux.
*   **Customizable:** Configure instance memory allocation and other settings.

## Installation

### Prerequisites

*   [Rust](https://rustup.rs/) (latest stable recommended)
*   Git

### Building from source

1.  Clone the repository:
    ```bash
    git clone https://github.com/OmarAfet/rustified.git
    cd rustified
    ```
2.  Build the project:
    ```bash
    cargo build --release
    ```
    The executable will be located at `./target/release/rustified`.

## Usage

Run `./target/release/rustified --help` to see the full list of commands.

```bash
./target/release/rustified --help
```

Here are some common commands:

### List Available Versions

Fetches and displays a list of available Minecraft versions from the Mojang manifest.

```bash
./target/release/rustified list
```

You can filter by type, search, limit results, and more:

```bash
./target/release/rustified list --limit 20 --types release snapshot --filter 1.20 --show-installed
```

### Authentication

`rustified` uses Microsoft account authentication. The `auth` command allows you to manage your login session. You will be prompted to authenticate via your web browser on the first `launch` attempt if no valid session is found.

```bash
# Check current authentication status
./target/release/rustified auth status

# Clear cached authentication
./target/release/rustified auth clear

# Force a re-authentication flow
./target/release/rustified auth refresh
```

### Instance Management

Instances provide isolated game environments.

```bash
# List all instances
./target/release/rustified instance list

# Create a new instance named 'my-world' using Minecraft version '1.20.4'
# Replace '1.20.4' with a version from 'rustified list'
./target/release/rustified instance create my-world 1.20.4 --description "My main survival world"

# Show details for an instance
./target/release/rustified instance info my-world

# Set memory allocation (in MB) for an instance (e.g., 4GB)
./target/release/rustified instance memory my-world 4096

# Delete an instance
./target/release/rustified instance delete my-world
```

### Prepare Game Files

Download the necessary files for a specific Minecraft version without launching the game. This is useful for pre-downloading.

```bash
# Prepare game files for version 1.20.4
./target/release/rustified prepare 1.20.4
```

### Launch Game

Launch a specific instance. If it's the first time launching this instance or version, it will automatically perform authentication and download required files.

```bash
# Launch the 'my-world' instance
./target/release/rustified launch my-world
```

### Java Management

Check detected Java installations and recommended versions.

```bash
# List detected Java installations
./target/release/rustified java list

# Show recommended Java version for a Minecraft version
./target/release/rustified java recommend 1.20.4
```

## Contributing

Contributions are welcome! Please see the [`CONTRIBUTING.md`](CONTRIBUTING.md) file for guidelines on how to contribute, set up your development environment, and run checks.

We have a strict [Dead Code Policy](CONTRIBUTING.md#dead-code-policy). Please ensure your code passes `cargo clippy --all-targets --all-features -- -D warnings -D dead_code` before submitting a Pull Request. The `./scripts/prepare-pr.sh` script can help automate checks.

## TODO

You can find the project's development status and future plans in the [`TODO.md`](TODO.md) file.

## License

This project is licensed under the [MIT License](LICENSE).