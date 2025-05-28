# 🦀 Rustified - Minecraft Launcher

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A command-line Minecraft launcher written in Rust. This project demonstrates Rust's capabilities for building game launchers with proper authentication, file management, and cross-platform support.

## 🎯 What Actually Works

**Fully Functional:**
- ✅ **List Minecraft Versions**: Fetches and displays all available releases and snapshots from Mojang
- ✅ **Java Detection**: Automatically finds Java installations on your system  
- ✅ **Instance Management**: Create and manage multiple game instances (tested with 2+ instances)
- ✅ **Version Information**: Shows latest release/snapshot versions with timestamps

**Partially Implemented:**
- 🔶 **Download Game Files**: Core downloading logic exists but needs authentication setup
- 🔶 **Microsoft Authentication**: OAuth2 flow implemented but requires Azure app registration
- 🔶 **Launch Minecraft**: Launching logic exists but depends on auth + file downloads

**Placeholder Features** (show "not yet implemented" messages):
- ❌ Java management commands
- ❌ Advanced instance operations  
- ❌ Mod support
- ❌ GUI interface (command-line only, no GUI planned)

## 📦 Installation

### Option 1: Download Pre-built Binaries

**Note**: No releases have been published yet, but the release infrastructure is ready. Once the first version is tagged, binaries will be automatically built for:
- macOS (Intel & Apple Silicon)
- Windows x64  
- Linux x64

