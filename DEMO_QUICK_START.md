# FlowCortex Demo - Quick Start Guide

**Date:** February 23, 2026  
**Phase:** 13 Complete - Demo Ready

---

## Quick Demo Commands

### 1. Start the FlowCortex L1 Node

```bash
cd /workspaces/flow-cortex
./scripts/start-l1-node.sh
```

The node will start on `localhost:50051` (gRPC)

---

## Demo API Usage Examples

### Create a Demo Settlement

**Using curl:**
```bash
curl -X POST http://localhost:50051/demo/settlements \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_id": "demo_001",
    "amount": 5000000000
  }'
```

**Response:**
```json
{
  "success": true,
  "scenario_id": "demo_001",
  "config": {
    "amount": 5000000000,
    "currency": "INR",
    "sender": {
      "id": "BANK_A",
      "name": "Bank A - Commercial Bank"
    },
    "receiver": {
      "id": "BANK_B",
      "name": "Bank B - Investment Bank"
    }
  },
  "steps": [...]
}
```

---

### Execute Settlement Step-by-Step

**Step 1: Anchor Commitment**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/1
```

**Step 2: Blockchain Confirmation**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/2
```

**Step 3: Submit Proof**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/3
```

**Step 4: Verify Proof**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/4
```

**Step 5: Mint FloweR Tokens**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/5
```

**Step 6: Burn Collateral**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/6
```

**Step 7: Update Status**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/7
```

**Step 8: Emit Completion Event**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/steps/8
```

---

### Auto-Execute All Steps

**Quick demo mode:**
```bash
curl -X POST http://localhost:50051/demo/settlements/demo_001/auto-execute
```

This executes all 8 steps in sequence automatically.

---

### Get Settlement Status

```bash
curl http://localhost:50051/demo/settlements/demo_001
```

**Response:**
```json
{
  "success": true,
  "scenario_id": "demo_001",
  "config": {...},
  "steps": [...],
  "current_step": 8,
  "completion_percentage": 100,
  "is_complete": true,
  "commitment_hash": "a1b2c3d4...",
  "proof_hash": "x9y8z7...",
  "block_height": 1002,
  "started_at": 1708704000,
  "completed_at": 1708704120
}
```

---

### List All Settlements

```bash
curl http://localhost:50051/demo/settlements
```

**Response:**
```json
{
  "success": true,
  "scenarios": [
    {
      "scenario_id": "demo_001",
      "amount": "₹50000000",
      "sender": "Bank A - Commercial Bank",
      "receiver": "Bank B - Investment Bank",
      "current_step": 8,
      "completion_percentage": 100,
      "is_complete": true
    }
  ],
  "total_count": 1
}
```

---

### Get Real-Time Events

**All events:**
```bash
curl http://localhost:50051/demo/events
```

**Events for specific settlement:**
```bash
curl http://localhost:50051/demo/events?scenario_id=demo_001
```

**Response:**
```json
{
  "success": true,
  "events": [
    {
      "event_id": "evt_001",
      "event_type": "commitment.anchored",
      "commitment_hash": "a1b2c3d4...",
      "block_height": 1000,
      "timestamp": 1708704000,
      "details": "Settlement commitment anchored"
    },
    {
      "event_id": "evt_002",
      "event_type": "proof.verified",
      "commitment_hash": "a1b2c3d4...",
      "proof_hash": "x9y8z7...",
      "block_height": 1002,
      "timestamp": 1708704120,
      "details": "Proof verified successfully"
    }
  ],
  "total_count": 2
}
```

---

### Get Dashboard Statistics

```bash
curl http://localhost:50051/demo/stats
```

**Response:**
```json
{
  "total_settlements": 1,
  "completed_settlements": 1,
  "in_progress_settlements": 0,
  "total_events": 8,
  "total_commitments": 1,
  "total_proofs": 1,
  "total_value_formatted": "₹50000000 / 50000000.000000 FLOWER",
  "block_height": 1002
}
```

---

### Reset Settlement

```bash
curl -X DELETE http://localhost:50051/demo/settlements/demo_001
```

---

## Demo Scenarios

### Scenario 1: Complete Settlement Flow

```bash
# Create settlement
curl -X POST http://localhost:50051/demo/settlements \
  -H "Content-Type: application/json" \
  -d '{"scenario_id": "settlement_001", "amount": 5000000000}'

# Auto-execute all steps
curl -X POST http://localhost:50051/demo/settlements/settlement_001/auto-execute

# Check status
curl http://localhost:50051/demo/settlements/settlement_001

# View events
curl http://localhost:50051/demo/events?scenario_id=settlement_001
```

---

### Scenario 2: Multiple Concurrent Settlements

```bash
# Create multiple settlements with different amounts
for i in {1..10}; do
  curl -X POST http://localhost:50051/demo/settlements \
    -H "Content-Type: application/json" \
    -d "{\"scenario_id\": \"settlement_00$i\", \"amount\": $((i * 1000000000))}"
done

# Auto-execute all
for i in {1..10}; do
  curl -X POST http://localhost:50051/demo/settlements/settlement_00$i/auto-execute
done

# List all settlements
curl http://localhost:50051/demo/settlements

# View stats
curl http://localhost:50051/demo/stats
```

---

