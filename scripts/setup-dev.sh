#!/bin/bash
# Setup script for development environment

set -e

echo "🚀 Setting up Rustified development environment..."

# Check if git is initialized
if [ ! -d ".git" ]; then
    echo "❌ This is not a git repository. Please run from the project root."
    exit 1
fi

# Install git hooks
echo "📋 Installing git hooks..."
if [ -f ".github/hooks/pre-commit" ]; then
    cp .github/hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Pre-commit hook installed"
else
    echo "⚠️  Pre-commit hook not found in .github/hooks/"
fi

# Check Rust installation
echo "🦀 Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install it from https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version)
echo "✅ Found: $RUST_VERSION"

# Check for required components
echo "🔧 Checking Rust components..."
if ! rustup component list --installed | grep -q "clippy"; then
    echo "📦 Installing clippy..."
    rustup component add clippy
fi

if ! rustup component list --installed | grep -q "rustfmt"; then
    echo "📦 Installing rustfmt..."
    rustup component add rustfmt
fi

echo "✅ All required components installed"

# Install dependencies
echo "📦 Installing project dependencies..."
cargo build

# Run initial checks
echo "🔍 Running initial code quality checks..."

echo "  • Checking formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting issues found. Run: cargo fmt"
    exit 1
fi

echo "  • Running clippy..."
if ! cargo clippy --all-targets -- -D warnings -D dead_code; then
    echo "❌ Clippy issues found. Please fix them before proceeding."
    exit 1
fi

echo "  • Running tests..."
if ! cargo test; then
    echo "❌ Tests failed. Please fix them before proceeding."
    exit 1
fi

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "Important reminders:"
echo "  • This project has a STRICT no-dead-code policy"
echo "  • All unused code will be rejected in CI/CD"
echo "  • Pre-commit hooks will catch dead code locally"
echo "  • See CONTRIBUTING.md for detailed guidelines"
echo ""
echo "To manually check for dead code:"
echo "  cargo check --all-targets"
echo "  cargo clippy --all-targets -- -D dead_code"
echo ""
echo "Happy coding! 🦀"
