# FlowCortex API Specifications

**Version:** 1.0  
**Date:** February 23, 2026  
**Status:** Production  

---

## Overview

FlowCortex provides a comprehensive API for anchoring authorization commitments and verifying cryptographic proofs. This document describes all available endpoints, request/response schemas, error codes, and usage examples.

---

## Table of Contents

1. [Commitment Anchoring API](#commitment-anchoring-api)
2. [Proof Verification API](#proof-verification-api)
3. [Query APIs](#query-apis)
4. [Demo Settlement APIs](#demo-settlement-apis)
5. [Error Codes](#error-codes)
6. [Authentication](#authentication)

---

## Commitment Anchoring API

### Anchor Commitment

Anchors an immutable authorization commitment on FlowCortex.

**Endpoint:** `POST /api/anchor_commitment`  
**gRPC Method:** `AnchorCommitment`

#### Request Schema

```protobuf
message AnchorCommitmentRequest {
    string commitment_hash = 1;  // SHA256 hash (64 hex chars)
    string policy_id = 2;        // FortressDigital policy identifier
    string txn_ref = 3;          // External transaction reference
    uint64 timestamp = 4;        // Unix timestamp (seconds)
    optional string context_ref = 5;  // Additional context data
}
```

**JSON Example:**
```json
{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "policy_id": "policy_fortress_001",
    "txn_ref": "settlement_20260223_001",
    "timestamp": 1708704000,
    "context_ref": "amount:50000000,currency:INR"
}
```

#### Response Schema

```protobuf
message AnchorCommitmentResponse {
    bool success = 1;
    string commitment_hash = 2;
    uint64 block_height = 3;     // FlowCortex block height
    string tx_hash = 4;          // L1 transaction hash
    uint64 timestamp = 5;
    optional string error_code = 6;
}
```

**Success Response Example:**
```json
{
    "success": true,
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "block_height": 12345,
    "tx_hash": "txn_00000000000000000000000000000000a1b2c3d4e5f6789012345678",
    "timestamp": 1708704001
}
```

**Error Response Example:**
```json
{
    "success": false,
    "error_code": "INVALID_HASH_FORMAT"
}
```

#### Validation Rules

- `commitment_hash`: Must be exactly 64 hexadecimal characters (SHA256)
- `policy_id`: Non-empty string, max 256 chars
- `txn_ref`: Non-empty string, max 256 chars
- `timestamp`: Unix timestamp (seconds since epoch)

#### Idempotency

Anchoring the same commitment hash multiple times returns the original block height. The second anchor returns:
- `success: true`
- Original `block_height`
- `tx_hash: "idempotent"`

#### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_HASH_FORMAT` | Commitment hash is not 64 hex characters |
| `INVALID_TXN_REF` | Transaction reference is empty or > 256 chars |
| `INVALID_POLICY` | Policy ID is empty |
| `CONFLICT_DETECTED` | Different commitment with same txn_ref already exists |

---

## Proof Verification API

### Verify Proof

Verifies a STARK proof against an anchored commitment.

**Endpoint:** `POST /api/verify_proof`  
**gRPC Method:** `VerifyProof`

#### Request Schema

```protobuf
message VerifyProofRequest {
    string commitment_hash = 1;        // Must reference existing commitment
    string proof_hash = 2;             // SHA256 of proof (64 hex chars)
    bytes proof_data = 3;              // STARK proof bytes
    string proof_type = 4;             // "STARK", "SNARK", etc.
    optional string public_inputs = 5; // Public inputs for verification
    string capsule_version = 6;        // Verifier capsule version
}
```

**JSON Example:**
```json
{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "proof_hash": "b2c3d4e5f6789012345678901234567890abcdef01234567890abcdef012345678",
    "proof_data": "AgQGCAo=",  // Base64 encoded bytes
    "proof_type": "STARK",
    "public_inputs": "user_id=alice,authorization_level=high",
    "capsule_version": "verifier_v1"
}
```

#### Response Schema

```protobuf
message VerifyProofResponse {
    bool success = 1;
    string proof_hash = 2;
    bool verified = 3;               // True if proof is valid
    uint64 verification_block = 4;   // Block height of verification
    optional string error_code = 5;
}
```

**Success Response Example:**
```json
{
    "success": true,
    "proof_hash": "b2c3d4e5f6789012345678901234567890abcdef01234567890abcdef012345678",
    "verified": true,
    "verification_block": 12346
}
```

**Failure Response Example:**
```json
{
    "success": false,
    "verified": false,
    "error_code": "PROOF_INVALID"
}
```

#### Validation Rules

- `commitment_hash`: Must reference an existing anchored commitment
- `proof_hash`: Must be exactly 64 hexadecimal characters
- `proof_data`: Non-empty byte array
- `proof_type`: Non-empty string
- `capsule_version`: Must be a registered verifier capsule version

#### Replay Protection

Submitting the same proof_hash for the same commitment multiple times will be rejected with `PROOF_ALREADY_VERIFIED` error.

#### Error Codes

| Code | Description |
|------|-------------|
| `COMMITMENT_NOT_FOUND` | Referenced commitment does not exist |
| `INVALID_PROOF_HASH` | Proof hash is not 64 hex characters |
| `INVALID_PROOF_FORMAT` | Proof data is empty or malformed |
| `PROOF_INVALID` | STARK proof verification failed |
| `PROOF_ALREADY_VERIFIED` | This proof has already been verified |
| `INVALID_CAPSULE_VERSION` | Capsule version not found |

---

## Query APIs

### Query Commitment

Retrieves an anchored commitment by hash.

**Endpoint:** `GET /api/commitment/{commitment_hash}`  
**gRPC Method:** `QueryCommitment`

#### Request

```json
{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef"
}
```

#### Response

```json
{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "policy_id": "policy_fortress_001",
    "txn_ref": "settlement_20260223_001",
    "timestamp": 1708704000,
    "block_height": 12345,
    "verified": true,
    "context_ref": "amount:50000000,currency:INR"
}
```

Returns `null` or `404` if commitment not found.

---

### Query Proof Status

Retrieves proof verification status for a commitment.

**Endpoint:** `GET /api/proof_status/{commitment_hash}`  
**gRPC Method:** `QueryProofStatus`

#### Request

```json
{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef"
}
```

#### Response

```json
{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "proof": {
        "proof_hash": "b2c3d4e5f6789012345678901234567890abcdef01234567890abcdef012345678",
        "verification_status": "Verified",
        "verification_block": 12346,
        "verifier_capsule_version": "verifier_v1",
        "submitted_at": 1708704010
    },
    "verified": true
}
```

---

### Query Events

Retrieves commitment/proof events with optional filtering.

**Endpoint:** `GET /api/events`  
**gRPC Method:** `QueryEvents`

#### Request Parameters

- `commitment_hash` (optional): Filter by commitment
- `proof_hash` (optional): Filter by proof
- `limit` (default: 100): Maximum results to return
- `offset` (default: 0): Pagination offset

#### Response

```json
{
    "events": [
        {
            "type": "CommitmentAnchored",
            "commitment_hash": "a1b2c3...",
            "policy_id": "policy_fortress_001",
            "txn_ref": "settlement_20260223_001",
            "block_height": 12345,
            "timestamp": 1708704001
        },
        {
            "type": "ProofVerified",
            "commitment_hash": "a1b2c3...",
            "proof_hash": "b2c3d4...",
            "verification_block": 12346,
            "verified_at": 1708704010,
            "verifier_capsule_version": "verifier_v1"
        }
    ],
    "total": 2,
    "limit": 100,
    "offset": 0
}
```

---

## Demo Settlement APIs

### Create Demo Settlement

Creates a new demo settlement scenario.

**Endpoint:** `POST /api/demo/settlement`

#### Request

```json
{
    "from_account": "Bank_A",
    "to_account": "Bank_B",
    "amount_inr": 50000000,
    "reference": "DEMO_SETTLE_001"
}
```

#### Response

```json
{
    "settlement_id": "settle_001",
    "scenario": {
        "total_steps": 8,
        "completed_steps": 0,
        "status": "Created"
    }
}
```

---

### Execute Settlement Step

Executes a single step in the settlement flow.

**Endpoint:** `POST /api/demo/settlement/{settlement_id}/step/{step_number}`

#### Response

```json
{
    "settlement_id": "settle_001",
    "step_number": 1,
    "step_name": "Authorization Request",
    "status": "Completed",
    "result": {
        "commitment_hash": "a1b2c3...",
        "block_height": 12345
    }
}
```

---

## Error Codes

### Global Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_REQUEST` | 400 | Malformed request body |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource conflict |
| `INTERNAL_ERROR` | 500 | Internal server error |

### Commitment-Specific Errors

| Code | Description |
|------|-------------|
| `INVALID_HASH_FORMAT` | Hash must be 64 hex characters |
| `INVALID_TXN_REF` | Transaction reference invalid |
| `INVALID_POLICY` | Policy ID invalid |
| `CONFLICT_DETECTED` | Conflicting commitment |

### Proof-Specific Errors

| Code | Description |
|------|-------------|
| `COMMITMENT_NOT_FOUND` | Commitment must be anchored first |
| `PROOF_INVALID` | Proof verification failed |
| `PROOF_ALREADY_VERIFIED` | Replay attack prevented |
| `INVALID_CAPSULE_VERSION` | Verifier version not found |

---

## Authentication

### API Key Authentication

**Header:** `Authorization: Bearer <api_key>`

Example:
```bash
curl -H "Authorization: Bearer sk_live_abc123..." \
     https://flowcortex.example.com/api/commitment/abc...
```

### Mutual TLS (Production)

Production deployments use mutual TLS for authentication:
- Client certificate validation
- Certificate pinning
- Secure channel encryption

---

## Rate Limits

| Tier | Requests/Second | Burst |
|------|-----------------|-------|
| Development | 10 | 20 |
| Production | 100 | 200 |
| Enterprise | 1000 | 2000 |

Rate limit headers included in responses:
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`

---

## Examples

### Complete Settlement Flow

```bash
# 1. Anchor commitment
curl -X POST https://flowcortex.example.com/api/anchor_commitment \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "policy_id": "policy_001",
    "txn_ref": "settle_001",
    "timestamp": 1708704000
  }'

# 2. Verify proof
curl -X POST https://flowcortex.example.com/api/verify_proof \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef",
    "proof_hash": "b2c3d4e5f6789012345678901234567890abcdef01234567890abcdef012345678",
    "proof_data": "AgQGCAo=",
    "proof_type": "STARK",
    "capsule_version": "verifier_v1"
  }'

# 3. Query status
curl https://flowcortex.example.com/api/proof_status/a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef
```

---

## See Also

- [Data Model Documentation](DATA_MODEL.md)
- [Event Schema Documentation](EVENT_SCHEMA.md)
- [Security Properties](SECURITY_PROPERTIES.md)
- [Operations Guide](OPERATIONS_GUIDE.md)