### Scenario 3: Step-by-Step with Monitoring

```bash
# Create settlement
curl -X POST http://localhost:50051/demo/settlements \
  -H "Content-Type: application/json" \
  -d '{"scenario_id": "monitored_001", "amount": 10000000000}'

# Execute steps one by one with status checks
for step in {1..8}; do
  echo "Executing step $step..."
  curl -X POST http://localhost:50051/demo/settlements/monitored_001/steps/$step
  echo "\n\nStatus after step $step:"
  curl http://localhost:50051/demo/settlements/monitored_001
  echo "\n\n---\n"
  sleep 2
done
```

---

## Expected Outputs

### Step 1 Output (Anchor Commitment)
```json
{
  "success": true,
  "scenario_id": "demo_001",
  "step_number": 1,
  "step_name": "Anchor Settlement Commitment",
  "message": "Commitment anchored successfully",
  "commitment_hash": "generated_hash_here...",
  "block_height": 1000
}
```

### Step 3 Output (Submit Proof)
```json
{
  "success": true,
  "scenario_id": "demo_001",
  "step_number": 3,
  "step_name": "Submit STARK Proof",
  "message": "Proof submitted successfully",
  "commitment_hash": "previous_commitment_hash...",
  "proof_hash": "generated_proof_hash..."
}
```

### Step 5 Output (Mint FloweR)
```json
{
  "success": true,
  "scenario_id": "demo_001",
  "step_number": 5,
  "step_name": "Mint FloweR Tokens",
  "message": "Minted 50000000.000000 FLOWER to Bank B - Investment Bank"
}
```

---

## FloweR Token Calculations

### INR to FLOWER Conversion
```
Amount: ₹50,000,000 (50M rupees)
      = 5,000,000,000 paise (since 1 rupee = 100 paise)
      
FLOWER: 1 INR = 1 FLOWER (1:1 peg)
        1 FLOWER = 1,000,000 base units (6 decimals)
        
Result: 5,000,000,000 paise × 10,000 = 50,000,000,000,000 base units
        = 50,000,000.000000 FLOWER
```

### FLOWER to INR Conversion
```
Amount: 50,000,000.000000 FLOWER
      = 50,000,000,000,000 base units
      
INR: base_units / 10,000 = paise
     50,000,000,000,000 / 10,000 = 5,000,000,000 paise
     = ₹50,000,000
```

---

## Troubleshooting

### Settlement Not Found
**Error:** `"Scenario not found: demo_001"`  
**Solution:** Create the settlement first using POST /demo/settlements

### Cannot Execute Step Out of Order
**Error:** `"Cannot complete step 5. Current step is 3"`  
**Solution:** Execute steps sequentially (1 → 2 → 3 → ... → 8)

### Commitment Already Anchored
**Behavior:** Idempotent - returns same response  
**Expected:** This is correct behavior, not an error

---

## Testing Checklist

- [ ] Create settlement ✓
- [ ] Execute step 1 (anchor) ✓
- [ ] Execute step 2 (confirm) ✓
- [ ] Execute step 3 (submit proof) ✓
- [ ] Execute step 4 (verify) ✓
- [ ] Execute step 5 (mint tokens) ✓
- [ ] Execute step 6 (burn collateral) ✓
- [ ] Execute step 7 (update status) ✓
- [ ] Execute step 8 (emit event) ✓
- [ ] Check final status (100% complete) ✓
- [ ] View all events ✓
- [ ] Check dashboard stats ✓
- [ ] Test multiple concurrent settlements ✓
- [ ] Test auto-execute mode ✓

---

## Integration Points

### For FortressDigital
```rust
// Call FlowCortex to anchor settlement
let response = client.anchor_commitment(AnchorCommitmentRequest {
    commitment_hash: calculate_commitment_hash(&settlement),
    txn_ref: settlement.reference_id,
    amount: settlement.amount,
    metadata: settlement.to_metadata(),
}).await?;

// Store block_height and tx_hash for tracking
settlement.block_height = response.block_height;
settlement.tx_hash = response.tx_hash;
```

### For ProofCortex
```rust
// Generate and submit proof
let proof_data = generate_stark_proof(&settlement);
let response = client.verify_proof(VerifyProofRequest {
    commitment_hash: settlement.commitment_hash,
    proof_hash: calculate_proof_hash(&proof_data),
    proof_data: proof_data,
    proof_type: "STARK".to_string(),
}).await?;

// Check verification result
if response.verified {
    println!("Proof verified at block {}", response.block_height);
}
```

---

## Demo Presentation Tips

1. **Start Simple:** Create one settlement and auto-execute
2. **Show Monitoring:** Use status API to watch progress
3. **Demonstrate Events:** Show real-time event stream
4. **Scale Up:** Create 10 concurrent settlements
5. **Show Stats:** Display dashboard statistics
6. **Explain Security:** Highlight immutability and proof binding
7. **Discuss Compliance:** Show audit trail and event log

---

## Next Steps

After completing the demo:

1. **Phase 14:** Run comprehensive test suite
2. **Phase 15:** Complete API documentation
3. **Phase 16:** Deploy demo dashboard UI
4. **Production:** Integrate real STARK verifier from ProofCortex

---

**Ready to Demo!** 🚀

All Phase 13 features implemented and ready for demonstration.
