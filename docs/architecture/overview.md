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

