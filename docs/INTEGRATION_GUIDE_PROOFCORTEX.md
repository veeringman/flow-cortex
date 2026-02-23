# FlowCortex Integration Guide - ProofCortex Team

**Version:** 1.0  
**Date:** February 23, 2026  
**Contact:** flowcortex-integrations@example.com

---

## Overview

ProofCortex generates STARK proofs that FortressDigital authorization decisions comply with policy. You submit these proofs to FlowCortex for verification and permanent anchoring.

---

## Your Role in the Flow

1. FortressDigital makes policy decision
2. FortressDigital anchors commitment on FlowCortex
3. **→ You generate STARK proof of policy compliance**
4. **→ You submit proof to FlowCortex for verification**
5. If verified, settlement proceeds

---

## API Endpoint

**Environment:**
- Development: `https://dev-l1.flowcortex.example.com`
- Production: `https://l1.flowcortex.example.com`

**Endpoint:** `POST /api/verify_proof`

---

## Integration Example

### 1. Receive Commitment Hash

You'll receive the `commitment_hash` from the treasury system (it was anchored by FortressDigital).

### 2. Generate STARK Proof

```python
# Your internal STARK proof generation
proof_data = generate_stark_proof(
    authorization_context=auth_context,
    policy_rules=policy_rules
)

# Compute proof hash for tracking
import hashlib
proof_hash = hashlib.sha256(proof_data).hexdigest()
```

### 3. Submit to FlowCortex

```python
import requests
import base64

response = requests.post(
    "https://l1.flowcortex.example.com/api/verify_proof",
    headers={
        "Authorization": "Bearer YOUR_API_KEY",
        "Content-Type": "application/json"
    },
    json={
        "commitment_hash": commitment_hash,  # From step 1
        "proof_hash": proof_hash,
        "proof_data": base64.b64encode(proof_data).decode(),
        "proof_type": "STARK",
        "public_inputs": "user_id=alice,auth_level=high",
        "capsule_version": "verifier_v1"
    }
)

result = response.json()
# {
#   "success": true,
#   "verified": true,
#   "verification_block": 12346
# }
```

### 4. Handle Result

- **If `verified: true`**: Proof is valid, settlement can proceed
- **If `verified: false`**: Proof is invalid, settlement blocked

---

## Request Schema

```json
{
    "commitment_hash": "string (64 hex chars, must exist)",
    "proof_hash": "string (64 hex chars)",
    "proof_data": "base64 encoded bytes",
    "proof_type": "STARK",
    "public_inputs": "optional string",
    "capsule_version": "verifier_v1"
}
```

**Required Fields:**
- `commitment_hash`: Must reference an existing commitment (anchored by FortressDigital)
- `proof_hash`: SHA256 of your proof data (64 hex chars)
- `proof_data`: Your STARK proof bytes (base64 encoded)
- `proof_type`: "STARK" (other types reserved for future)
- `capsule_version`: Verifier version ("verifier_v1" for production)

**Optional Fields:**
- `public_inputs`: Public inputs used in proof generation

---

## Response Schema

**Success:**
```json
{
    "success": true,
    "proof_hash": "b2c3d4...",
    "verified": true,
    "verification_block": 12346
}
```

**Verification Failed:**
```json
{
    "success": false,
    "verified": false,
    "error_code": "PROOF_INVALID"
}
```

**Commitment Not Found:**
```json
{
    "success": false,
    "error_code": "COMMITMENT_NOT_FOUND"
}
```

---

## Error Codes

| Code | Meaning | Action |
|------|---------|--------|
| `COMMITMENT_NOT_FOUND` | Commitment not anchored yet | Wait for FortressDigital anchor |
| `PROOF_INVALID` | STARK verification failed | Check proof generation |
| `PROOF_ALREADY_VERIFIED` | Duplicate submission | Idempotent - use original result |
| `INVALID_PROOF_HASH` | Hash not 64 hex chars | Validate hash format |
| `INVALID_PROOF_FORMAT` | Empty proof data | Check proof encoding |

---

## Verifier Capsule

FlowCortex uses a "Verifier Capsule" to validate your STARK proofs:

**Current Version:** `verifier_v1`
**Algorithm:** STARK proof verification (deterministic)

**Verification Properties:**
- Deterministic: Same proof → same result
- Isolated: Capsule runs in isolated environment
- Versioned: Multiple capsule versions supported

**Future Versions:**
- `verifier_v2`: Enhanced STARK verifier with Poseidon hash
- Custom capsules can be registered for your specific proof format

---

## Replay Protection

FlowCortex prevents replay attacks:
- Same `proof_hash` for same `commitment_hash` can only be verified once
- Second submission returns `PROOF_ALREADY_VERIFIED`

---

## Testing

**Development Endpoint:**
```bash
curl -X POST https://dev-l1.flowcortex.example.com/api/verify_proof \
  -H "Authorization: Bearer dev_test_key_123" \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "proof_hash": "b2c3d4e5f6789012345678901234567890abcdef01234567890abcdef012345678",
    "proof_data": "AgQGCAoMDhASFA==",
    "proof_type": "STARK",
    "capsule_version": "verifier_v1"
  }'
```

**Mock Verifier Behavior (Development):**
- Proof data with even last byte → `verified: true`
- Proof data with odd last byte → `verified: false`

---

## SDK Support

**Python SDK:**
```python
from flowcortex_sdk import FlowCortexClient

client = FlowCortexClient(api_key="YOUR_KEY", environment="production")

result = client.verify_proof(
    commitment_hash="a1b2c3...",
    proof_hash="b2c3d4...",
    proof_data=proof_bytes,
    proof_type="STARK",
    capsule_version="verifier_v1"
)

if result.verified:
    print("Proof verified! Settlement can proceed.")
else:
    print(f"Verification failed: {result.error_code}")
```

---

## Rate Limits

- Development: 10 requests/second
- Production: 100 requests/second

---

## Support

- Technical Questions: integrations@flowcortex.example.com
- Capsule Registration: capsule-support@flowcortex.example.com
- Production Issues: support@flowcortex.example.com (24/7)
