# FlowCortex Architecture Overview

FlowCortex is a payment-centric, ordering-less blockchain designed around **stateless verification**, **parallel execution**, and **proof-driven correctness**.

This document provides a high-level architectural view of FlowCortex, explaining how its core components fit together and why the system fundamentally departs from traditional block-based blockchains.

---

## 1. Architectural Philosophy

Most blockchains assume the world is sequential:
- transactions are totally ordered
- state is replayed globally
- correctness emerges from execution order

FlowCortex rejects this assumption.

Instead, FlowCortex is built on three principles:

1. **State, not history, defines correctness**
2. **Proofs replace replay**
3. **Ordering is a last resort, not a default**

This allows FlowCortex to scale payments without scaling coordination.

---

## 2. Layered Architecture

FlowCortex is organized into **orthogonal layers**, each with a clear responsibility.

![FlowCortex Architecture Overview](/docs/assets/flowcortex.architecture.overview.png)


Each layer can evolve independently without breaking the others.

---

## 3. Application Layer (Payments First)

FlowCortex is designed **from day one** for payments.

The base transaction primitives assume:
- balances
- transfers
- batching
- settlement

Smart contracts may exist later, but payments are the native operation.

This avoids general-purpose complexity while enabling:
- predictable performance
- simpler invariants
- hardware acceleration

---

## 4. FlowGraph: Ordering-Less Transaction Layer

FlowGraph replaces blocks, mempools, and sequencers.

### Key Properties
- Transactions form a **DAG**, not a chain
- Each transaction explicitly declares:
  - state reads
  - state writes
- No implicit global ordering

### Consequences
- Non-conflicting transactions execute independently
- Conflicts are local and explicit
- Throughput scales with state partitioning

FlowGraph ensures the system remains live even under extreme load.

---

## 5. QCT: Quantum Cascade Tree

QCT is the **cryptographic backbone** of FlowCortex.

It enables:
- Stateless validation
- Multi-key proofs
- Range and aggregate checks
- Snapshot-based verification

### QCT Structure
Each node in the tree contains:
1. **Hash commitments** for structure
2. **Polynomial commitments (ALC)** for compact multi-key proofs
3. **Frequency aggregates** for payment safety

QCT proofs grow with **keys touched**, not with total state size.

---

## 6. Stateless Validation Model

Validators in FlowCortex:
- do not store full state
- do not replay transaction history
- do not depend on blocks

Instead:
- each transaction carries all required proofs
- validators verify proofs against a snapshot root
- only local state deltas are applied

This enables:
- lightweight nodes
- fast sync
- horizontal validator scaling

---

## 7. Finality Without Blocks

Finality in FlowCortex is **state-based**, not height-based.

- Transactions finalize against QCT snapshot roots
- Conflicts are resolved locally
- Finality is monotonic and irreversible

There are:
- no chain reorganizations
- no global rollbacks
- no fork-choice rules

Finality emerges from proof validity and conflict resolution.

---

## 8. Conflict Detection & Resolution

Conflicts arise only when:
- two transactions attempt to write the same state key
- or violate shared aggregate constraints

Resolution is handled by:
- local re-evaluation
- deferred execution
- rejection if invariants fail

This replaces global contention with bounded local contention.

---

## 9. Parallelism & Hardware Acceleration

FlowCortex is designed for parallel hardware.

- Proof verification is embarrassingly parallel
- Polynomial checks map naturally to GPUs
- Future ASICs can accelerate payment validation

Unlike sequential blockchains, FlowCortex benefits directly from faster hardware.

---

## 10. Runtime & Language Choices

### Rust
Chosen for:
- memory safety
- deterministic execution
- high-performance cryptography

### WASM
Used for:
- light clients
- constrained environments
- portable execution

The core protocol remains deterministic across runtimes.

---

## 11. Security Model (High-Level)

FlowCortex guarantees:
- no double-spend under valid proofs
- no overdraft through aggregate constraints
- explicit and auditable state access

Security relies on:
- cryptographic soundness of QCT
- honest majority for finality
- bounded adversarial contention

Formal proofs are addressed in separate documents.

---

## 12. Why This Architecture Matters

FlowCortex does not attempt to:
- out-sequence existing chains
- out-optimize block production
- hide coordination costs

Instead, it removes unnecessary coordination entirely.

The result is a system where:
- payments scale naturally
- verification is cheap
- ordering exists only where it must

---

## Architectural Maxim

> **FlowCortex does not ask the network to agree on order.  
> It asks the network to agree on correctness.**
