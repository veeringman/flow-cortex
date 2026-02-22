#!/bin/bash
# Development environment setup script

set -e

echo "🚀 FlowCortex Development Environment Setup"
echo "=========================================="

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "✅ Rust installed"
else
    echo "✅ Rust found: $(cargo --version)"
fi

# Check Node.js (optional)
if ! command -v node &> /dev/null; then
    echo "⚠️  Node.js not found (optional, needed for TypeScript examples)"
else
    echo "✅ Node.js found: $(node --version)"
fi

# Check Python (optional)
if ! command -v python3 &> /dev/null; then
    echo "⚠️  Python not found (optional, needed for Python examples)"
else
    echo "✅ Python found: $(python3 --version)"
fi

# Check Docker (optional)
if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker not found (optional, needed for containerized setup)"
else
    echo "✅ Docker found: $(docker --version)"
fi

# Update Rust
echo ""
echo "📦 Updating Rust toolchain..."
rustup update

# Install useful Rust tools
echo "📦 Installing development tools..."
cargo install cargo-watch || true
cargo install cargo-flamegraph || true
cargo install cargo-edit || true

echo ""
echo "✅ Development environment ready!"
echo ""
echo "Next steps:"
echo "  1. Build: ./scripts/build.sh"
echo "  2. Run:   ./scripts/run_servers.sh"
echo "  3. Test:  cargo test --all"
echo ""
