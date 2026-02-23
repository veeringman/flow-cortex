# FlowCortex Integration Guide - Treasury Settlement Team

**Version:** 1.0  
**Date:** February 23, 2026  
**Contact:** flowcortex-integrations@example.com

---

## Overview

Your treasury system initiates settlements. FlowCortex provides cryptographic proof that each settlement was properly authorized according to FortressDigital security policies.

---

## Settlement Flow

```
1. User initiates ₹50M settlement in your UI
2. You call FortressDigital for authorization
3. FortressDigital anchors commitment on FlowCortex
4. FortressDigital returns commitment_hash to you
5. You trigger ProofCortex to generate proof
6. ProofCortex submits proof to FlowCortex
7. You query FlowCortex for verification status
8. If verified → execute settlement
9. If not verified → block settlement
```

---

## API Endpoints

**Base URL:**
- Development: `https://dev-l1.flowcortex.example.com`
- Production: `https://l1.flowcortex.example.com`

**Key Endpoints:**
- Query proof status: `GET /api/proof_status/{commitment_hash}`
- Query commitment: `GET /api/commitment/{commitment_hash}`
- Subscribe to events: `WS /api/events/subscribe`

---

## Integration Example

### 1. Check Proof Status

After FortressDigital anchors commitment and ProofCortex submits proof:

```python
import requests

def check_verification_status(commitment_hash: str) -> dict:
    response = requests.get(
        f"https://l1.flowcortex.example.com/api/proof_status/{commitment_hash}",
        headers={"Authorization": "Bearer YOUR_API_KEY"}
    )
    return response.json()

# Usage
status = check_verification_status("a1b2c3d4e5f67890...")

if status.get("verified"):
    print("✅ Settlement is provably authorized - proceed")
    execute_settlement()
else:
    print("❌ No valid proof - block settlement")
    block_settlement()
```

### 2. Real-Time Event Subscription (Optional)

For real-time updates, subscribe to FlowCortex events:

```python
import websocket
import json

ws = websocket.create_connection(
    "wss://l1.flowcortex.example.com/api/events/subscribe",
    header={"Authorization": "Bearer YOUR_API_KEY"}
)

# Subscribe to specific commitment
ws.send(json.dumps({
    "action": "subscribe",
    "commitment_hash": "a1b2c3d4e5f67890..."
}))

# Receive events
while True:
    event = json.loads(ws.recv())
    if event["type"] == "ProofVerified":
        print("✅ Proof verified - settlement can proceed")
        break
    elif event["type"] == "ProofVerificationFailed":
        print("❌ Proof failed - block settlement")
        break
```

---

## Query APIs

### Get Proof Status

**Endpoint:** `GET /api/proof_status/{commitment_hash}`

**Response:**
```json
{
    "commitment_hash": "a1b2c3...",
    "verified": true,
    "proof": {
        "proof_hash": "b2c3d4...",
        "verification_block": 12346,
        "verified_at": 1708704010,
        "verifier_capsule_version": "verifier_v1"
    }
}
```

### Get Commitment Details

**Endpoint:** `GET /api/commitment/{commitment_hash}`

**Response:**
```json
{
    "commitment_hash": "a1b2c3...",
    "policy_id": "policy_fortress_001",
    "txn_ref": "settlement_001",
    "block_height": 12345,
    "timestamp": 1708704000,
    "verified": true
}
```

---

## Settlement Decision Logic

```python
def can_proceed_with_settlement(commitment_hash: str) -> bool:
    """
    Check if settlement can proceed based on FlowCortex verification.
    
    Returns:
        True if settlement is provably authorized
        False otherwise
    """
    try:
        status = check_verification_status(commitment_hash)
        
        # Must have verified proof
        if not status.get("verified"):
            return False
        
        # Additional checks
        proof = status.get("proof")
        if not proof:
            return False
        
        # Optional: Check proof age
        import time
        verified_at = proof.get("verified_at", 0)
        age_seconds = time.time() - verified_at
        
        if age_seconds > 300:  # 5 minutes
            print("⚠️ Warning: Proof is older than 5 minutes")
        
        return True
        
    except Exception as e:
        print(f"Error checking verification: {e}")
        return False  # Fail closed
```

---

## Demo Settlement API (Development Only)

For testing, FlowCortex provides demo settlement endpoints:

### Create Demo Settlement

```bash
curl -X POST https://dev-l1.flowcortex.example.com/api/demo/settlement \
  -H "Authorization: Bearer dev_test_key" \
  -d '{
    "from_account": "Bank_A",
    "to_account": "Bank_B",
    "amount_inr": 50000000,
    "reference": "DEMO_001"
  }'
```

### Execute Settlement Steps

```bash
# Step 1: Authorization
curl -X POST https://dev-l1.flowcortex.example.com/api/demo/settlement/{id}/step/1

# Step 2: Commitment Anchoring
curl -X POST https://dev-l1.flowcortex.example.com/api/demo/settlement/{id}/step/2

# ... through Step 8
```

---

## Error Handling

```python
class SettlementVerificationError(Exception):
    """Raised when settlement verification fails"""
    pass

def verify_and_settle(commitment_hash: str, settlement_data: dict):
    """
    Verify proof and execute settlement with proper error handling.
    """
    try:
        # Check verification
        if not can_proceed_with_settlement(commitment_hash):
            raise SettlementVerificationError(
                "Settlement not verified on FlowCortex"
            )
        
        # Execute settlement
        result = execute_settlement(settlement_data)
        
        # Log to audit trail
        log_settlement_audit({
            "commitment_hash": commitment_hash,
            "settlement_id": result["id"],
            "status": "completed",
            "verified_on_flowcortex": True
        })
        
        return result
        
    except SettlementVerificationError as e:
        # Block settlement
        log_settlement_audit({
            "commitment_hash": commitment_hash,
            "status": "blocked",
            "reason": str(e)
        })
        raise
```

---

## Best Practices

1. **Always verify before settling**: Don't execute settlements without FlowCortex verification
2. **Fail closed**: If verification check fails, block the settlement
3. **Audit trail**: Log all verification checks for regulatory compliance
4. **Timeout handling**: Set reasonable timeouts (5-10 seconds) for verification queries
5. **Retry logic**: Implement exponential backoff for transient failures

---

## SDK Support

**Python SDK:**
```python
from flowcortex_sdk import FlowCortexClient

client = FlowCortexClient(api_key="YOUR_KEY", environment="production")

# Check if settlement can proceed
if client.is_verified(commitment_hash="a1b2c3..."):
    execute_settlement()
else:
    block_settlement()
```

**Node.js SDK:**
```javascript
const FlowCortex = require('@flowcortex/sdk');
const client = new FlowCortex.Client({apiKey: 'YOUR_KEY'});

const verified = await client.isVerified('a1b2c3...');

if (verified) {
    await executeSettlement();
} else {
    await blockSettlement();
}
```

---

## Rate Limits

- Development: 10 requests/second
- Production: 100 requests/second

---

## Support

- Integration Help: integrations@flowcortex.example.com
- Production Issues: support@flowcortex.example.com (24/7)
- Documentation: https://docs.flowcortex.example.com
