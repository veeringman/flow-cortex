# FlowCortex L1 Integration Guide

Complete guide for integrating applications with FlowCortex L1 node.

## Overview

FlowCortex L1 provides two main interfaces for integration:

1. **REST API** - HTTP-based API on port `3000`
2. **gRPC API** - High-performance gRPC service on port `50051`

Both interfaces provide the same core functionality:
- Account and balance management
- Token transfers and minting
- Block creation and querying
- Capsule (wasm/bytecode) management
- Anchor and proof management
- Transaction submission

## Choosing an Interface

### Use REST API if:
- You prefer simple HTTP requests
- Integration with web applications
- Quick curl-based testing
- Language without gRPC support
- Lower performance requirements

### Use gRPC if:
- High performance is critical
- Binary protocol efficiency needed
- Streaming support desired
- Real-time data synchronization
- Low-latency requirements

## Quick Start

### 1. Start FlowCortex L1 Node

```bash
cd flowcortex-l1
cargo run --release
```

The node will start listening on:
- REST API: `http://127.0.0.1:3000`
- gRPC: `http://127.0.0.1:50051`

### 2. Basic Transaction Flow

#### Using JavaScript/TypeScript

```typescript
import { FlowCortexL1Client } from './examples/l1-integration-clients/typescript/src/client';

const client = new FlowCortexL1Client('http://127.0.0.1:3000');

// Create account
await client.createAccount('alice');

// Check balance
const balance = await client.getBalance('alice', 'Proof');
console.log(`Balance: ${balance.balance}`);

// Mint tokens
await client.mint({
  caller: 'admin',
  to: 'alice',
  token: 'Proof',
  amount: 1000,
});

// Transfer tokens
await client.transfer({
  from: 'alice',
  to: 'bob',
  token: 'Proof',
  amount: 100,
});
```

#### Using Python

```python
from client import FlowCortexL1Client

client = FlowCortexL1Client('http://127.0.0.1:3000')

# Create account
client.create_account('alice')

# Check balance
balance = client.get_balance('alice', 'Proof')
print(f"Balance: {balance.balance}")

# Mint tokens
client.mint(caller='admin', to='alice', token='Proof', amount=1000)

# Transfer tokens
client.transfer(from_account='alice', to='bob', token='Proof', amount=100)
```

#### Using Rust (gRPC)

```rust
use tonic::transport::Channel;

let channel = Channel::from_static("http://127.0.0.1:50051")
    .connect()
    .await?;
let mut client = L1Client::new(channel);

let response = client.get_balance(tonic::Request::new(BalanceRequest {
    account: "alice".to_string(),
    token: "Proof".to_string(),
})).await?;

println!("Balance: {}", response.into_inner().balance);
```

#### Using cURL

```bash
# Create account
curl -X POST http://127.0.0.1:3000/account \
  -H "Content-Type: application/json" \
  -d '{"account":"alice"}'

# Get balance
curl -X GET http://127.0.0.1:3000/balance/alice/Proof

# Transfer tokens
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{
    "from":"alice",
    "to":"bob",
    "token":"Proof",
    "amount":100
  }'
```

## Detailed API Reference

### Core Concepts

#### Accounts
- Identified by `account_id` (string)
- Hold balances in different token types
- Default accounts: `admin`, `alice`, `bob`

#### Tokens
- **Proof**: Primary token for the network
- **FloweR**: Secondary token

#### Transactions
- Submitted via `/transfer` (token transfers)
- Minted via `/mint` (token creation by admin)
- Generic submission via `/tx` (signed transactions)

#### Blocks
- Created automatically every 5 seconds
- Can be manually created via `/block`
- Contain transaction history

#### Capsules
- Smart contract-like bytecode (WASM)
- Uploaded as base64-encoded binary
- Can be invoked with input data

### REST API Endpoints

#### Account Management

**Create Account**
```
POST /account
Content-Type: application/json

{
  "account": "alice"
}
```

