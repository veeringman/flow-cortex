# FlowCortex Integration Guide - FortressDigital Team

**Version:** 1.0  
**Date:** February 23, 2026  
**Contact:** flowcortex-integrations@example.com

---

## Overview

FortressDigital anchors authorization commitments on FlowCortex after making policy decisions. This document describes the integration points and API usage.

---

## Your Role in the Flow

1. User authenticates (FIDO + OTP)
2. FortressDigital evaluates policy
3. **→ You anchor commitment hash on FlowCortex**
4. ProofCortex generates STARK proof
5. ProofCortex submits proof to FlowCortex
6. Settlement proceeds if verified

---

## API Endpoint

**Environment:**
- Development: `https://dev-l1.flowcortex.example.com`
- Production: `https://l1.flowcortex.example.com`

**Endpoint:** `POST /api/anchor_commitment`

---

## Integration Example

### 1. Generate Commitment Hash

After your policy decision, compute commitment hash:

```python
import hashlib
import json

# Your authorization decision data
auth_decision = {
    "user_id": "alice@bank.com",
    "device_trust": "high",
    "risk_score": 85,
    "policy_id": "policy_fortress_001",
    "decision": "allow",
    "amount": 50000000,
    "timestamp": 1708704000
}

# Compute SHA256 hash
data_str = json.dumps(auth_decision, sort_keys=True)
commitment_hash = hashlib.sha256(data_str.encode()).hexdigest()
# Result: 64 hex character string
```

### 2. Anchor on FlowCortex

```python
import requests

response = requests.post(
    "https://l1.flowcortex.example.com/api/anchor_commitment",
    headers={
        "Authorization": "Bearer YOUR_API_KEY",
        "Content-Type": "application/json"
    },
    json={
        "commitment_hash": commitment_hash,
        "policy_id": "policy_fortress_001",
        "txn_ref": "settlement_001",
        "timestamp": 1708704000,
        "context_ref": "amount:50000000,currency:INR"
    }
)

result = response.json()
# {
#   "success": true,
#   "block_height": 12345,
#   "tx_hash": "txn_000...",
# }
```

### 3. Return to Treasury System

Return the `commitment_hash` and `block_height` to the treasury system so ProofCortex can reference it when submitting proof.

---

## Request Schema

```json
{
    "commitment_hash": "string (64 hex chars)",
    "policy_id": "string (your policy identifier)",
    "txn_ref": "string (external settlement reference)",
    "timestamp": 1708704000,
    "context_ref": "optional metadata string"
}
```

**Required Fields:**
- `commitment_hash`: Exactly 64 hexadecimal characters (SHA256)
- `policy_id`: Your policy identifier (non-empty)
- `txn_ref`: Unique transaction reference from treasury system
- `timestamp`: Unix timestamp (seconds)

**Optional Fields:**
- `context_ref`: Additional context (e.g., amount, currency)

---

## Response Schema

**Success:**
```json
{
    "success": true,
    "commitment_hash": "a1b2c3...",
    "block_height": 12345,
    "tx_hash": "txn_000...",
    "timestamp": 1708704001
}
```

**Error:**
```json
{
    "success": false,
    "error_code": "INVALID_HASH_FORMAT"
}
```

---

## Error Codes

| Code | Meaning | Action |
|------|---------|--------|
| `INVALID_HASH_FORMAT` | Hash not 64 hex chars | Check hash computation |
| `INVALID_TXN_REF` | txn_ref empty or too long | Validate txn_ref |
| `CONFLICT_DETECTED` | Different commitment with same txn_ref | Check for duplicates |

---

## Idempotency

If you submit the same `commitment_hash` multiple times, FlowCortex returns the original `block_height`. The response will have:
- `success: true`
- Original `block_height`
- `tx_hash: "idempotent"`

This prevents duplicate anchoring.

---

## Rate Limits

- Development: 10 requests/second
- Production: 100 requests/second

Headers included:
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`

---

## Testing

**Development Endpoint:**
```bash
curl -X POST https://dev-l1.flowcortex.example.com/api/anchor_commitment \
  -H "Authorization: Bearer dev_test_key_123" \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "policy_id": "policy_test_001",
    "txn_ref": "test_txn_001",
    "timestamp": 1708704000
  }'
```

---

## SDK Support

**Python SDK:**
```python
from flowcortex_sdk import FlowCortexClient

client = FlowCortexClient(api_key="YOUR_KEY", environment="production")

result = client.anchor_commitment(
    commitment_hash="a1b2c3...",
    policy_id="policy_001",
    txn_ref="settle_001",
    timestamp=1708704000
)

print(f"Anchored at block {result.block_height}")
```

**Node.js SDK:**
```javascript
const FlowCortex = require('@flowcortex/sdk');

const client = new FlowCortex.Client({
  apiKey: 'YOUR_KEY',
  environment: 'production'
});

const result = await client.anchorCommitment({
  commitmentHash: 'a1b2c3...',
  policyId: 'policy_001',
  txnRef: 'settle_001',
  timestamp: 1708704000
});

console.log(`Anchored at block ${result.blockHeight}`);
```

---

## Support

- Technical Questions: integrations@flowcortex.example.com
- Production Issues: support@flowcortex.example.com (24/7)
- Documentation: https://docs.flowcortex.example.com
