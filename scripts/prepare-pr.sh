#!/bin/bash
# scripts/prepare-pr.sh
# Ensures your branch is ready for Pull Request to Rustified
# Usage: ./scripts/prepare-pr.sh

set -e

trap 'echo "❌ Script failed. Please check the error messages above and fix the issues before making a PR."; exit 1' ERR

# Run clippy for linting and dead code
echo "🔍 Running clippy"
cargo clippy

echo "✅ Clippy passed."

# Run tests
echo "🔍 Running tests..."
cargo test

echo "✅ All tests passed."

# Run formatting check
echo "🔍 Checking code formatting..."
cargo fmt --all -- --check

echo "✅ Formatting OK."

echo "🎉 Your branch is ready for a Pull Request!"
