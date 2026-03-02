# Detailed Expectations from FlowCortex Platform
## Engineering Contract for Development Team

### 1. Core Role in the Demo Architecture

FlowCortex acts as:

**A deterministic anchoring and verification execution layer that immutably records authorization commitments and validates STARK proofs to produce independently verifiable settlement authorization.**

All capabilities described below are **implemented and working**.

---

### 2. Functional Capabilities (Delivered)

#### 2.1 Commitment Anchoring (from FortressDigital) ✅

**Endpoint:** `POST /api/anchor_commitment`

**Implementation:**
- Accepts commitment_hash, policy_id, txn_ref, timestamp, context_ref
- Records immutably in ledger with block_height and tx_hash
- Idempotent: same commitment submitted twice → same response
- Rejects different commitment with same txn_ref
- Called by FortressDigital via `HttpFlowAnchorClient` (`FLOW_ANCHOR_MODE=http`)

#### 2.2 Verifier Capsule Runtime (for ProofCortex) ✅

**Endpoints:** `POST /capsule/{id}/invoke` (native), `POST /capsule/{id}/invoke_wasm` (WASM)

**Implementation:**
- Native Rust verifier capsules (primary path)
- WASM capsule runtime via wasmtime with sandboxed host functions
- Host functions: `host_mint`, `host_transfer`, `host_burn`, `host_balance`, `host_log`, `host_output`
- Ledger operations accumulated during execution, applied atomically on success
- Capsule IDE in Explorer for development and testing

#### 2.3 Proof Verification Binding Logic ✅

**Endpoint:** `POST /api/verify_proof`

FlowCortex must enforce the invariant:

```
verify(stark_proof, public_inputs) == true
AND
public_inputs.commitment_hash == anchored_commitment
```

This ensures:
- Proof correctness
- Binding of proof to the exact anchored decision
- Protection from replay attacks

---

### 3. Interface Implementation

#### 3.1 Commitment Anchoring API ✅

**Endpoint:** `POST /api/anchor_commitment`
**Used by**: FortressDigital after policy decision.

**Behavior**:
- Validate request structure
- Persist commitment deterministically
- Emit anchor event
- Return inclusion metadata

**Failure Semantics**:
- Invalid hash format → reject
- Duplicate conflicting context → reject
- Duplicate identical anchor → return existing tx reference

#### 3.2 Proof Submission API ✅

**Endpoint:** `POST /api/verify_proof`
**Used by**: ProofCortex after STARK proof generation.

**Behavior**:
- Check commitment exists
- Execute verifier capsule
- Validate proof correctness
- Emit verification result event
- Store proof reference (hash/index)

**Failure Semantics**:
- Missing commitment → reject
- Invalid proof → reject + emit failure event
- Hash mismatch → reject deterministically

#### 3.3 Query Interfaces (Read APIs) ✅

FlowCortex provides deterministic read APIs for:
- `GET /api/commitment/{hash}` — Commitment lookup by hash
- `GET /api/proof_status/{hash}` — Proof verification status
- `GET /blocks` — Block inclusion details
- `GET /api/events` — Event retrieval for dashboards
- `GET /api/stats` — Dashboard statistics

These will power:
- Treasury UI status updates
- FortressDigital audit console
- External auditor verification

---

### 4. Data Model Expectations

#### 4.1 Commitment Record

Must minimally store:
- `commitment_hash`
- `policy_id`
- `timestamp`
- `block_height`
- `context_ref` (txn identifier)

#### 4.2 Proof Record

Must minimally store:
- `commitment_hash`
- `proof_hash`
- `verification_status`
- `verification_block`
- `verifier_capsule_version`

---

### 5. Event Emission Expectations

FlowCortex must emit structured deterministic events:

**Event 1: CommitmentAnchored**
- Signals immutable recording of decision context.

**Event 2: ProofVerified**
- Signals cryptographic validation of authorization correctness.

These events are critical for demo storytelling and UI synchronization.

---

### 6. Determinism & Consensus Expectations

Since FlowCortex is used as a trust anchor, it must ensure:

- Deterministic execution of verifier capsule
- Same proof + inputs always produce same result
- No nondeterministic ordering or race outcomes
- Verifiable block inclusion order

This is essential for auditor replay and regulatory confidence.

---

### 7. Security Expectations

FlowCortex must guarantee:

#### 7.1 Immutability
- Anchored commitments cannot be altered.

#### 7.2 Replay Protection
- Proof cannot be reused against a different commitment.

#### 7.3 Integrity Binding
- Proof must cryptographically correspond to anchored commitment.

#### 7.4 Verifier Isolation
- Capsule execution must be sandboxed and deterministic.

---

### 8. Performance Expectations (Demo Scope)

| Operation | Target |
|-----------|--------|
| Anchor commitment | < 50 ms |
| Proof verification | < 100 ms |
| Query status | < 20 ms |
| Event propagation | near-real-time |

These numbers ensure a smooth live demo experience.

---

### 9. Failure & Edge Case Handling

FlowCortex must clearly define deterministic responses for:

- Commitment missing when proof submitted
- Invalid or malformed STARK proof
- Duplicate proof submission
- Commitment/proof hash mismatch
- Capsule execution failure

All failures must:
- Return explicit error code
- Emit a failure event
- Leave immutable audit trace

---

### 10. Versioning Expectations

FlowCortex should support:

- Versioned verifier capsules (verifier_v1, verifier_v2)
- Future upgrade to real STARK verifier
- Backward compatibility for already anchored commitments

This is important for evolving ProofCortex circuits.

---

### 11. Delivered MVP (All Complete)

All capabilities required for demo are implemented and working:

- ✅ Commitment anchoring endpoint (`POST /api/anchor_commitment`)
- ✅ Verifier capsule runtime (native Rust + WASM/wasmtime)
- ✅ Proof submission endpoint (`POST /api/verify_proof`)
- ✅ Deterministic event emission (`GET /api/events`)
- ✅ Query APIs for commitment & verification status
- ✅ FloweR stablecoin with full token lifecycle
- ✅ Settlement routes (mint/redeem/transfer) for approved banks
- ✅ Bank administration API (approve, daily limits)
- ✅ Explorer UI (11 tabs including Capsule IDE)
- ✅ gRPC services (6 total)
- ✅ 29 REST API routes on port 3000
- ✅ E2E test suite

---

**Document Created:** February 23, 2026
**Last Updated:** March 1, 2026
**Expectations Version:** 2.0 — All delivered
