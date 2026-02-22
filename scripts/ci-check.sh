#!/bin/bash
# CI checks - format, lint, test

set -e

echo "🔍 Running CI Checks"
echo "===================="

# Format check
echo ""
echo "📋 Checking code formatting..."
cargo fmt --all -- --check
echo "✅ Format OK"

# Clippy lint
echo ""
echo "🔎 Running clippy lints..."
cargo clippy --all --all-targets -- -D warnings
echo "✅ Clippy OK"

# Tests
echo ""
echo "🧪 Running tests..."
cargo test --all --lib
echo "✅ Tests passed"

# Documentation
echo ""
echo "📚 Checking documentation..."
cargo doc --all --no-deps
echo "✅ Documentation OK"

echo ""
echo "✅ All CI checks passed!"

