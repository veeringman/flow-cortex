# FortressDigital + Stablecoin Settlement + Zero Proof Verification
## Demo Context Document

### 1. Demo Objective

The demo showcases an enterprise treasury settlement executed using a stablecoin (FloweR), where:

- A treasury platform initiates a settlement to a counterparty
- FortressDigital enforces Zero Trust security policies
- ProofCortex generates a STARK Zero-Knowledge Proof that the authorization complied with policy
- FlowCortex anchors:
  - Authorization commitment
  - Proof verification result

Settlement proceeds only after cryptographic verification.

This demonstrates: **"Provably compliant enterprise treasury settlement with Zero Trust + Zero Knowledge guarantees."**

### 2. Business Scenario (Narrative)

**Use Case**: A corporate treasury system sends a ₹50M stablecoin settlement to a supplier bank.

**Security and compliance requirements:**
- Strong user authentication (FIDO + OTP)
- Risk-aware policy decision (device + behavior + amount)
- Cryptographic proof that policy was followed
- Immutable audit trail verifiable by regulators

The demo proves that: **The settlement was not only authorized — it was provably authorized according to enterprise security policy.**

### 3. Platforms Involved in Demo

#### 3.1 Treasury Settlement Platform
- Simulated enterprise treasury UI
- Initiates payment
- Displays approval status
- Waits for "Provably Authorized" signal before final submission

#### 3.2 FortressDigital (Control Plane)
**Responsibilities:**
- Identity validation via IdP
- Risk scoring
- Policy evaluation
- Decision: Allow / Step-up / Block
- Generates commitment hash

**Commitment Hash Computation:**
```
C = H(user_id, device_trust, risk_score, policy_id, decision, txn_bucket, timestamp)
```

#### 3.3 ProofCortex (ZK Proof Engine)
**Responsibilities:**
- Accept policy decision inputs
- Generate STARK proof: "Policy rules were satisfied for this settlement"
- Submit proof to FlowCortex for verification

#### 3.4 FlowCortex (Anchor & Verifier Chain)
**Primary responsibilities in demo:**
- Anchor commitment from FortressDigital
- Accept STARK proof from ProofCortex
- Execute Verifier Capsule
- Emit immutable verification event
- Provide query APIs for audit & UI display

### 4. End-to-End Demo Flow (Ordered)

**Step 1 — User Initiates Settlement**
- Treasury UI: Pay ₹50,000,000 to Supplier Bank

**Step 2 — Authentication & Risk Evaluation**
- FortressDigital performs:
  - FIDO authentication
  - OTP step-up (for high amount)
  - Device trust & behavioral risk computation
- Example: risk_score = 72, auth_strength = HIGH, decision = ALLOW

**Step 3 — Commitment Generation**
- FortressDigital computes: `commitment_hash = H(context + decision)`
- This represents: The exact state under which authorization was granted

**Step 4 — Commitment Anchoring on FlowCortex**
- FortressDigital calls: `anchor_commitment(commitment_hash, policy_id, txn_ref)`
- FlowCortex records immutable inclusion: `CommitmentAnchored(commitment_hash, block_height)`

**Step 5 — STARK Proof Generation**
- ProofCortex receives:
  - Private inputs: risk_score, device_trust, auth_strength
  - Public inputs: policy_id, commitment_hash
- Proof statement: `policy_eval(private_inputs, policy_id) == true AND hash(private_inputs) == commitment_hash`
- Produces: π_stark

**Step 6 — Proof Submission to FlowCortex**
- ProofCortex calls: `verify_proof(commitment_hash, stark_proof, public_inputs)`
- FlowCortex Verifier Capsule:
  - Confirms commitment exists
  - Validates STARK proof
  - Confirms binding to commitment

**Step 7 — Verification Event Emission**
- FlowCortex emits: `ProofVerified(commitment_hash, status=VERIFIED)`
- This event signals: Authorization is cryptographically valid and immutable.

**Step 8 — Settlement Execution**
- Treasury platform observes: `status = PROVABLY_AUTHORIZED`
- Settlement transfer of FloweR stablecoin proceeds.

### 5. Data Objects Flowing Through FlowCortex

**5.1 Commitment Object**
```json
{
  "commitment_hash": "0xABC123",
  "policy_id": "treasury_settlement_v1",
  "txn_ref": "TXN-90877",
  "timestamp": 1710000000
}
```