**Get Balance**
```
GET /balance/{account}/{token}
```
Parameters:
- `account`: Account ID
- `token`: "Proof" or "FloweR"

Response:
```json
{
  "account": "alice",
  "token": "Proof",
  "balance": 1000
}
```

#### Token Operations

**Mint Tokens**
```
POST /mint
Content-Type: application/json

{
  "caller": "admin",
  "to": "alice",
  "token": "Proof",
  "amount": 1000,
  "rw_set": {},          // Optional
  "proof": {}            // Optional
}
```

**Transfer Tokens**
```
POST /transfer
Content-Type: application/json

{
  "from": "alice",
  "to": "bob",
  "token": "Proof",
  "amount": 100,
  "rw_set": {},          // Optional
  "proof": {}            // Optional
}
```

**Submit Signed Transaction**
```
POST /tx
Content-Type: application/json

{
  "caller": "alice",
  "pubkey": "base64_encoded_pubkey",
  "signature": "base64_encoded_signature",
  "tx": {}
}
```

#### Block Management

**Create Block**
```
POST /block
```

**List Blocks**
```
GET /blocks
```

Response:
```json
[
  {
    "height": 0,
    "transactions": [...]
  }
]
```

**Get Snapshot**
```
GET /snapshot
```

Response:
```json
{
  "root": "hex_encoded_state_root"
}
```

**Get Pool**
```
GET /pool
```

Response:
```json
{
  "pending": [...]
}
```

#### Capsule Management

**Upload Capsule**
```
POST /capsule
Content-Type: application/json

{
  "id": "my_capsule",
  "code": "base64_encoded_wasm"
}
```

**List Capsules**
```
GET /capsule
```

Response:
```json
{
  "capsules": ["capsule_1", "capsule_2"]
}
```

**Invoke Capsule**
```
POST /capsule/{capsule_id}/invoke
Content-Type: application/json

{
  "input": "base64_encoded_input"
}
```

Response:
```json
{
  "output": "base64_encoded_output"
}
```

#### Anchor Management

**List Anchors**
```
GET /anchors
```

Response:
```json
{
  "anchors": ["anchor_1", "anchor_2"]
}
```

**Get Anchor**
```
GET /anchor/{anchor_id}
```

Response:
```json
{
  "id": "anchor_1",
  "proof": "base64_encoded_proof"
}
```

### gRPC API

All gRPC services use the protobuf definitions in `flowcortex-l1/proto/l1.proto`.

**Service: L1**

Methods:
- `GetBalance(BalanceRequest) -> BalanceResponse`
- `SubmitTx(TxRequest) -> TxResponse`
- `ListPool(Empty) -> PoolResponse`
- `ListBlocks(Empty) -> BlocksResponse`
- `Snapshot(Empty) -> SnapshotResponse`
- `CreateBlock(Empty) -> Block`
- `ListAnchors(Empty) -> AnchorListResponse`
- `GetAnchor(AnchorRequest) -> AnchorResponse`
- `UploadCapsule(CapsuleUploadRequest) -> CapsuleUploadResponse`
- `ListCapsules(Empty) -> CapsuleListResponse`
- `InvokeCapsule(CapsuleInvokeRequest) -> CapsuleInvokeResponse`

## Example Scenarios

### Scenario 1: Multi-User Payment System

```typescript
const client = new FlowCortexL1Client();

// Setup users
await client.createAccount('user_1');
await client.createAccount('user_2');

// Admin mints initial tokens
await client.mint({
  caller: 'admin',
  to: 'user_1',
  token: 'Proof',
  amount: 500,
});

// User 1 transfers to User 2
await client.transfer({
  from: 'user_1',
  to: 'user_2',
  token: 'Proof',
  amount: 100,
});

// Verify balances
const bal1 = await client.getBalance('user_1', 'Proof');
const bal2 = await client.getBalance('user_2', 'Proof');
console.log(`User 1: ${bal1.balance}, User 2: ${bal2.balance}`);
```

