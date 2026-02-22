#!/bin/bash
# Run integration client examples

set -e

echo "🧪 Running Integration Client Examples"
echo "======================================"

# Wait for L1 node to be ready
echo "Waiting for L1 node on http://127.0.0.1:3000..."
for i in {1..30}; do
    if curl -s http://127.0.0.1:3000/blocks > /dev/null; then
        echo "✅ L1 node is ready"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ L1 node did not start in time"
        exit 1
    fi
    sleep 1
done

cd examples/l1-integration-clients

# cURL examples
echo ""
echo "📌 Running cURL Examples..."
cd curl
./run-examples.sh | head -30
cd ..
echo "✅ cURL examples completed"

# Python examples
if command -v python3 &> /dev/null; then
    echo ""
    echo "🐍 Running Python Examples..."
    cd python
    python3 example.py | head -30
    cd ..
    echo "✅ Python examples completed"
fi

# TypeScript examples
if command -v node &> /dev/null; then
    echo ""
    echo "📘 Running TypeScript Examples..."
    cd typescript
    if [ ! -d "dist" ]; then
        npm run build > /dev/null
    fi
    npm run example:node | head -30
    cd ..
    echo "✅ TypeScript examples completed"
fi

# Rust gRPC examples
echo ""
echo "🦀 Running Rust gRPC Examples..."
cd rust-grpc
cargo run --release 2>&1 | head -30
echo "✅ Rust gRPC examples completed"

echo ""
echo "✅ All client examples finished!"

