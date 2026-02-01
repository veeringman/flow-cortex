# FlowCortex L0  
An Ordering-Less, Payment-Centric State & Proof Layer

FlowCortex L0 is the foundational layer of the FlowCortex blockchain stack.  
It defines how state is represented, updated, committed, and verified — without assuming blocks, ordering, or consensus.

This layer is intentionally minimal, cryptographically sound, and extensible.

---

## 1. What is L0 in FlowCortex?

L0 is the state & proof substrate.

It answers one core question:

    How can a node verify correctness of payments
    without replaying global history or enforcing transaction order?

L0 provides:
- Ordering-less state updates
- Stateless verification
- Aggregated commitments
- Async-friendly verification
- A foundation for zk proofs, GPUs, and ASICs

No networking.  
No consensus.  
No blocks.

Those come later.

---

## 2. Core Ideas (High Level)

### Ordering-less
Transactions do not rely on a global sequence.  
They are verified against state commitments, not history.

### Stateless Verification
Validators do not need full state.  
Proofs plus commitments are sufficient.

### Payment-centric
The primary operation is a balance delta (credit / debit).  
Everything else builds on top.

---

## 3. Project Structure

    src/
    ├── main.rs      # End-to-end L0 demo
    ├── lib.rs       # Public module exports
    ├── types.rs     # Domain types (transactions, aggregates)
    ├── alc.rs       # Algebraic Commitment Layer
    ├── qct.rs       # Quantum Cascade Tree (state structure)
    └── verify.rs    # Verification engine

Each file has one responsibility.

---

## 4. Domain Types (types.rs)

### Transaction

    pub struct Transaction {
        pub key: Vec<u8>,
        pub amount: i64,
    }

Represents a state delta.

- key  
  Prefix-addressable identifier (account, user, device, etc.)

- amount  
  Positive = credit  
  Negative = debit  

There is no notion of sender/receiver here — only state change.

This abstraction allows:
- Payments
- Metering
- IoT aggregation
- Event accounting

---

### Frequency

    pub struct Frequency {
        pub count: u64,
        pub sum: i64,
    }

Aggregated statistics stored at every node.

Used for:
- Balance enforcement
- Range checks
- Fraud detection
- Analytics

This enables validation without scanning full state.

---

## 5. Algebraic Commitment Layer (alc.rs)

### Purpose

ALC commits to values in a way that is:
- Compact
- Verifiable
- Aggregatable
- Future-proof for zk and PQC

### Key Types

    Commitment(Vec<u8>)
    Blinding(u64)

A commitment hides a value using a blinding factor.

### Commit Flow

    commit(value) → (Commitment, Blinding)

This binds:
- the value
- the randomness

Later this evolves into:
- polynomial commitments
- vector commitments
- zk-friendly math

For L0 MVP, the interface matters more than the math.

---

## 6. Quantum Cascade Tree (QCT) (qct.rs)

### What is QCT?

QCT is the state structure of FlowCortex.

It replaces:
- Merkle trees
- Global ordering
- Sequential execution

### QCTNode

    pub struct QCTNode {
        pub prefix: Vec<u8>,
        pub commitment: Commitment,
        pub blinding: Blinding,
        pub frequency: Frequency,
        pub children: HashMap<u8, QCTNode>,
    }

Each node represents:
- A prefix of keys
- An aggregated commitment
- Aggregated statistics
- A routing point for deeper prefixes

---

### Prefix-Based Routing (Cascade Depth)

Transactions are routed by key prefix, one byte at a time.

This enables:
- Parallel updates
- Localized conflicts
- Natural sharding
- Logarithmic growth

No global ordering is required.

---

### Insert Flow

    Transaction
        ↓
    Update frequency
        ↓
    Recompute commitment
        ↓
    Route to child prefix
        ↓
    Repeat

Each level aggregates state independently.

---

## 7. Verification Engine (verify.rs)

### What verification means in L0

Verification checks:
- Commitments match aggregated values
- No hidden state manipulation
- Tree consistency holds

It does not:
- Re-execute history
- Enforce ordering
- Require full state

---

### Traversal-Based Verification

    verify_subtree(root)

Verification is implemented as an explicit tree traversal, not recursion.

Why:
- No async recursion boxing
- No stack growth
- GPU and zk friendly
- Deterministic memory usage

This mirrors how real proof systems work.

---

## 8. Why async Exists Here

Async is used for:
- Composition
- Future networking integration
- zk and proof backends

Not for threads.

Parallelism will later come from:
- Proof batching
- GPUs
- ASICs
- zk circuits

---

## 9. main.rs — L0 End-to-End Demo

Demonstrates:
1. Creating a QCT root
2. Inserting transactions
3. Verifying the entire state
4. Producing a boolean result

This is a reference flow, not production code.

---

## 10. What L0 Guarantees

L0 guarantees:
- State integrity
- Commitment correctness
- Aggregation validity

L0 does not decide:
- Finality
- Consensus
- Fork choice
- Fees

Those belong to higher layers.

---

## 11. How This Grows

From here, FlowCortex evolves into:
- ProofCortex (proof & witness layer)
- L1 consensus shell
- Stateless validators
- zk rollups
- Banking and payment rails
- IoT and data aggregation

Without rewriting L0.

---

## 12. One-Line Summary

FlowCortex L0 is an ordering-less, stateless, payment-centric state commitment layer designed for massive parallel verification and long-term cryptographic evolution.

This README documents intent, not just code.

