#!/usr/bin/env bash

set -e

ROOT_DIR="."
PROJECT_NAME="flowcortex"

echo "🧠 Generating code index.md files for $PROJECT_NAME..."

# Default description fallback
default_description() {
  cat <<EOF
This module is part of the FlowCortex codebase.

It contains implementation logic specific to its responsibility within the
ordering-less, payment-centric blockchain architecture.
EOF
}

# Hand-tuned descriptions for important directories
describe_dir() {
  case "$1" in
    *crates/core*)
      echo "Core primitives and shared types used across the FlowCortex blockchain.
Includes foundational abstractions, error handling, and core data models."
      ;;
    *crates/node*)
      echo "FlowCortex node implementation.
Responsible for networking, state validation, proof verification, and block production."
      ;;
    *crates/proofcortex*)
      echo "ProofCortex implementation layer.
Handles QCT proofs, stateless verification, batching, and parallel proof execution."
      ;;
    *crates/qct*)
      echo "Quantum Cascade Tree (QCT) data structures and commitment logic.
Implements stateless state commitments optimized for ordering-less execution."
      ;;
    *crates/consensus*)
      echo "Consensus and validator coordination logic.
Defines validator roles, message flows, and finality mechanisms."
      ;;
    *crates/execution*)
      echo "Transaction execution and state transition logic.
Supports ordering-less execution and conflict-aware validation."
      ;;
    *crates/runtime*)
      echo "Runtime environments for FlowCortex.
Includes WASM execution, native runtimes, and sandboxed execution layers."
      ;;
    *crates/crypto*)
      echo "Cryptographic primitives used by FlowCortex.
Includes hashing, signatures, commitments, and post-quantum cryptography support."
      ;;
    *crates/network*)
      echo "Peer-to-peer networking layer.
Manages gossip, discovery, message routing, and validator communication."
      ;;
    *)
      default_description
      ;;
  esac
}

# Find all directories that look like code modules
find "$ROOT_DIR" -type d \
  \( -name target -o -name .git \) -prune -false \
  -o -type d | while read -r DIR; do

  # Skip root
  [[ "$DIR" == "." ]] && continue

  # Heuristic: code directories usually contain Cargo.toml or src/
  if [[ -f "$DIR/Cargo.toml" || -d "$DIR/src" ]]; then
    INDEX_FILE="$DIR/index.md"

    if [[ ! -f "$INDEX_FILE" ]]; then
      echo "📝 Creating $INDEX_FILE"

      TITLE=$(basename "$DIR" | tr '-' ' ' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)}1')

      cat <<EOF > "$INDEX_FILE"
# $TITLE

$(describe_dir "$DIR")

---

## Scope

This module may evolve independently as FlowCortex grows.
Public interfaces should remain stable where possible.

## Notes

Implementation details, design decisions, and invariants should be documented here.

EOF
    else
      echo "✅ Skipping existing $INDEX_FILE"
    fi
  fi
done

echo "✅ Code documentation index generation complete."

