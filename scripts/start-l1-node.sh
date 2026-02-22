#!/bin/bash
# Start L1 node only

set -e

# Get L1 bind address (default: 0.0.0.0:3000)
L1_ADDR="${1:-0.0.0.0:3000}"
GRPC_ADDR="${2:-0.0.0.0:50051}"

echo "🚀 Starting FlowCortex L1 Node"
echo "=============================="
echo "REST API: $L1_ADDR"
echo "gRPC:     $GRPC_ADDR"
echo ""

cd flowcortex-l1

export BIND_ADDR="$L1_ADDR"
export GRPC_ADDR="$GRPC_ADDR"

cargo run --release

