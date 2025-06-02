# Rustified

[![CI](https://github.com/OmarAfet/rustified/workflows/CI/badge.svg)](https://github.com/OmarAfet/rustified/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Minecraft](https://img.shields.io/badge/Minecraft-Java%20Edition-green.svg)](https://www.minecraft.net/)

A powerful, cross-platform command-line Minecraft launcher written in Rust. Rustified provides a fast, reliable, and automation-friendly alternative to the official Minecraft launcher, with support for multiple instances, mod management, and advanced configuration options.

## Features

### Core Functionality
- **Cross-platform support** (Windows, macOS, Linux)
- **Microsoft Account authentication** with OAuth 2.0
- **Multiple Minecraft versions** (releases, snapshots, and legacy versions)
- **Instance management** for isolated game environments
- **Automatic Java detection** and version matching
- **Parallel file downloads** with integrity verification
- **Mod loader support** (Vanilla, Forge, Fabric, Quilt)

### Game Management
- **Version preparation** without launching
- **Asset and library management** with content-addressable storage
- **Memory allocation** per instance
- **Custom Java arguments** and game parameters
- **Server quick-connect** configuration
- **Saves and resource pack isolation**

### Developer Features
- **CLI-first design** perfect for automation and scripting
- **Detailed logging** with configurable levels
- **JSON-based configuration** for easy integration
- **Secure credential storage** with platform-specific keychains
- **Fast startup** with cached authentication

## Installation

### Prerequisites
- **Rust** (2024 edition) - [Install Rust](https://rustup.rs/)
- **Java** 8, 11, 16, 17, or 21 (depending on Minecraft version)
- **Microsoft account** with Minecraft Java Edition

### From Source
```bash
git clone https://github.com/OmarAfet/rustified.git
cd rustified
cargo build --release
```

The executable will be available at `target/release/rustified`.

### Environment Setup
No environment setup required for Microsoft authentication. The Microsoft client ID is now included in the codebase.

## Quick Start

### 1. List Available Minecraft Versions
```bash
rustified list --limit 20
rustified list --releases-only
```

### 2. Create Your First Instance
```bash
rustified instance create my-world 1.20.4 --description "My survival world"
```

### 3. Launch Minecraft
```bash
rustified launch my-world
```

### 4. Prepare a Version (Download Only)
```bash
rustified prepare 1.21
```

## Usage

### Instance Management

#### Create a new instance
```bash
rustified instance create <name> <version> [--description "Optional description"]
```

#### List all instances
```bash
rustified instance list
```

#### Get instance details
```bash
rustified instance info <name>
```

#### Delete an instance
```bash
rustified instance delete <name>
```

#### Set memory allocation
```bash
rustified instance memory <name> <memory_mb>
```

### Authentication

#### Check authentication status
```bash
rustified auth status
```

#### Clear cached credentials
```bash
rustified auth clear
```

#### Force re-authentication
```bash
rustified auth refresh
```

### Java Management

#### List detected Java installations
```bash
rustified java list
```

#### Check recommended Java version
```bash
rustified java recommend 1.20.4
```

## Configuration

### Instance Configuration
Each instance is stored as a JSON file in `.minecraft/instances/<name>/instance.json`:

```json
{
  "name": "my-world",
  "version": "1.20.4",
  "description": "My survival world",
  "created": "2025-01-01T12:00:00Z",
  "last_used": "2025-01-02T08:30:00Z",
  "settings": {
    "memory_mb": 4096,
    "java_args": ["-XX:+UseG1GC"],
    "game_args": [],
    "debug": false,
    "server": {
      "address": "mc.hypixel.net",
      "port": 25565
    }
  },
  "mods": {
    "loader": "fabric",
    "loader_version": "0.15.6",
    "mods": []
  }
}
```

### Directory Structure
```
.minecraft/
├── instances/           # Instance-specific directories
│   └── my-world/
│       ├── instance.json
│       ├── saves/
│       ├── resourcepacks/
│       └── screenshots/
├── versions/            # Minecraft version files
├── libraries/           # Java libraries
├── assets/             # Game assets
└── launcher_profiles.json
```

## Architecture

### Core Components

- **`launcher/`** - Main launcher logic and game execution
- **`auth/`** - Microsoft OAuth 2.0 authentication flow
- **`files/`** - Download management and file verification
- **`instance/`** - Instance configuration and management
- **`java/`** - Java detection and version matching

### Authentication Flow

1. **Microsoft OAuth 2.0** - Initial authentication with Microsoft services
2. **Xbox Live** - Token exchange for Xbox Live authentication
3. **XSTS** - Xbox Security Token Service authentication
4. **Minecraft Services** - Final token exchange for Minecraft access
5. **Profile Retrieval** - Fetch player profile and game ownership

### Java Version Requirements

| Minecraft Version | Required Java |
|------------------|---------------|
| ≤ 1.15           | Java 8        |
| 1.16             | Java 8/11     |
| 1.17             | Java 16       |
| 1.18-1.20        | Java 17       |
| 1.21+            | Java 21       |

## Development

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Code Style
```bash
cargo fmt
```

### Prepare for Pull Request
Run the following script to check formatting, linting, and tests before submitting a PR:
```bash
./scripts/prepare-pr.sh
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `./scripts/prepare-pr.sh` to ensure your branch is ready
5. Submit a pull request

## Security

- Credentials are stored securely using platform-specific keychains
- OAuth 2.0 flow with proper token refresh handling
- File integrity verification using SHA1 checksums
- No hardcoded secrets or credentials

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This project is not affiliated with Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Studios.