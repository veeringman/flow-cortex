#!/usr/bin/env bash
# Helper to build and launch L1 node and explorer, optionally binding to a public interface.
# Usage: scripts/run_servers.sh [l1_bind] [explorer_bind]
# e.g. scripts/run_servers.sh 0.0.0.0:3000 0.0.0.0:4000

set -euo pipefail

l1_addr=${1:-0.0.0.0:3000}
expl_addr=${2:-0.0.0.0:4000}

# build both crates
echo "Building flowcortex-l1..."
cargo build --manifest-path flowcortex-l1/Cargo.toml

echo "Building explorer..."
cargo build --manifest-path explorer/Cargo.toml

# launch processes in background

export BIND_ADDR="$l1_addr"
(cd flowcortex-l1 && cargo run --quiet) &
L1_PID=$!

echo "L1 node pid=$L1_PID bound to $l1_addr"

export BIND_ADDR="$expl_addr"
(cd explorer && cargo run --quiet) &
EXP_PID=$!

echo "Explorer pid=$EXP_PID bound to $expl_addr"

echo "Servers are running; press Ctrl-C to stop"

# trap to kill children
trap 'kill $L1_PID $EXP_PID 2>/dev/null || true' EXIT

# wait until killed
wait
