# Justfile for Rustified development tasks
# Install 'just' command runner: https://github.com/casey/just

# Show available commands
default:
    @just --list

# Run all quality checks (what CI runs)
check-all: format-check clippy test
    @echo "✅ All quality checks passed!"

# Format code
format:
    cargo fmt --all

# Check formatting without changing files
format-check:
    cargo fmt --all -- --check

# Run clippy (automatically applies dead code and all quality checks from Cargo.toml)
clippy:
    cargo clippy --all-targets --all-features

# Run clippy with strict settings
clippy-strict:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Build debug version
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Setup development environment
setup:
    @echo "🚀 Setting up development environment..."
    @chmod +x scripts/setup-dev.sh
    @scripts/setup-dev.sh

# Install git hooks manually
install-hooks:
    @echo "📋 Installing git hooks..."
    @cp .github/hooks/pre-commit .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @echo "✅ Pre-commit hook installed"

# Run security audit
audit:
    cargo audit

# Generate code coverage report
coverage:
    cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html

# Run all checks that should pass before committing
pre-commit: format clippy-strict test
    @echo "🎉 Pre-commit checks passed! Ready to commit."

# Run the launcher with debug logging (requires an instance)
run-debug instance:
    RUST_LOG=debug cargo run -- launch {{instance}}

# Run the launcher (requires an instance)
run instance:
    cargo run -- launch {{instance}}

# List available Minecraft versions
list-versions:
    cargo run -- list

# Check authentication status
auth-status:
    cargo run -- auth status

# Show project statistics
stats:
    @echo "📊 Project Statistics:"
    @echo "Lines of code:"
    @find src -name "*.rs" -exec wc -l {} + | tail -1
    @echo "Number of files:"
    @find src -name "*.rs" | wc -l
    @echo "Dependencies:"
    @cargo tree --depth 1
