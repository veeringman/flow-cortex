#!/usr/bin/env bash
# End-to-end helper for building, testing, and exercising the L1 node and explorer.
# Usage: scripts/e2e/run_l1_explorer_e2e.sh

set -euo pipefail

# helper for jq output if available
jqcmd="jq ."
if ! command -v jq >/dev/null 2>&1; then
    echo "warning: jq not found; JSON output will be printed raw"
    jqcmd="cat"
fi

echo "
==== Building and testing crates ===="

cargo build --manifest-path flowcortex-l1/Cargo.toml
cargo test --manifest-path flowcortex-l1/Cargo.toml

cargo build --manifest-path explorer/Cargo.toml
# explorer has no tests currently, but building ensures dependencies are correct


echo "
==== Launching L1 node ===="
# start L1 node in background
cargo run --manifest-path flowcortex-l1/Cargo.toml &
L1_PID=$!
trap 'kill "${L1_PID}" 2>/dev/null || true' EXIT
# give the server a moment to start
sleep 2

# basic RPC smoke tests
echo "- creating accounts and minting"
curl -s -X POST http://127.0.0.1:3000/account \
    -H 'Content-Type: application/json' \
    -d '{"account":"test1"}' | $jqcmd

curl -s -X POST http://127.0.0.1:3000/mint \
    -H 'Content-Type: application/json' \
    -d '{"caller":"admin","to":"test1","token":"proof","amount":100}' | $jqcmd

curl -s http://127.0.0.1:3000/balance/test1/proof | $jqcmd

curl -s -X POST http://127.0.0.1:3000/account \
    -H 'Content-Type: application/json' \
    -d '{"account":"test2"}' | $jqcmd

curl -s -X POST http://127.0.0.1:3000/transfer \
    -H 'Content-Type: application/json' \
    -d '{"from":"test1","to":"test2","token":"proof","amount":25}' | $jqcmd

curl -s http://127.0.0.1:3000/balance/test2/proof | $jqcmd


echo "
==== Launching Explorer ===="

cargo run --manifest-path explorer/Cargo.toml &
EXP_PID=$!
trap 'kill "${L1_PID}" "${EXP_PID}" 2>/dev/null || true' EXIT
sleep 1

echo "- querying explorer UI"
if curl -s -f http://127.0.0.1:4000/ | grep -q "<html"; then
    echo "explorer UI returned HTML"
else
    echo "explorer UI failed or returned unexpected content"
    exit 1
fi


# cleanup (trap will take care of killing processes)
echo "
==== E2E smoke tests completed successfully ===="