Check the [Releases page](https://github.com/OmarAfet/rustified/releases) for available downloads.

### Option 2: Build from Source

**Requirements:**
- Rust 1.70 or later
- Microsoft Azure App Registration (for authentication)

```bash
# Clone the repository
git clone https://github.com/OmarAfet/rustified.git
cd rustified

# Build release binary
cargo build --release

# The binary will be available at target/release/rustified
```

### Authentication Setup

You'll need a Microsoft Azure app registration to use authentication:

1. Go to [Azure Portal](https://portal.azure.com)
2. Create a new App Registration
3. Set redirect URI to `http://localhost:8080`
4. Copy the Client ID
5. Create a `.env` file in the project root:
   ```
   MS_CLIENT_ID=your-client-id-here
   ```

## 🚀 Quick Start

### 1. Build and Setup
```bash
git clone https://github.com/OmarAfet/rustified.git
cd rustified
cargo build --release
```

Set up your `.env` file with your Microsoft Azure Client ID (see Installation section above).

### 2. List Available Versions
```bash
./target/release/rustified list --limit 10
```

### 3. Download a Version (Optional)
```bash
# This downloads the game files without launching
./target/release/rustified prepare 1.21.5
```

### 4. Launch Minecraft
```bash
./target/release/rustified launch 1.21.5
```

On first launch, it will:
1. Open your browser for Microsoft authentication
2. Create a default instance if none exists
3. Download any missing game files
4. Launch Minecraft

## 📚 Commands

### Basic Commands

```bash
# List available Minecraft versions
./rustified list

# Launch a specific version
./rustified launch 1.21.5

# Download files without launching
./rustified prepare 1.21.5

# Check authentication status
./rustified auth status

# List instances
./rustified instance list

# Get help
./rustified --help
```

### Instance Management

```bash
# Create a new instance
./rustified instance create my_instance 1.21.5 --description "My custom setup"

# Launch with specific instance
./rustified launch 1.21.5 --instance my_instance

# List all instances
./rustified instance list

# Get instance details
./rustified instance info my_instance

# Delete an instance
./rustified instance delete old_instance
```

### Authentication

```bash
# Check if you're logged in
./rustified auth status

# Clear saved authentication (forces re-login)
./rustified auth clear

# Force refresh authentication
./rustified auth refresh
```

## ⚙️ How It Works

Rustified follows the standard Minecraft launcher protocol:

1. **Authentication**: Uses Microsoft OAuth2 → Xbox Live → Minecraft Services authentication chain
2. **Version Discovery**: Fetches available versions from `https://launchermeta.mojang.com/mc/game/version_manifest.json`
3. **File Management**: Downloads game JAR, libraries, and assets to standard `.minecraft` directory
4. **Instance Isolation**: Each instance has its own save directory while sharing downloaded files
5. **Java Integration**: Finds system Java installations and uses appropriate version for each Minecraft version

### Directory Structure

```
~/.minecraft/           # macOS/Linux
%APPDATA%\.minecraft\  # Windows
├── versions/          # Game JARs and version info
├── libraries/         # Java libraries  
├── assets/           # Game assets (sounds, textures, etc.)
└── instances/        # Instance-specific data
    ├── default/      # Default instance
    └── my_instance/  # Custom instances
```

## 🔧 Requirements

### System Requirements
- **OS**: macOS 10.15+, Windows 10+, or modern Linux distribution
- **Memory**: 4GB RAM minimum (8GB recommended for Minecraft)
- **Storage**: 2GB free space (more for multiple versions and worlds)
- **Java**: Java 17+ (automatically detected by the launcher)
- **Network**: Internet connection for downloads and authentication

### Development Requirements
- **Rust**: 1.70 or later
- **Microsoft Azure**: App registration for authentication (free)

### Supported Platforms
- **macOS**: ARM64 (Apple Silicon), x86_64 (Intel)
- **Windows**: x86_64
- **Linux**: x86_64

## 🛠️ Development

### Building

```bash
# Clone and build
git clone https://github.com/OmarAfet/rustified.git
cd rustified
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- launch 1.21.5
```

### Project Structure

```
src/
├── main.rs              # CLI entry point and command handling
├── auth/                # Microsoft/Xbox/Minecraft authentication
│   ├── microsoft.rs     # OAuth2 with Microsoft
│   ├── xbox.rs          # Xbox Live authentication  
│   ├── minecraft.rs     # Minecraft Services API
│   └── storage.rs       # Token caching
└── launcher/            # Core launcher functionality
    ├── game.rs          # Game launching logic
    ├── files.rs         # Download and file management
    ├── instance.rs      # Instance management
    ├── java.rs          # Java detection and management
    └── version.rs       # Version manifest parsing
```

### Code Architecture

The launcher is built with these principles:
- **Modular Design**: Clear separation between authentication, file management, and game launching
- **Error Handling**: Uses `anyhow` for comprehensive error context
- **Async/Await**: All network operations are asynchronous
- **Type Safety**: Leverages Rust's type system to prevent common launcher bugs
- **Cross-Platform**: Conditional compilation for platform-specific behavior

## 🤝 Contributing

Contributions are welcome! This project is in active development and there are many opportunities to help.

### Areas for Contribution

- **Mod Support**: Implementing Fabric/Forge mod loading
- **Better Error Messages**: Improving user-facing error handling
- **Testing**: Adding more comprehensive tests
- **Documentation**: Code documentation and user guides  
- **Performance**: Optimizing download speeds and memory usage
- **Features**: Implementing the placeholder commands that show "not yet implemented"

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes and add tests
4. Ensure code quality: `cargo clippy` and `cargo test`
5. Commit changes: `git commit -m 'Add feature'`
6. Push and open a Pull Request

### Code Quality

This project enforces strict code quality standards:
- **No dead code** - All code must be used
- **No warnings** - Code must compile without warnings
- **Proper error handling** - Use `anyhow::Result` for error propagation
- **Tests** - Add tests for new functionality when possible

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This is an unofficial Minecraft launcher. Minecraft is a trademark of Mojang Studios/Microsoft. This project is not affiliated with or endorsed by Mojang Studios or Microsoft.

## 🙏 Acknowledgments

- **Mojang Studios/Microsoft** - For creating Minecraft and providing the authentication APIs
- **Rust Community** - For the excellent ecosystem of crates used in this project
- **OAuth2 and HTTP libraries** - This project builds on the work of many open-source contributors

## 🔮 Planned Features

Future development may include:
- Mod loader support (Fabric/Forge)
- Resource pack management  
- Better Java version management
- Configuration file support
- More comprehensive error recovery
- Performance optimizations

---

**Current Status**: Early development, basic functionality working, authentication implemented, some features still in progress.
