#!/bin/bash
# Start Explorer UI only

set -e

# Get Explorer bind address (default: 0.0.0.0:4000)
EXPLORER_ADDR="${1:-0.0.0.0:4000}"

echo "🌐 Starting FlowCortex Explorer UI"
echo "=================================="
echo "UI: http://$EXPLORER_ADDR"
echo ""

cd explorer

export BIND_ADDR="$EXPLORER_ADDR"

cargo run --release

