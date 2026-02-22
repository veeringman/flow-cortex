#!/bin/bash
# Build all components

set -e

echo "🔨 Building FlowCortex Components"
echo "=================================="

# Build FlowCortex L0
echo ""
echo "📦 Building L0 (Proof System)..."
cd flowcortex-l0
cargo build --release
cd ..
echo "✅ L0 built"

# Build FlowCortex L1
echo ""
echo "📦 Building L1 (Node)..."
cd flowcortex-l1
cargo build --release
cd ..
echo "✅ L1 built"

# Build Explorer
echo ""
echo "📦 Building Explorer (UI)..."
cd explorer
cargo build --release
cd ..
echo "✅ Explorer built"

# Build TypeScript examples (optional)
if command -v npm &> /dev/null; then
    echo ""
    echo "📦 Building TypeScript Examples..."
    cd examples/l1-integration-clients/typescript
    npm install --legacy-peer-deps
    npm run build
    cd ../../..
    echo "✅ TypeScript examples built"
fi

echo ""
echo "✅ All components built successfully!"
echo ""
echo "Binaries location:"
echo "  - L1 Node:   ./flowcortex-l1/target/release/flowcortex-l1"
echo "  - Explorer:  ./explorer/target/release/explorer"
echo ""
echo "Run: ./scripts/run_servers.sh"
