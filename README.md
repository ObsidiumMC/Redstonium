# Rustified

[![CI](https://github.com/OmarAfet/rustified/actions/workflows/ci.yml/badge.svg)](https://github.com/OmarAfet/rustified/actions)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A fast and modern Minecraft CLI launcher built with Rust. Rustified provides a robust, cross-platform alternative to graphical launchers, supporting advanced instance management, Microsoft authentication, and seamless integration into scripts and automation workflows.

---

## ✨ Features
- **Instance-based launching**: Manage multiple Minecraft instances with isolated configs
- **Microsoft/Xbox Live authentication**: Secure OAuth 2.0 device flow
- **Automatic download**: Handles Minecraft versions, libraries, and assets
- **Java runtime management**: Detects and recommends correct Java version per instance
- **Cross-platform**: Works on Windows, macOS, and Linux
- **CLI-first**: Scriptable, fast, and minimal dependencies
- **Dead code policy**: CI and pre-commit hooks enforce zero dead code

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Java (see below for version requirements)
- Git

### Build from Source
```bash
git clone https://github.com/OmarAfet/rustified.git
cd rustified
cargo build --release
```
The binary will be at `target/release/rustified`.

### Or Download Prebuilt Binaries
See [Releases](https://github.com/OmarAfet/rustified/releases) for Windows, macOS, and Linux builds.

## 🕹️ Usage

```bash
./rustified --help
```

**Main commands:**
- `list` — List available Minecraft versions
- `launch <instance>` — Launch a Minecraft instance
- `prepare <version>` — Download a version without launching
- `auth <status|clear|refresh>` — Manage authentication
- `instance <list|create|delete|info|memory>` — Manage instances
- `java <list|recommend>` — Java runtime management

**Examples:**
```bash
# List latest 10 versions
./rustified list

# Launch an instance
./rustified launch my-instance

# Authenticate with Microsoft
./rustified auth status

# Create a new instance
./rustified instance create my-instance 1.21.5 --description "My modded setup"

# Show recommended Java version for Minecraft 1.21.5
./rustified java recommend 1.21.5
```

## 🛠️ Development

- **Setup:**
  ```bash
  scripts/setup-dev.sh
  # or
  just setup
  ```
- **Run all checks:** `just check-all` or manually:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings -D dead_code
  cargo test
  ```
- **Pre-commit hooks:** Installed by setup script for code formatting and dead code checks
- **Test:** `cargo test` (see `tests/integration_tests.rs`)
- **Coverage:** `cargo tarpaulin --out Html` (requires [cargo-tarpaulin](https://github.com/xd009642/tarpaulin))

## 🧩 Architecture
- Modular: `auth/`, `launcher/`, and submodules for each concern
- Async: Uses `tokio` and `reqwest` for fast downloads and authentication
- Strict linting: CI and local checks enforce code quality
- See [GUIDE.md](GUIDE.md) for a deep-dive into technical design

## 📝 Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines, code style, and workflow. PRs welcome!

## 📜 License
MIT © 2025 OmarAfet

## 📚 Resources
- [Technical Guide](GUIDE.md)
- [Changelog](CHANGELOG.md)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Minecraft Wiki](https://minecraft.wiki/w/Minecraft_Launcher)

---

*Rustified is not affiliated with Mojang, Microsoft, or Minecraft. Minecraft is a trademark of Mojang AB and Microsoft.*
