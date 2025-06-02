#!/bin/bash
# scripts/prepare-pr.sh
# Ensures your branch is ready for Pull Request to Rustified
# Usage: ./scripts/prepare-pr.sh

set -e

trap 'echo "❌ Script failed. Please check the error messages above and fix the issues before making a PR."; exit 1' ERR

# Check for uncommitted changes
git diff --quiet || { echo "❌ You have uncommitted changes. Please commit or stash them before making a PR."; exit 1; }

echo "✅ No uncommitted changes."

# Run formatting check
echo "🔍 Checking code formatting..."
cargo fmt --all -- --check

echo "✅ Formatting OK."

# Run clippy for linting and dead code
echo "🔍 Running clippy (with dead code check)..."
cargo clippy --all-targets -- -D warnings -D dead_code

echo "✅ Clippy passed."

# Run tests
echo "🔍 Running tests..."
cargo test

echo "✅ All tests passed."

echo "🎉 Your branch is ready for a Pull Request!"
