#!/usr/bin/env bash
set -e

PROJECT_NAME="flowcortex"

echo "🚀 Initializing FlowCortex blockchain project structure..."

# Root
mkdir -p $PROJECT_NAME
cd $PROJECT_NAME

# -----------------------------
# Core Rust crates
# -----------------------------
mkdir -p crates/{core,consensus,execution,runtime,networking,storage,crypto,zk,proofcortex,qct}

# -----------------------------
# Node & client implementations
# -----------------------------
mkdir -p nodes/{validator,full,light,archive,sequencer}
mkdir -p clients/{cli,wallet-sdk,banking-sdk,iot-sdk}

# -----------------------------
# Runtime & VM
# -----------------------------
mkdir -p runtime/{wasm,native,tee}
mkdir -p runtime/wasm/{host,guest,syscalls}

# -----------------------------
# Consensus & Ordering-less logic
# -----------------------------
mkdir -p consensus/{orderingless,finality,fork-choice,slashing}

# -----------------------------
# Proof & verification pipelines
# -----------------------------
mkdir -p proof/{proofcortex,qct,verkle,ipa,kzg}
mkdir -p proof/hardware/{cpu,gpu,asic}

# -----------------------------
# Cryptography & PQC
# -----------------------------
mkdir -p crypto/{hash,signatures,commitments,pqc,keys}
mkdir -p crypto/pqc/{dilithium,falcon,sphincs}

# -----------------------------
# Zero Knowledge
# -----------------------------
mkdir -p zk/{circuits,provers,verifiers,plonk,stark,custom}

# -----------------------------
# Payments-first domain
# -----------------------------
mkdir -p domains/{payments,banking,settlement,compliance}
mkdir -p domains/payments/{accounts,fees,finality,refunds}

# -----------------------------
# Networking
# -----------------------------
mkdir -p networking/{p2p,gossip,discovery,rpc,grpc}

# -----------------------------
# Storage
# -----------------------------
mkdir -p storage/{state,ledger,pruning,snapshots,verkle}

# -----------------------------
# APIs
# -----------------------------
mkdir -p api/{rpc,rest,grpc,events}

# -----------------------------
# Smart contracts / apps
# -----------------------------
mkdir -p contracts/{payment,identity,settlement,zkapps}

# -----------------------------
# DevOps & Infrastructure
# -----------------------------
mkdir -p infra/{docker,k8s,terraform,ansible}
mkdir -p infra/docker/{node,validator,client}

# -----------------------------
# Benchmarks & Testing
# -----------------------------
mkdir -p benchmarks/{execution,proofs,network,storage}
mkdir -p tests/{unit,integration,e2e,fuzz,simulation}

# -----------------------------
# Research & specs
# -----------------------------
mkdir -p research/{qct,orderingless,finality,proofs,pqc}
mkdir -p specs/{protocol,wire-format,consensus,execution}

# -----------------------------
# Docs
# -----------------------------
mkdir -p docs/{architecture,protocol,consensus,proofs,crypto,devops,use-cases}
mkdir -p docs/architecture/{diagrams,decisions}

# -----------------------------
# Tools & scripts
# -----------------------------
mkdir -p tools/{keygen,genesis,benchmarking,debug}
mkdir -p scripts/{ci,release,localnet}

# -----------------------------
# Examples & demos
# -----------------------------
mkdir -p examples/{payments,banking,iot,zk}

# -----------------------------
# Config & misc
# -----------------------------
mkdir -p config/{dev,test,prod}
mkdir -p .github/{workflows,ISSUE_TEMPLATE}

# -----------------------------
# Git + meta files
# -----------------------------
touch README.md
touch LICENSE
touch .gitignore
touch rust-toolchain.toml

# -----------------------------
# Seed README
# -----------------------------
cat <<EOF > README.md
# FlowCortex

FlowCortex is a **payment-centric, ordering-less blockchain** built in Rust.

### Core Ideas
- Ordering-less execution
- ProofCortex + QCT finality
- WASM-based runtime
- Zero-knowledge proofs
- Post-quantum cryptography
- CPU / GPU / ASIC optimized verification

### Status
🚧 Research & Architecture phase

See \`docs/architecture/overview.md\` to begin.
EOF

# -----------------------------
# Gitignore
# -----------------------------
cat <<EOF > .gitignore
/target
**/*.rs.bk
.DS_Store
.env
.idea
.vscode
EOF

echo "✅ FlowCortex project structure created successfully."
echo "📁 Next steps:"
echo "   - git init"
echo "   - docs/architecture/overview.md"
echo "   - crates/core"

