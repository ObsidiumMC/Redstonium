# Rustified

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub release](https://img.shields.io/github/v/release/OmarAfet/rustified)](https://github.com/OmarAfet/rustified/releases)
[![CI](https://github.com/OmarAfet/rustified/workflows/CI/badge.svg)](https://github.com/OmarAfet/rustified/actions)

A high-performance, cross-platform Minecraft launcher written in Rust that provides a modern CLI experience for managing and launching Minecraft instances.

## Features

### **Secure Authentication**
- Microsoft OAuth 2.0 integration with Xbox Live authentication
- Secure token storage and automatic refresh
- Full support for Microsoft/Mojang accounts

### **Instance Management**
- Create and manage multiple Minecraft instances
- Per-instance settings (memory allocation, Java args, game directory)
- Quick instance switching and launch tracking

### **Smart Java Management**
- Automatic Java detection across platforms
- Version compatibility matching for Minecraft versions
- Support for multiple Java installations

### **Efficient File Management**
- Parallel downloading with SHA1 verification
- Incremental updates and smart caching
- Support for all Minecraft versions (Alpha, Beta, Release, Snapshot)

### **Performance Optimized**
- Concurrent asset and library downloads
- Memory-efficient file processing
- Fast startup times with cached authentication

### **Cross-Platform Support**
- Windows, macOS, and Linux compatibility
- Native file system integration
- Platform-specific optimizations

## Quick Start

### Installation

#### From Releases (Recommended)
Download the latest binary from [GitHub Releases](https://github.com/OmarAfet/rustified/releases).

#### From Source
```bash
# Clone the repository
git clone https://github.com/OmarAfet/rustified.git
cd rustified

# Build and install
cargo install --path .
```

### First Launch

1. **Create your first instance:**
    ```bash
    rustified instance create my-instance 1.20.4
    ```

2. **Launch Minecraft:**
    ```bash
    rustified launch my-instance
    ```

The launcher will automatically handle authentication and download necessary files.

## Usage

### Version Management

```bash
# List available Minecraft versions
rustified list

# Show only release versions
rustified list --releases-only

# Filter versions and limit results
rustified list --filter "1.20" --limit 5

# Show installation status
rustified list --show-installed
```

### Instance Management

```bash
# List all instances
rustified instance list

# Create a new instance
rustified instance create <name> <version> [description]

# Delete an instance
rustified instance delete <name>

# Show instance details
rustified instance info <name>

# Set memory allocation (in MB)
rustified instance memory <amount>
```

### Game Operations

```bash
# Launch an instance
rustified launch <instance-name>

# Skip file verification for faster launch
rustified launch <instance-name> --skip-verification

# Prepare/download a version without launching
rustified prepare <version>
```

### Authentication

```bash
# Check authentication status
rustified auth status

# Clear cached authentication
rustified auth clear

# Force re-authentication
rustified auth refresh
```

### Java Management

```bash
# List detected Java installations
rustified java list

# Get recommended Java version for Minecraft version
rustified java recommend <minecraft-version>
```

## Architecture

Rustified is built with a modular architecture:

```
src/
├── auth/           # Microsoft OAuth 2.0 & Minecraft authentication
├── commands/       # CLI command implementations
├── launcher/       # Core launcher functionality
│   ├── files.rs    # Download manager with parallel processing
│   ├── game.rs     # Game launch logic and argument handling
│   ├── instance.rs # Instance configuration management
│   └── java.rs     # Java detection and version matching
├── cli.rs          # Command-line interface definitions
├── error.rs        # Custom error types and handling
└── main.rs         # Application entry point
```

### Key Components

- **FileManager**: Handles parallel downloads, SHA1 verification, and caching
- **GameLauncher**: Constructs JVM arguments and launches Minecraft processes
- **InstanceManager**: Manages instance configurations and settings
- **JavaManager**: Detects and manages Java installations
- **AuthManager**: Handles the complete Microsoft → Xbox Live → Minecraft auth flow

## Configuration

### Instance Settings

Each instance supports customizable settings:

```rust
// Instance configuration example
{
     "name": "my-instance",
     "version": "1.20.4",
     "description": "My custom instance",
     "settings": {
          "memory_mb": 4096,
          "java_args": ["-XX:+UseG1GC"],
          "resolution": {"width": 1920, "height": 1080}
     },
     "last_used": "2024-01-15T10:30:00Z"
}
```

### Authentication Storage

Authentication tokens are securely stored in:
- **Windows**: `%APPDATA%/rustified/auth.json`
- **macOS**: `~/Library/Application Support/rustified/auth.json`
- **Linux**: `~/.local/share/rustified/auth.json`

## Development

### Prerequisites

- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/OmarAfet/rustified.git
cd rustified

# Build in debug mode
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- list
```

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Code Quality

The project maintains high code quality with:
- Comprehensive error handling with custom error types
- Extensive logging and debugging capabilities
- Performance optimizations for file operations
- Memory-safe concurrent operations
- Cross-platform compatibility testing

## Performance

Rustified is designed for performance:

- **Parallel Downloads**: Up to 50 concurrent asset downloads
- **Smart Caching**: Incremental updates with SHA1 verification
- **Memory Efficient**: Optimized for large modpacks and asset collections
- **Fast Startup**: Cached authentication and version data

## Troubleshooting

### Common Issues

**Authentication fails:**
```bash
# Clear cached auth and try again
rustified auth clear
rustified auth refresh
```

**Java not detected:**
```bash
# Check Java installations
rustified java list

# Ensure Java 17+ is installed for modern Minecraft versions
```

**Download failures:**
```bash
# Retry with debug logging
RUST_LOG=debug rustified prepare <version>
```

### Debug Mode

Enable detailed logging:
```bash
RUST_LOG=debug rustified <command>
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] **GUI Interface**: Native desktop application
- [ ] **Mod Support**: Forge, Fabric, and Quilt integration
- [ ] **Modpack Management**: Import/export and automatic installation
- [ ] **Profile Sharing**: Community instance sharing
- [ ] **Advanced Java Management**: Automatic Java downloading

---

<div align="center">
    <strong>Made with ❤️ and 🦀 Rust</strong>
    <br>
    <a href="https://github.com/OmarAfet/rustified">⭐ Star this project on GitHub</a>
</div>