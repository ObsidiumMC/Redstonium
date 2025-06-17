# Rustified

A command-line Minecraft launcher written in Rust.

Rustified is a lightweight, cross-platform tool for managing and launching Minecraft from your terminal. It handles authentication, version management, and game instances, providing a simple interface for automation and server-side setups.

## Features

- **Authentication**: Securely logs into your Microsoft account to authenticate with Minecraft services. Caches credentials for future sessions.
- **Version Management**: Lists available Minecraft versions (releases, snapshots, etc.) and downloads all necessary game files, libraries, and assets.
- **Instance Management**: Create, list, and delete separate game instances, each with its own version and settings.
- **Configuration**: Set instance-specific memory allocation.
- **Java Detection**: Automatically finds suitable Java installations on your system.
- **Cross-Platform**: Works on Windows, macOS, and Linux.

## Installation

You need the Rust toolchain installed to build Rustified.

1.  Clone the repository:
    ```sh
    git clone https://github.com/OmarAfet/Rustified.git
    cd Rustified
    ```

2.  Build the project for release:
    ```sh
    cargo build --release
    ```

3.  The executable will be located at `target/release/rustified`. You can move this file to a directory in your system's PATH to use it globally.

## Usage

Rustified is controlled via subcommands. You can see all available commands and options by running:

```sh
rustified --help
```

### Basic Workflow

1.  **Create an instance:**
    First, create an instance for a specific Minecraft version. For example, to create an instance named `vanilla-1-21` for Minecraft `1.21`:

    ```sh
    rustified instance create vanilla-1-21 1.21
    ```

2.  **Launch the instance:**
    The first time you launch an instance, Rustified will guide you through the Microsoft authentication process in your web browser. After that, your login will be cached.

    ```sh
    rustified launch vanilla-1-21
    ```

### Other Commands

**List available Minecraft versions:**
```sh
# Show the 10 newest releases and snapshots
rustified list

# Show only the 5 latest release versions
rustified list --releases-only --limit 5

# Filter versions by a pattern
rustified list --filter "1.18"
```

**Manage instances:**
```sh
# List all created instances
rustified instance list

# Delete an instance
rustified instance delete vanilla-1-21

# Set the memory for an instance to 4096 MB
rustified instance memory vanilla-1-21 4096
```

**Manage authentication:**
```sh
# Check your current authentication status
rustified auth status

# Clear the cached login credentials
rustified auth clear
```

**Manage Java:**
```sh
# List detected Java installations
rustified java list

# See the recommended Java version for a Minecraft version
rustified java recommend 1.21
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.