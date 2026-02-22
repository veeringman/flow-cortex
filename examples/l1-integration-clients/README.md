# FlowCortex L1 Client Examples

Collection of sample client implementations for integrating with FlowCortex L1 node.

## Directory Structure

```
l1-integration-clients/
├── INTEGRATION_GUIDE.md          # Comprehensive integration guide
├── rust-grpc/                    # Rust gRPC client example
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/main.rs
│   ├── proto/l1.proto
│   └── README.md
├── typescript/                   # TypeScript/JavaScript REST client
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   │   ├── client.ts
│   │   └── examples/
│   │       ├── node-example.ts
│   │       └── browser-example.html
│   └── README.md
├── python/                       # Python REST and gRPC clients
│   ├── client.py                 # REST client
│   ├── grpc_client.py            # gRPC client
│   ├── example.py                # REST example
│   ├── l1.proto
│   ├── requirements.txt
│   └── README.md
└── curl/                         # cURL examples
    ├── run-examples.sh
    └── README.md
```

## Quick Start

### Choose Your Language

- **TypeScript/JavaScript**: [typescript/README.md](./typescript/README.md)
- **Python**: [python/README.md](./python/README.md)
- **Rust**: [rust-grpc/README.md](./rust-grpc/README.md)
- **cURL**: [curl/README.md](./curl/README.md)

### Running Examples

#### TypeScript
```bash
cd typescript
npm install
npm run example:node
```

#### Python
```bash
cd python
pip install -r requirements.txt
python3 example.py
```

#### Rust
```bash
cd rust-grpc
cargo run --release
```

#### cURL
```bash
cd curl
./run-examples.sh
```

## API Overview

FlowCortex L1 provides two interfaces:

### REST API (Port 3000)
HTTP-based API with JSON request/response format. Ideal for web applications and quick testing.

**Base URL**: `http://127.0.0.1:3000`

Main endpoints:
- `POST /account` - Create account
- `GET /balance/{account}/{token}` - Check balance
- `POST /mint` - Mint tokens
- `POST /transfer` - Transfer tokens
- `GET /blocks` - List blocks
- `POST /block` - Create block
- `POST /capsule` - Upload capsule
- `GET /capsule` - List capsules
- `POST /capsule/{id}/invoke` - Invoke capsule
- `GET /anchors` - List anchors
- `GET /anchor/{id}` - Get anchor

### gRPC API (Port 50051)
High-performance binary protocol. Best for real-time systems and high-throughput applications.

**Service**: `l1.L1`

Main methods:
- `GetBalance` - Query balance
- `ListBlocks` - Retrieve blocks
- `UploadCapsule` - Deploy code
- `InvokeCapsule` - Execute code
- `GetAnchor` - Retrieve proof
- And more...

## Supported Operations

### Account Management
- Create accounts
- Query balances for Proof and FloweR tokens

### Token Operations
- Mint tokens (admin only)
- Transfer between accounts
- Submit signed transactions

### Smart Contracts (Capsules)
- Upload WASM/bytecode
- List uploaded capsules
- Invoke with input data

### Block Management
- Create blocks
- List all blocks
- Get state snapshot

### Proof Management
- List anchors
- Retrieve anchor proofs

## Integration Scenarios

1. **Payment System**: Account creation, minting, transfers
2. **NFT Platform**: Capsule-based token contracts
3. **State Management**: Block querying, snapshots, verification
4. **Proof Systems**: Anchor and verification queries

## Complete Integration Guide

See [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md) for:
- Detailed API reference
- Error handling
- Performance considerations
- Security best practices
- Troubleshooting

## Example Usage

### Check Account Balance

**TypeScript**
```typescript
import { FlowCortexL1Client } from './typescript/src/client';
const client = new FlowCortexL1Client();
const balance = await client.getBalance('alice', 'Proof');
console.log(balance.balance);
```

**Python**
```python
from python.client import FlowCortexL1Client
client = FlowCortexL1Client()
balance = client.get_balance('alice', 'Proof')
print(balance.balance)
```

**Rust (gRPC)**
```rust
let balance = client.get_balance(
    tonic::Request::new(BalanceRequest {
        account: "alice".to_string(),
        token: "Proof".to_string(),
    })
).await?;
```

**cURL**
```bash
curl http://127.0.0.1:3000/balance/alice/Proof
```

### Transfer Tokens

**TypeScript**
```typescript
await client.transfer({
    from: 'alice',
    to: 'bob',
    token: 'Proof',
    amount: 100,
});
```

**Python**
```python
client.transfer(
    from_account='alice',
    to='bob',
    token='Proof',
    amount=100,
)
```

**cURL**
```bash
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{"from":"alice","to":"bob","token":"Proof","amount":100}'
```

## Prerequisites

- FlowCortex L1 node running on `127.0.0.1:3000` (REST) and `127.0.0.1:50051` (gRPC)
- Language-specific tools:
  - **TypeScript**: Node.js 16+
  - **Python**: Python 3.7+
  - **Rust**: Rust 1.56+
  - **cURL**: Standard Unix tools

## Configuration

Default node URL: `http://127.0.0.1:3000` (REST) / `http://127.0.0.1:50051` (gRPC)

Override in your client:

**TypeScript**
```typescript
const client = new FlowCortexL1Client('http://your-node:3000');
```

**Python**
```python
client = FlowCortexL1Client('http://your-node:3000')
```

**Rust**
```rust
let channel = Channel::from_static("http://your-node:50051").connect().await?;
```

## Contributing

To add new examples:
1. Create a new directory in `l1-integration-clients/`
2. Add a `README.md` with setup and usage instructions
3. Include complete working examples
4. Document all API interactions

## License

Same as FlowCortex project - See LICENSE file in root
