# Contributing to Rustified

Thank you for your interest in contributing to Rustified! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ installed ([rustup.rs](https://rustup.rs/))
- Git
- A GitHub account

### Setting Up the Development Environment

1. **Fork the repository**
   ```bash
   # Visit https://github.com/OmarAfet/rustified and click "Fork"
   ```

2. **Clone your fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/rustified.git
   cd rustified
   ```

3. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/OmarAfet/rustified.git
   ```

4. **Install dependencies and build**
   ```bash
   cargo build
   ```

5. **Run the setup script (recommended)**
   ```bash
   scripts/setup-dev.sh
   ```
   This will:
   - Install git hooks for dead code checking
   - Install required Rust components
   - Run initial quality checks
   - Verify your environment is ready

6. **Run tests**
   ```bash
   cargo test
   ```

## 🏗️ Development Workflow

### 1. Create a Feature Branch

```bash
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write clean, idiomatic Rust code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run clippy for linting (includes dead code check)
cargo clippy --all-targets -- -D warnings -D dead_code

# Check for dead code specifically
cargo check --all-targets

# Code formatting happens automatically in CI when you push
# But you can run it locally if needed: cargo fmt

# Test on different Minecraft versions
cargo run -- launch 1.21.5
cargo run -- launch 1.20.4
```

**⚠️ Important:** Make sure there are no dead code warnings before submitting your PR. Our CI will automatically reject PRs with unused code.

### 4. Prepare for Pull Request

Before opening a Pull Request, run the following script to ensure your branch meets all requirements:

```bash
scripts/prepare-pr.sh
```

This script will:
- Check for uncommitted changes
- Check code formatting
- Run clippy with dead code checks
- Run all tests

Your branch must pass all these checks before submitting a PR.

### Quick Commands (using justfile)

If you have [`just`](https://github.com/casey/just) installed, you can use these convenient commands:

```bash
# Run all quality checks (same as CI)
just check-all

# Check for dead code specifically
just dead-code-check

# Run pre-commit checks
just pre-commit

# Setup development environment
just setup
```

### 5. Commit Your Changes

```bash
git add .
git commit -m "feat: add amazing new feature"
```

**Commit Message Format:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

### 6. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then visit GitHub and create a Pull Request.

## 📝 Code Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **Code formatting is automatic** - your code will be formatted by CI when you push
- Use `clippy` for linting: `cargo clippy`
- Prefer explicit error handling over `unwrap()`/`expect()`
- Use descriptive variable and function names

### ⚠️ Dead Code Policy

**IMPORTANT: Dead code is strictly prohibited in this project.**

- **No unused functions, variables, imports, or code blocks** are allowed
- All code must serve a purpose and be actively used
- Remove any experimental or commented-out code before submitting
- Use `#[cfg(test)]` for test-only code and `#[cfg(feature = "...")]` for feature-gated code
- If you need to keep code for future use, create a separate branch or issue instead

📖 **[Read the complete Dead Code Policy](docs/DEAD_CODE_POLICY.md)** for detailed guidelines and troubleshooting.

**Before submitting your PR:**
```bash
# Quick way using justfile (if you have 'just' installed)
just check-all

# Or manually:
# Check for dead code warnings
cargo check --all-targets
cargo clippy --all-targets -- -D warnings -D dead_code

# Our CI will automatically reject PRs with dead code
```

**Why this policy exists:**
- Maintains code quality and readability
- Reduces maintenance burden
- Prevents confusing future contributors
- Keeps the codebase lean and focused

### Code Organization

```rust
// Good: Clear module organization
mod auth {
    pub mod microsoft;
    pub mod minecraft;
}

// Good: Proper error handling
fn download_file(url: &str) -> Result<Vec<u8>, DownloadError> {
    // Implementation
}

// Good: Descriptive naming
fn validate_minecraft_version(version: &str) -> bool {
    // Implementation
}
```

### Documentation

- Add rustdoc comments for public APIs
- Include examples in documentation
- Update README.md for user-facing changes

```rust
/// Downloads and validates a Minecraft version manifest.
/// 
/// # Arguments
/// 
/// * `version` - The Minecraft version to download (e.g., "1.21.5")
/// 
/// # Returns
/// 
/// Returns `Ok(VersionInfo)` on success, or `Err(VersionError)` if the
/// version is invalid or download fails.
/// 
/// # Examples
/// 
/// ```
/// let version_info = download_version_manifest("1.21.5").await?;
/// println!("Downloaded version: {}", version_info.id);
/// ```
pub async fn download_version_manifest(version: &str) -> Result<VersionInfo, VersionError> {
    // Implementation
}
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_version_parsing

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = parse_version("1.21.5").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 21);
        assert_eq!(version.patch, 5);
    }

    #[tokio::test]
    async fn test_download_version_manifest() {
        let result = download_version_manifest("1.21.5").await;
        assert!(result.is_ok());
    }
}
```

## 🐛 Reporting Issues

### Before Submitting an Issue

1. **Search existing issues** to avoid duplicates
2. **Test with the latest version** to ensure the issue still exists
3. **Gather information**:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Rustified version
   - Java version
   - Steps to reproduce

### Issue Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. See error

**Expected behavior**
A description of what you expected to happen.

**Environment:**
- OS: [e.g. macOS 13.0, Windows 11, Ubuntu 22.04]
- Rust version: [e.g. 1.75.0]
- Rustified version: [e.g. 0.1.0]
- Java version: [e.g. OpenJDK 21]

**Additional context**
Add any other context about the problem here.
```

## 🎯 Areas for Contribution

### High Priority
- **Cross-platform testing** - Test on Windows and Linux
- **Error handling improvements** - Better error messages and recovery
- **Performance optimizations** - Faster downloads and startup
- **Documentation** - API docs, tutorials, examples

### Medium Priority
- **New features** - Mod support, GUI interface
- **Code quality** - Refactoring, better abstractions
- **Testing** - More comprehensive test coverage
- **CI/CD improvements** - Better automation

### Good First Issues
Look for issues labeled `good first issue` or `help wanted` in the GitHub repository.

## 📚 Resources

### Rust Learning
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### Minecraft Launcher Development
- [Minecraft Wiki - Launcher](https://minecraft.wiki/w/Minecraft_Launcher)
- [Minecraft Version Manifest](https://launchermeta.mojang.com/mc/game/version_manifest.json)
- [Microsoft Authentication](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-oauth2-auth-code-flow)

### Project-Specific
- [tokio Documentation](https://docs.rs/tokio/) - Async runtime
- [reqwest Documentation](https://docs.rs/reqwest/) - HTTP client
- [serde Documentation](https://docs.rs/serde/) - Serialization

## 🔒 Security

If you discover a security vulnerability, please **DO NOT** open a public issue. Instead:

1. Email the maintainers privately
2. Include a detailed description of the vulnerability
3. Provide steps to reproduce if possible
4. Allow time for the issue to be addressed before public disclosure

## 📜 Code of Conduct

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, education, socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

**Positive behavior includes:**
- Being respectful and inclusive
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards others

**Unacceptable behavior includes:**
- Harassment, trolling, or discriminatory language
- Personal attacks or insults
- Publishing private information without permission
- Any other conduct that would be inappropriate in a professional setting

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the project maintainers. All complaints will be reviewed and investigated promptly and fairly.

## 🎉 Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes for significant contributions
- Special mentions for outstanding contributions

Thank you for contributing to Rustified! 🦀❤️
