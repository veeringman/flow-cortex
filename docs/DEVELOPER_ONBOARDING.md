# FlowCortex Developer Onboarding Guide

**Version:** 1.0  
**Date:** February 23, 2026  
**Estimated Reading Time:** 30 minutes

---

## Welcome to FlowCortex!

FlowCortex is a Layer-1 blockchain platform for anchoring authorization commitments and verifying zero-knowledge proofs. This guide will get you up to speed quickly.

---

## Architecture Overview (5 minutes)

### What Does FlowCortex Do?

**In Simple Terms:**
FlowCortex provides cryptographic proof that financial settlements were properly authorized.

**Three Core Operations:**
1. **Anchor Commitment**: FortressDigital records authorization decision
2. **Verify Proof**: ProofCortex submits STARK proof of policy compliance
3. **Query Status**: Treasury systems check if settlement is verified

### Key Components

```
flowcortex-l1/
├── src/
│   ├── ledger.rs          # Core: Commitment & proof storage
│   ├── node.rs            # Node: Transaction processing
│   ├── grpc/              # API: gRPC service handlers
│   ├── demo.rs            # Demo: Settlement scenarios
│   └── types.rs           # Types: Data structures
├── tests/
│   └── e2e.rs             # Tests: End-to-end integration
└── Cargo.toml
```

---

## 15-Minute Quick Start

### 1. Setup (5 min)

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repo
git clone https://github.com/flowcortex/flow-cortex
cd flow-cortex/flowcortex-l1

# Build
cargo build

# Run tests
cargo test
```

### 2. Start Node (2 min)

```bash
# Terminal 1: Start L1 node
cargo run

# You should see:
# FlowCortex L1 Node starting...
# Listening on http://127.0.0.1:3000
```

### 3. Try It Out (3 min)

```bash
# Terminal 2: Anchor a commitment
curl -X POST http://192.168.29.78:3000/api/anchor_commitment \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "policy_id": "test_policy",
    "txn_ref": "test_001",
    "timestamp": 1708704000
  }'

# Response:
# {"success":true,"block_height":1,"tx_hash":"txn_000..."}

# Verify a proof
curl -X POST http://192.168.29.78:3000/api/verify_proof \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "proof_hash": "b2c3d4e5f6789012345678901234567890abcdef01234567890abcdef012345678",
    "proof_data": "AgQGCAo=",
    "proof_type": "STARK",
    "capsule_version": "verifier_v1"
  }'

# Query status
curl http://192.168.29.78:3000/api/proof_status/a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef
```

### 4. Explore Code (5 min)

Open `src/ledger.rs` and find:
- Line 870: `anchor_commitment()` - See how commitments are anchored
- Line 972: `verify_proof()` - See how proofs are verified
- Line 2056: Tests - See comprehensive test suite

---

## Key Code Patterns

### Adding a New RPC Method

**Step 1:** Add to `src/grpc/mod.rs`:

```rust
pub async fn my_new_method(
    request: MyRequest
) -> Result<MyResponse, String> {
    // Your logic here
    Ok(MyResponse { ... })
}
```

**Step 2:** Register route in `src/grpc.rs`:

```rust
Router::new()
    .route("/api/my_method", post(my_new_method))
```

**Step 3:** Add test in `src/grpc/mod.rs`:

```rust
#[test]
fn test_my_new_method() {
    // Test your method
}
```

### Adding a New Event Type

**Step 1:** Add to `src/types.rs`:

```rust
pub enum CommitmentProofEvent {
    // Existing events...
    
    MyNewEvent {
        field1: String,
        field2: u64,
    },
}
```

**Step 2:** Emit in ledger operations:

```rust
self.commitment_proof_events.push(
    CommitmentProofEvent::MyNewEvent {
        field1: "value".to_string(),
        field2: 12345,
    }
);
```

---

## Testing Guide

### Run Specific Test

```bash
# Run one test
cargo test test_commitment_crud_operations

# Run all ledger tests
cargo test ledger::

# Run with output
cargo test -- --nocapture
```

### Write a Test

```rust
#[test]
fn test_my_feature() {
    let admin = "admin".to_string();
    let mut ledger = Ledger::new(admin);
    
    // Setup
    let commitment_hash = "a".repeat(64);
    
    // Action
    let result = ledger.anchor_commitment(...);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(ledger.commitments.len(), 1);
}
```

---

## Common Tasks

### Task: Add Validation to API

```rust
// Before
pub fn anchor_commitment(&mut self, hash: String) -> Result<...> {
    // Direct storage
}

// After
pub fn anchor_commitment(&mut self, hash: String) -> Result<...> {
    // Validate hash format
    if hash.len() != 64 {
        return Err("INVALID_HASH_FORMAT".to_string());
    }
    
    // Storage logic...
}
```

### Task: Add New Index for Lookups

```rust
// In Ledger struct
pub struct Ledger {
    commitments: HashMap<String, CommitmentRecord>,
    
    // Add new index
    policy_to_commitments: HashMap<String, Vec<String>>,
}

// Update on insert
self.commitments.insert(hash.clone(), commitment);
self.policy_to_commitments
    .entry(policy_id.clone())
    .or_insert_with(Vec::new)
    .push(hash.clone());
```

---

## Debugging Tips

### Enable Debug Logs

```bash
RUST_LOG=debug cargo run
```

### Print Ledger State

```rust
println!("Ledger state: {:#?}", ledger);
println!("Commitments: {}", ledger.commitments.len());
```

### Use Debugger

```toml
# Cargo.toml - Add to dev dependencies
[dev-dependencies]
pretty_assertions = "1.3"
```

```rust
use pretty_assertions::assert_eq;
```

---

## Resources

### Documentation
- API Specs: `docs/API_SPECIFICATIONS.md`
- Data Model: `docs/DATA_MODEL.md`
- Integration Guides: `docs/INTEGRATION_GUIDE_*.md`

### Code References
- Ledger operations: `src/ledger.rs`
- API handlers: `src/grpc/mod.rs`
- Types: `src/types.rs`
- Tests: `src/ledger.rs` (bottom of file)

### Getting Help
- Team Chat: #flowcortex-dev
- Code Reviews: GitHub PRs
- Weekly Sync: Thursdays 2pm
- Docs: https://docs.flowcortex.example.com

---

## Next Steps

1. **Read the integration guides** (30 min)
   - Understand how external teams use FlowCortex
   
2. **Run the test suite** (5 min)
   - Verify your setup works
   
3. **Pick a starter task** from GitHub Issues
   - Look for "good-first-issue" label
   
4. **Pair with a teammate** (1 hour)
   - Get a walkthrough of a real feature

---

## Welcome aboard! 🚀
