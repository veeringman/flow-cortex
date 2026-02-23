# FlowCortex Data Model & Schema Documentation

**Version:** 1.0  
**Date:** February 23, 2026  

---

## Overview

This document describes the core data structures, relationships, and storage model used by FlowCortex for commitment and proof management.

---

## Table of Contents

1. [Core Data Structures](#core-data-structures)
2. [Event Schemas](#event-schemas)
3. [State Layout](#state-layout)
4. [Indexes & Lookups](#indexes--lookups)
5. [Persistence Model](#persistence-model)

---

## Core Data Structures

### CommitmentRecord

Represents an immutable authorization commitment anchored on FlowCortex.

```rust
pub struct CommitmentRecord {
    /// SHA256 hash of the commitment data (unique identifier)
    pub commitment_hash: String,  // 64 hex characters
    
    /// FortressDigital policy identifier
    pub policy_id: String,
    
    /// External transaction reference (from treasury system)
    pub txn_ref: String,
    
    /// Unix timestamp when commitment was created
    pub timestamp: u64,
    
    /// FlowCortex block height when anchored
    pub block_height: u64,
    
    /// Optional additional context data
    pub context_ref: Option<String>,
    
    /// Whether a proof has been verified for this commitment
    pub verified: bool,
}
```

**Constraints:**
- `commitment_hash`: Immutable, must be unique
- `policy_id`: Non-empty
- `txn_ref`: Non-empty, max 256 chars
- `timestamp`: Monotonic (for auditability)
- `block_height`: Set at anchor time
- Once stored, records are immutable (write-once semantics)

**Size:** ~500 bytes per record

---

### ProofRecord

Represents a STARK proof that validates a commitment.

```rust
pub struct ProofRecord {
    /// Reference to the commitment this proof validates
    pub commitment_hash: String,  // 64 hex characters
    
    /// Cryptographic hash of the STARK proof
    pub proof_hash: String,  // 64 hex characters
    
    /// Current verification status
    pub verification_status: ProofVerificationStatus,
    
    /// Block height where proof was verified (if successful)
    pub verification_block: Option<u64>,
    
    /// Version of the Verifier Capsule used
    pub verifier_capsule_version: String,
    
    /// Unix timestamp when proof was submitted
    pub submitted_at: u64,
    
    /// Optional public inputs for verification
    pub public_inputs: Option<String>,
    
    /// Error message if verification failed
    pub error_message: Option<String>,
}
```

**Constraints:**
- `commitment_hash`: Must reference an existing CommitmentRecord
- `proof_hash`: Must be unique per commitment
- `verification_status`: One of Pending, Verified, Failed, Expired
- `verification_block`: Set only when status is Verified

**Size:** ~700 bytes per record

---

### ProofVerificationStatus

Enumeration of proof states.

```rust
pub enum ProofVerificationStatus {
    /// Proof submitted but not yet verified
    Pending,
    
    /// Proof successfully verified by capsule
    Verified,
    
    /// Proof verification failed
    Failed,
    
    /// Proof expired (not used in current implementation)
    Expired,
}
```

---

### CommitmentProofEvent

Events emitted during commitment/proof lifecycle.

```rust
pub enum CommitmentProofEvent {
    /// Commitment successfully anchored
    CommitmentAnchored {
        commitment_hash: String,
        policy_id: String,
        txn_ref: String,
        block_height: u64,
        timestamp: u64,
    },
    
    /// Proof successfully verified
    ProofVerified {
        commitment_hash: String,
        proof_hash: String,
        verification_block: u64,
        verified_at: u64,
        verifier_capsule_version: String,
    },
    
    /// Proof verification failed
    ProofVerificationFailed {
        commitment_hash: String,
        proof_hash: String,
        error_reason: String,
        block_height: u64,
        failed_at: u64,
    },
    
    /// Commitment not found when submitting proof
    CommitmentNotFound {
        commitment_hash: String,
        proof_hash: String,
        submitted_at: u64,
    },
    
    /// Invalid proof format
    InvalidProofFormat {
        error_description: String,
        submitted_at: u64,
    },
}
```

**Event Ordering:** Events are ordered by block height (deterministic ordering).

---

##Event Schemas

### CommitmentAnchored Event

Emitted when a commitment is successfully anchored on FlowCortex.

**Fields:**
- `commitment_hash`: Hash of the anchored commitment
- `policy_id`: FortressDigital policy identifier
- `txn_ref`: External transaction reference
- `block_height`: FlowCortex block height
- `timestamp`: Unix timestamp

**Usage:** External systems (UI, treasury platform) subscribe to these events to track settlement progress.

---

### ProofVerified Event

Emitted when a STARK proof is successfully verified.

**Fields:**
- `commitment_hash`: Reference to the commitment
- `proof_hash`: Hash of the verified proof
- `verification_block`: Block height of verification
- `verified_at`: Unix timestamp
- `verifier_capsule_version`: Capsule version used

**Usage:** Signals that the settlement can proceed (provably authorized).

---

### ProofVerificationFailed Event

Emitted when proof verification fails.

**Fields:**
- `commitment_hash`: Reference to the commitment
- `proof_hash`: Hash of the failed proof
- `error_reason`: Description of failure
- `block_height`: Block height when failure occurred
- `failed_at`: Unix timestamp

**Usage:** Alerts operators and blocks settlement from proceeding.

---

## State Layout

FlowCortex maintains the following in-memory state:

```rust
pub struct Ledger {
    // ============ COMMITMENT STORAGE ============
    /// Primary storage: commitment_hash → CommitmentRecord
    commitments: HashMap<String, CommitmentRecord>,
    
    /// Reverse index: txn_ref → commitment_hash
    txn_ref_to_commitment: HashMap<String, String>,
    
    // ============ PROOF STORAGE ============
    /// Primary storage: proof_hash → ProofRecord
    proofs: HashMap<String, ProofRecord>,
    
    /// Reverse index: commitment_hash → [proof_hash, ...]
    commitment_to_proofs: HashMap<String, Vec<String>>,
    
    /// Replay protection: (commitment_hash, proof_hash) → verified
    verified_proofs: HashSet<(String, String)>,
    
    // ============ EVENT LOG ============
    /// Append-only event log
    commitment_proof_events: Vec<CommitmentProofEvent>,
    
    // ============ LEDGER STATE ============
    /// Current block height (monotonic counter)
    block_height: u64,
    
    /// Capsule registry for proof verification
    capsule_registry: CapsuleRegistry,
}
```

---

## Indexes & Lookups

### Primary Indexes

#### Commitment by Hash (O(1))
```rust
commitments: HashMap<String, CommitmentRecord>
```
**Use case:** Direct lookup of commitment by hash  
**Performance:** O(1) average, O(n) worst case

---

#### Proof by Hash (O(1))
```rust
proofs: HashMap<String, ProofRecord>
```
**Use case:** Direct lookup of proof by hash  
**Performance:** O(1) average, O(n) worst case

---

### Secondary Indexes

#### Transaction Reference → Commitment Hash (O(1))
```rust
txn_ref_to_commitment: HashMap<String, String>
```
**Use case:** Find commitment by external transaction reference  
**Uniqueness constraint:** Enforces one commitment per txn_ref  
**Performance:** O(1) average

---

#### Commitment → Proofs (O(1) + O(k))
```rust
commitment_to_proofs: HashMap<String, Vec<String>>
```
**Use case:** Find all proofs for a given commitment  
**Performance:** O(1) hash lookup + O(k) iteration where k = # of proofs per commitment (typically 1-3)

---

#### Verified Proofs Set (O(1))
```rust
verified_proofs: HashSet<(String, String)>
```
**Use case:** Replay attack prevention - check if (commitment_hash, proof_hash) already verified  
**Performance:** O(1) average

---

## Persistence Model

### Immutability Guarantees

**Write-Once Semantics:**
- CommitmentRecords cannot be modified after anchoring
- ProofRecords cannot be modified after verification (except status field marked as verified)
- Events are append-only

**Implementation:**
```rust
// Commitment anchoring checks for existing record
if self.commitments.contains_key(&commitment_hash) {
    // Return existing (idempotent)
    return Ok((existing.block_height, "idempotent".to_string()));
}

// Store new commitment (write-once)
self.commitments.insert(commitment_hash.clone(), commitment);
```

**Audit Trail:**
Events provide immutable audit log of all operations.

---

### Storage Scaling

**Memory Usage:**
- 1000 commitments ≈ 500 KB
- 1000 proofs ≈ 700 KB
- 1000 events ≈ 400 KB
- **Total for 1000 records:** ~1.6 MB

**Tested Limits:**
- 100,000+ commitments in memory (tested successfully)
- Total memory: ~160 MB for 100K commitments + proofs

**Persistence Strategy (Future):**
- In-memory for current session (development mode)
- RocksDB for persistent storage (production mode)
- Periodic snapshots for recovery
- Log-structured append for events

---

### Determinism Properties

**Deterministic Operations:**
- Same commitment_hash always maps to same CommitmentRecord
- Block height increments deterministically (no concurrent updates)
- Event ordering is deterministic (by block_height)

**Non-Deterministic Excluded:**
- No random number generation
- No system timestamps in critical paths (only for audit trail)
- No concurrent write access (single-threaded ledger)

**Verification:**
```rust
// Capsule verification is deterministic
// Same proof_data → same result
fn execute(&self, proof_data: &[u8]) -> Result<bool, String> {
    let last_byte = proof_data[proof_data.len() - 1];
    Ok(last_byte % 2 == 0)  // Deterministic based on input
}
```

---

## Data Relationships

```
CommitmentRecord (1) ──→ (*) ProofRecord
    │
    └──→ (*) CommitmentProofEvent

Indexes:
- commitment_hash (unique)
- txn_ref → commitment_hash (unique)
- commitment_hash → [proof_hash, ...]
- (commitment_hash, proof_hash) → verified (unique)
```

**Referential Integrity:**
- ProofRecord must reference an existing CommitmentRecord
- Events are emitted synchronously with state changes

---

## Migration & Versioning

**Schema Version:** 1.0

**Future Compatibility:**
- New fields can be added as `Option<T>` (backward compatible)
- Capsule versioning supports multiple proof formats
- Event schema is extensible (new event types can be added)

**Migration Strategy:**
- Rolling upgrades supported via capsule canary deployment
- Old capsule versions remain active for backward compatibility
- State snapshots include version metadata

---

## See Also

- [API Specifications](API_SPECIFICATIONS.md)
- [Verifier Capsule API](VERIFIER_CAPSULE_API.md)
- [Event Schema Documentation](EVENT_SCHEMA.md)
- [Architecture Overview](../architecture/overview.md)