### Scenario 2: Smart Contract Deployment

```typescript
// Load WASM bytecode
const fs = require('fs');
const wasmCode = fs.readFileSync('contract.wasm');
const codeBase64 = Buffer.from(wasmCode).toString('base64');

// Upload contract as capsule
await client.uploadCapsule('my_contract', codeBase64);

// Invoke contract
const inputBase64 = Buffer.from(JSON.stringify({
  method: 'transfer',
  params: { to: 'alice', amount: 100 }
})).toString('base64');

const output = await client.invokeCapsule('my_contract', inputBase64);
console.log('Contract output:', output.output);
```

### Scenario 3: State Verification

```python
# Get current state snapshot
snapshot_data = client.get_snapshot()
print(f"Current state root: {snapshot_data['root']}")

# List all blocks for audit
blocks = client.list_blocks()
for block in blocks:
    print(f"Block {block.height}: {len(block.transactions)} transactions")

# Get pending transactions
pool = client.get_pool()
print(f"Pending transactions: {pool}")
```

## Error Handling

### REST API Errors

All errors return JSON with an `error` field:

```json
{
  "error": "unknown token `InvalidToken`"
}
```

HTTP Status Codes:
- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid input
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

### gRPC Errors

Handled via `tonic::Status`:

```rust
match response {
    Ok(data) => { /* handle success */ },
    Err(status) => {
        eprintln!("Error: {} - {}", status.code(), status.message());
    }
}
```

## Performance Considerations

### REST API
- Suitable for: Web applications, quick integrations
- Overhead: HTTP framing, JSON serialization
- Latency: ~10-100ms per request

### gRPC
- Suitable for: High-throughput, real-time systems
- Overhead: Minimal binary protocol
- Latency: ~1-10ms per request
- Supports streaming for bulk operations

## Security

### Authentication
Currently, FlowCortex L1 does not enforce authentication. For production:
- Implement API keys or JWT tokens
- Use HTTPS/TLS for gRPC encryption
- Validate all inputs on client side

### Data Validation
- Always validate account IDs
- Check token names before submission
- Verify base64 encoding for binary data
- Validate amount values (must be > 0)

## Running Examples

### Quick Test with cURL

```bash
cd examples/l1-integration-clients/curl
./run-examples.sh
```

### TypeScript/Node.js

```bash
cd examples/l1-integration-clients/typescript
npm install
npm run build
npm run example:node
```

### Browser Client

```bash
cd examples/l1-integration-clients/typescript
npm run example:browser
# Open http://192.168.29.78:8080/src/examples/browser-example.html
```

### Python

```bash
cd examples/l1-integration-clients/python
pip install -r requirements.txt
python3 example.py
```

### Rust gRPC

```bash
cd examples/l1-integration-clients/rust-grpc
cargo run --release
```

## Troubleshooting

### Connection Refused
- Verify FlowCortex L1 is running
- Check if correct port is configured (3000 for REST, 50051 for gRPC)
- Check firewall settings

### Invalid Token Error
- Valid tokens: "Proof", "FloweR" (case-insensitive in REST)
- Check spelling in requests

### Account Not Found
- Create account first with `POST /account`
- Default accounts: admin, alice, bob

### Base64 Encoding Issues
- Use base64 encoding for all binary data (pubkey, signature, code, input)
- For Python: `base64.b64encode(data).decode('utf-8')`
- For Node.js: `Buffer.from(data).toString('base64')`

## Additional Resources

- [FlowCortex L1 Source Code](../../flowcortex-l1)
- [gRPC Protocol Buffer Definition](../../flowcortex-l1/proto/l1.proto)
- [Rust Client Example](./rust-grpc)
- [TypeScript Client Example](./typescript)
- [Python Client Example](./python)
- [cURL Examples](./curl)

## Support

For issues or questions:
1. Check example code for your language
2. Enable debug logging in your client
3. Check FlowCortex L1 node logs
4. Review API documentation in this guide
