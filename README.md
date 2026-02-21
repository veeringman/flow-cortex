# FlowCortex

**FlowCortex** is a payment-centric, ordering-less blockchain designed for extreme throughput, stateless verification, and parallel execution.

At its core, FlowCortex replaces global transaction ordering with **proof-driven state validation**, enabling payments to scale with actual contention rather than network size.

The system is implemented in **Rust**, optimized for **CPU, GPU, and future ASIC acceleration**, and built to be **post-quantum ready**.

---

## Vision

Traditional blockchains are constrained by:
- Global transaction ordering
- Sequential execution
- Full state replication
- Verification cost tied to total state size

FlowCortex takes a different path:

> **Transactions do not compete for position in a block.  
They compete only for the state they touch.**

This makes FlowCortex naturally suited for:
- High-volume payments
- Real-time settlement
- Stateless validators
- Hardware-accelerated verification

---

## Core Design Principles

- **Ordering-less execution**  
  No global transaction sequence unless state conflicts require it.

- **Stateless verification**  
  Validators do not store full state; transactions carry their own proofs.

- **Payment-first architecture**  
  Optimized for balance transfers, batching, and settlement.

- **Parallelism by default**  
  Proof verification and execution scale across CPU cores, GPUs, and ASICs.

- **Post-quantum oriented**  
  Commitment schemes designed to evolve beyond classical cryptography.

---

## Key Components

### 1. FlowGraph (Ordering-less Transaction DAG)

FlowGraph replaces blocks and mempools with a **causal DAG of transactions**.

- Transactions explicitly declare the state they read and write
- Conflicts are local and explicit
- Non-conflicting transactions execute and finalize independently

There is no global reordering or chain reorganization.

---

### 2. Quantum Cascade Tree (QCT)

QCT is FlowCortex’s stateless state-commitment and verification scheme.

It combines three layers:

- **Hash Layer** — structural integrity
- **Polynomial Commitment Layer (ALC)** — multi-key, batched proofs
- **Frequency / Aggregate Layer** — range and sum verification (e.g., no overdraft)

QCT enables:
- Stateless validation
- Multi-account payment proofs
- Massive parallel verification
- Snapshot-based finality

---

### 3. Enhanced Finality via QCT

Finality is not tied to block height.

Instead:
- Transactions finalize against **QCT snapshot roots**
- Conflicts are resolved locally
- Finality is monotonic and non-reverting

This allows fast settlement without global coordination.

---

### 4. Zero-Knowledge & Privacy (Planned)

FlowCortex is designed to support:
- Zero-knowledge balance proofs
- Private payments with public validity
- Selective disclosure for compliance

These are layered **on top of QCT**, not bolted on later.

---

## Execution Model

1. Transaction is created with:
   - Explicit state access declaration
   - QCT proofs
   - State transition delta

2. Validator:
   - Verifies proofs against snapshot root
   - Checks payment invariants
   - Applies delta locally

3. Finality increases as conflicts are resolved.

No global replay. No block contention.

---

## Hardware Acceleration

FlowCortex treats hardware as a first-class citizen:

- **CPU** — baseline execution and verification
- **GPU** — batch polynomial proof verification
- **Custom ASIC (future)** — payment and proof acceleration

The architecture avoids sequential bottlenecks, making acceleration meaningful.

---

## Language & Runtime

- **Primary Language:** Rust
- **Runtime Targets:**
  - Native (validators, nodes)
  - WASM (light clients, embedded environments)

Rust ensures:
- Memory safety
- Deterministic execution
- High-performance cryptography

---

## Project Status

⚠️ **Early architecture & specification phase**

Current focus:
- Formal documentation
- Cryptographic design validation
- Execution and conflict semantics
- Proof system refinement

Code will follow once core invariants are locked.

---

## Repository Structure (Planned)

> Note:
> Directories under `crates/` represent compile-time Rust units.
> Directories under the project root represent runtime, operational,
> or conceptual system domains. Names may overlap by design.

### Experimental prototypes

- `flowcortex-l0` – a tiny proof-of-concept library for the quantum cascade tree.
- `flowcortex-l1` – minimal L1 blockchain with an in‑memory ledger, RPC server, QCT stubs, read/write sets, conflict detection and a toy block producer (ordering‑less, no real consensus yet). See `flowcortex-l1/readme.md` for details.

- `flowcortex-explorer` – separate crate providing a web-based explorer UI (Rust/axum/Askama) for querying the L1 node. A CLI or gRPC client could be added later depending on user needs.

---

## End-to-End Smoke Tests

A set of simple build/test scripts and integration tests exercise the minimal L1 node and explorer together:

* `scripts/e2e/run_l1_explorer_e2e.sh` – shell script that builds both crates, runs their unit tests, launches the servers, and performs basic RPC/HTTP checks using `curl`.
* `scripts/run_servers.sh` – simple helper that builds and starts the L1 node and explorer together; accepts optional bind addresses for public exposure.

### Running the prototypes

The L1 node and explorer binaries read `BIND_ADDR` to determine the listen address. By default they bind to `0.0.0.0:3000` and `0.0.0.0:4000` respectively, which makes them reachable from outside the container once ports are forwarded. You can override with environment variables or via the `run_servers.sh` script:

```sh
# start both services, listening on all interfaces
scripts/run_servers.sh

# custom addresses
scripts/run_servers.sh 0.0.0.0:3001 127.0.0.1:4005
```
* Crate-level integration tests (`flowcortex-l1/tests/e2e.rs` and `explorer/tests/e2e.rs`) which spawn the binary and use `reqwest` to verify core endpoints. Run with `cargo test --manifest-path <crate>/Cargo.toml`.

These tests help catch regressions as the prototypes evolve.
