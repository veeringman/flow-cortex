# FlowCortex L1 cURL Examples

Quick reference for testing FlowCortex L1 REST API using `curl`.

## Quick Start

```bash
# Make the script executable
chmod +x run-examples.sh

# Run all examples
./run-examples.sh

# Run with custom base URL
BASE_URL=http://your-node:3000 ./run-examples.sh
```

## Individual Examples

### 1. Create Account

```bash
curl -X POST http://127.0.0.1:3000/account \
  -H "Content-Type: application/json" \
  -d '{"account":"alice"}'
```

### 2. Get Balance

```bash
# Proof token
curl -X GET http://127.0.0.1:3000/balance/admin/Proof

# FloweR token
curl -X GET http://127.0.0.1:3000/balance/admin/FloweR
```

**Response:**
```json
{
  "account": "admin",
  "token": "Proof",
  "balance": 1000000
}
```

### 3. Mint Tokens

```bash
curl -X POST http://127.0.0.1:3000/mint \
  -H "Content-Type: application/json" \
  -d '{
    "caller": "admin",
    "to": "alice",
    "token": "Proof",
    "amount": 1000
  }'
```

### 4. Transfer Tokens

```bash
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{
    "from": "alice",
    "to": "bob",
    "token": "Proof",
    "amount": 100
  }'
```

### 5. Get Pending Transaction Pool

```bash
curl -X GET http://127.0.0.1:3000/pool
```

**Response:**
```json
{
  "pending": [...]
}
```

### 6. List Blocks

```bash
curl -X GET http://127.0.0.1:3000/blocks
```

**Response:**
```json
[
  {
    "height": 0,
    "transactions": [...]
  },
  {
    "height": 1,
    "transactions": [...]
  }
]
```

### 7. Create Block

```bash
curl -X POST http://127.0.0.1:3000/block
```

**Response:**
```json
{
  "height": 5,
  "transactions": [...]
}
```

### 8. Get Snapshot

```bash
curl -X GET http://127.0.0.1:3000/snapshot
```

**Response:**
```json
{
  "root": "a1b2c3d4e5f6..."
}
```

### 9. List Anchors

```bash
curl -X GET http://127.0.0.1:3000/anchors
```

**Response:**
```json
{
  "anchors": ["anchor_id_1", "anchor_id_2", ...]
}
```

### 10. Get Specific Anchor

```bash
curl -X GET http://127.0.0.1:3000/anchor/anchor_id_1
```

**Response:**
```json
{
  "id": "anchor_id_1",
  "proof": "base64_encoded_proof_data"
}
```

### 11. Upload Capsule

```bash
# Create base64 encoded code
CAPSULE_CODE=$(echo -n "sample wasm code" | base64)

curl -X POST http://127.0.0.1:3000/capsule \
  -H "Content-Type: application/json" \
  -d "{
    \"id\": \"my_capsule\",
    \"code\": \"$CAPSULE_CODE\"
  }"
```

**Response:**
```json
{
  "success": true
}
```

### 12. List Capsules

```bash
curl -X GET http://127.0.0.1:3000/capsule
```

**Response:**
```json
{
  "capsules": ["my_capsule", "another_capsule", ...]
}
```

### 13. Invoke Capsule

```bash
# Create base64 encoded input
INPUT=$(echo -n "test input" | base64)

curl -X POST http://127.0.0.1:3000/capsule/my_capsule/invoke \
  -H "Content-Type: application/json" \
  -d "{
    \"input\": \"$INPUT\"
  }"
```

**Response:**
```json
{
  "output": "base64_encoded_output"
}
```

### 14. Submit Signed Transaction

```bash
# Create base64 encoded pubkey and signature
PUBKEY=$(echo -n "public_key_bytes" | base64)
SIGNATURE=$(echo -n "signature_bytes" | base64)

curl -X POST http://127.0.0.1:3000/tx \
  -H "Content-Type: application/json" \
  -d "{
    \"caller\": \"alice\",
    \"pubkey\": \"$PUBKEY\",
    \"signature\": \"$SIGNATURE\",
    \"tx\": {}
  }"
```

## Tips

### Pretty Print JSON

```bash
curl -s http://127.0.0.1:3000/snapshot | python3 -m json.tool
```

### Save Response to File

```bash
curl -X GET http://127.0.0.1:3000/blocks > blocks.json
```

### Check HTTP Status

```bash
curl -w "HTTP Status: %{http_code}\n" -X GET http://127.0.0.1:3000/blocks
```

### Add Request Headers

```bash
curl -H "Authorization: Bearer token" \
     -H "X-Custom-Header: value" \
     http://127.0.0.1:3000/blocks
```

### Follow Redirects

```bash
curl -L http://127.0.0.1:3000/snapshot
```

## Environment Variables

Set the base URL for all requests:

```bash
export BASE_URL=http://your-node:3000

curl -X GET $BASE_URL/blocks
```

## Error Handling

### Bad Request (400)

```json
{
  "error": "unknown token `InvalidToken`"
}
```

### Not Found (404)

```json
{
  "error": "not found"
}
```

### Server Error (500)

Check server logs for details.

## Batch Operations

```bash
#!/bin/bash

BASE_URL="http://127.0.0.1:3000"

# Create 10 accounts
for i in {1..10}; do
  curl -X POST "$BASE_URL/account" \
    -H "Content-Type: application/json" \
    -d "{\"account\":\"user_$i\"}"
done

# Mint tokens to all
for i in {1..10}; do
  curl -X POST "$BASE_URL/mint" \
    -H "Content-Type: application/json" \
    -d "{\"caller\":\"admin\",\"to\":\"user_$i\",\"token\":\"Proof\",\"amount\":1000}"
done
```