**5.2 Proof Submission Object**
```json
{
  "commitment_hash": "0xABC123",
  "stark_proof": "<binary>",
  "public_inputs": {
    "policy_id": "treasury_settlement_v1",
    "txn_amount_bucket": "HIGH"
  }
}
```

### 6. What FlowCortex Demonstrates in Demo

**6.1 Anchoring Capability** ✅
- Immutable storage of commitments via `POST /api/anchor_commitment`
- Deterministic block inclusion with tx_hash and block_height
- Idempotent anchoring (same commitment → same response)

**6.2 Verifier Capsule Execution** ✅
- Native Rust verifier capsules (primary path via `POST /capsule/{id}/invoke`)
- WASM capsule runtime via wasmtime (`POST /capsule/{id}/invoke_wasm`)
- Capsule IDE in Explorer with WAT editor, example gallery, and wabt.js compilation
- Host functions: `host_mint`, `host_transfer`, `host_burn`, `host_balance`, `host_log`, `host_output`

**6.3 Proof Verification** ✅
- Accept proof submissions via `POST /api/verify_proof`
- Validate proof and bind to anchored commitment
- Query status via `GET /api/proof_status/{hash}`

**6.4 Event Emission** ✅
- CommitmentAnchored, ProofVerified events
- Event stream via `GET /api/events`
- Dashboard stats via `GET /api/stats`

**6.5 Settlement Operations** ✅
- FloweR stablecoin (FLW, 1:1 INR peg, 6 decimals)
- Settlement mint/redeem/transfer routes for approved banks
- Bank administration (approve, daily mint limits)
- Full token lifecycle: create, mint, transfer, burn

**6.6 Integration Points** ✅
- FortressDigital → FlowCortex: commitment anchoring via `FLOW_ANCHOR_MODE=http`
- ProofCortex → FlowCortex: proof verification via `POST /api/verify_proof`
- KeyCortex → FlowCortex: balance/transfer via HTTP client
- Explorer → FlowCortex: all read endpoints (11-tab UI)

### 7. Demo UI Signals (What Audience Will See)

| Stage | UI Indicator |
|-------|--------------|
| After policy decision | "Policy Approved (Awaiting Proof)" |
| After commitment anchor | "Decision Anchored on FlowCortex" |
| After proof verification | "Provably Authorized" |
| Final step | "Stablecoin Settlement Executed" |

This creates a powerful, visible trust chain.

### 8. Key Demo Security Narrative

The demo must clearly show:

1. Decision made by FortressDigital
2. Decision anchored immutably on FlowCortex
3. Proof generated independently by ProofCortex
4. FlowCortex verifies proof against anchored commitment
5. Only then settlement is allowed

Thus proving: **No hidden override, no tampering, no blind trust.**

### 9. Delivered MVP Capabilities

All capabilities required for demo are implemented and working:

- ✅ Commitment anchoring endpoint (`POST /api/anchor_commitment`)
- ✅ Verifier capsule runtime (native Rust + WASM/wasmtime)
- ✅ Proof submission and verification (`POST /api/verify_proof`)
- ✅ Deterministic event emission (`GET /api/events`)
- ✅ Query APIs for commitment & verification status
- ✅ FloweR stablecoin with full token lifecycle
- ✅ Settlement routes (mint/redeem/transfer) for approved banks
- ✅ Bank administration API
- ✅ Explorer UI with 11 tabs including Capsule IDE
- ✅ gRPC services (6 total: Ledger, BlockProducer, TxPool, Token, CommitmentAnchor, ProofVerifier)
- ✅ REST API (29 routes on port 3000)
- ✅ E2E test suite and demo scenarios

### 10. Summary

In this demo, FlowCortex serves as:

**The immutable trust anchor and deterministic verifier execution layer that records FortressDigital authorization commitments and validates ProofCortex STARK proofs, ultimately enabling provably compliant enterprise stablecoin settlement.**

All capabilities described in this document are implemented and integrated with FortressDigital, ProofCortex, KeyCortex, and TreasurySettlement.

---

**Document Created:** February 23, 2026
**Last Updated:** March 1, 2026
**Demo Context Version:** 2.0 — All capabilities delivered
