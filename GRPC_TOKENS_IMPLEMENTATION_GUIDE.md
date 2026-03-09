# gRPC Implementation Guide for Tokens & Stablecoin System

**Status:** Architecture & Implementation Guidance  
**Date:** February 22, 2026

---

## Current State Analysis

### What You Have
✅ **gRPC Service** (`flowcortex-l1/src/grpc.rs`)
- Running on port 50051
- Tonic framework + Protobuf
- Basic operations: GetBalance, SubmitTx, ListPool, Snapshot

✅ **REST API** (`flowcortex-l1/src/rpc.rs`)
- Running on port 3000 (Axum)
- Explorer uses this for UI
- More endpoints than gRPC

✅ **Proto Definition** (`flowcortex-l1/proto/l1.proto`)
- Already using Protobuf3
- Currently minimal (6 RPC methods)
- Can be extended for tokens

---

## Architecture Recommendation: gRPC-Primary with REST Gateway

### Why gRPC for Treasury Settlement Platform?

| Aspect | REST | gRPC |
|--------|------|------|
| **Performance** | HTTP/1.1 | HTTP/2 binary |
| **Latency** | ~50ms | ~5-10ms |
| **Bandwidth** | JSON text | Protobuf binary |
| **Type Safety** | Manual validation | Compiler enforced |
| **Streaming** | Polling required | Native bidi streaming |
| **Code Gen** | Manual client | Auto-generated |
| **Bank Clients** | Harder | Native libraries |

### Recommended Setup

```
┌────────────────────────────────────────────┐
│  Treasury Settlement Platform (gRPC Client) │
│  (Banks, Admin, Settlement Service)        │
└────────────────┬─────────────────────────┘
                 │ gRPC (primary)
                 │
    ┌────────────▼────────────┐
    │  FlowCortex L1 gRPC      │
    │  Port: 50051             │
    │  - Token Service         │
    │  - Settlement Service    │
    │  - Query Service         │
    └────────────┬────────────┘
                 │
                 │ (optional: REST gateway for browser)
                 │
    ┌────────────▼────────────┐
    │  REST Gateway            │
    │  Port: 3000              │
    │  (gRPC → REST transcoding)
    └──────────────────────────┘
                 │
          ┌──────┴──────┐
          │             │
    Browser UI      Explorer
   (Optional)      (Optional)
```

**Benefits:**
- ✅ Banks use gRPC client libs directly (better UX)
- ✅ Native streaming for audit logs
- ✅ Type-safe token operations
- ✅ High performance for settlement ops
- ✅ REST gateway for legacy/browser clients
- ✅ Single source of truth (proto file)

---

## Proto File Structure

### Option 1: Extend Existing `l1.proto` (Recommended)

Keep everything in one proto file but organize by service.

**Current structure:**
```protobuf
syntax = "proto3";
package l1;

service L1 {
    rpc GetBalance(BalanceRequest) returns (BalanceResponse);
    rpc SubmitTx(TxRequest) returns (TxResponse);
}
```

**After tokens:**
```protobuf
syntax = "proto3";
package l1;

// ============== MESSAGES ==============

// 1. Token messages
message Token {
    string symbol = 1;      // "FLOWER", "USDC"
    string name = 2;
    uint32 decimals = 3;
    uint64 total_supply = 4;
    string creator = 5;
    string token_type = 6;   // "Native", "Stablecoin"
    string status = 7;       // "Active", "Frozen"
}

message TokenMetadata {
    string symbol = 1;
    string name = 2;
    uint32 decimals = 3;
    uint64 total_supply = 4;
    string creator = 5;
    string token_type = 6;
    string status = 7;
    int64 created_at = 8;
    string metadata_json = 9;  // Backing info, collateral, etc.
}

// 2. Token operation messages
message CreateTokenRequest {
    string symbol = 1;
    string name = 2;
    uint32 decimals = 3;
    uint64 initial_supply = 4;
    string token_type = 5;
    string metadata_json = 6;
}

message CreateTokenResponse {
    bool success = 1;
    string symbol = 2;
    string error = 3;
}

message MintRequest {
    string caller = 1;
    string token = 2;
    string to = 3;
    uint64 amount = 4;
}

message BurnRequest {
    string caller = 1;
    string token = 2;
    string from = 3;
    uint64 amount = 4;
}

message BurnResponse {
    bool success = 1;
    uint64 remaining_supply = 2;
    string error = 3;
}

// 3. Settlement messages
message SettlementMintRequest {
    string bank_account = 1;    // "bank-a.institution.com"
    string token = 2;           // "FLOWER"
    uint64 amount = 3;
    string reference = 4;       // Off-chain reference
    string metadata_json = 5;   // Purpose, notes
}

message SettlementTransferRequest {
    string from_account = 1;
    string to_account = 2;
    string token = 3;
    uint64 amount = 4;
    string reference = 5;
    string metadata_json = 6;
}

message SettlementResponse {
    bool success = 1;
    string tx_hash = 2;
    uint64 block_height = 3;
    string from_balance = 4;
    string to_balance = 5;
    string error = 6;
}

message SettlementStatusRequest {
    string reference = 1;
}

message SettlementStatusResponse {
    string status = 1;           // pending, confirmed, failed
    uint64 block_height = 2;
    int64 timestamp = 3;
    string tx_hash = 4;
}

// 4. Query messages
message TokenListRequest {}

message TokenListResponse {
    repeated TokenMetadata tokens = 1;
}

message TokenRequest {
    string symbol = 1;
}

message BankAccountRequest {}

message BankAccountResponse {
    string account_id = 1;
    string bank_name = 2;
    bool is_approved = 3;
}

message BankListResponse {
    repeated BankAccountResponse banks = 1;
}

// 5. Audit/History messages
message TransactionHistoryRequest {
    string account = 1;
    uint64 limit = 2;
    uint64 offset = 3;
}

message TransactionRecord {
    string tx_hash = 1;
    string kind = 2;            // Mint, Burn, Transfer
    string from = 3;
    string to = 4;
    string token = 5;
    uint64 amount = 6;
    uint64 block_height = 7;
    int64 timestamp = 8;
    string status = 9;
}

message TransactionHistoryResponse {
    repeated TransactionRecord transactions = 1;
    uint64 total_count = 2;
}

message TokenHistoryRequest {
    string token = 1;
    uint64 limit = 2;
}

message TokenEvent {
    string event_type = 1;      // Created, Minted, Burned
    string token = 2;
    string account = 3;
    uint64 amount = 4;
    uint64 block_height = 5;
    int64 timestamp = 6;
}

message TokenHistoryResponse {
    repeated TokenEvent events = 1;
}

// ============== SERVICES ==============

service L1 {
    // Existing operations
    rpc GetBalance(BalanceRequest) returns (BalanceResponse);
    rpc SubmitTx(TxRequest) returns (TxResponse);
    rpc ListPool(Empty) returns (PoolResponse);
    rpc ListBlocks(Empty) returns (BlocksResponse);
    rpc Snapshot(Empty) returns (SnapshotResponse);
}

// NEW: Token Service
service Tokens {
    // Token management
    rpc CreateToken(CreateTokenRequest) returns (CreateTokenResponse);
    rpc ListTokens(TokenListRequest) returns (TokenListResponse);
    rpc GetToken(TokenRequest) returns (TokenMetadata);
    
    // Token operations
    rpc Mint(MintRequest) returns (BalanceResponse);
    rpc Burn(BurnRequest) returns (BurnResponse);
    
    // History & audit
    rpc GetTokenHistory(TokenHistoryRequest) returns (TokenHistoryResponse);
    rpc GetTransactionHistory(TransactionHistoryRequest) returns (TransactionHistoryResponse);
}

// NEW: Settlement Service
service Settlement {
    // Settlement operations
    rpc Mint(SettlementMintRequest) returns (SettlementResponse);
    rpc Burn(BurnRequest) returns (SettlementResponse);
    rpc Transfer(SettlementTransferRequest) returns (SettlementResponse);
    
    // Status & queries
    rpc GetStatus(SettlementStatusRequest) returns (SettlementStatusResponse);
    rpc ListBanks(BankAccountRequest) returns (BankListResponse);
    
    // Streaming: Subscribe to settlement events (optional)
    rpc StreamSettlements(BankAccountRequest) returns (stream SettlementStatusResponse);
}

// NEW: Admin Service
service Admin {
    rpc ApproveBank(BankAccountResponse) returns (BankAccountResponse);
    rpc SetDailyLimit(DailyLimitRequest) returns (DailyLimitResponse);
    rpc FreezeToken(FreezeTokenRequest) returns (FreezeTokenResponse);
    rpc UnfreezeToken(FreezeTokenRequest) returns (FreezeTokenResponse);
}
```

### Option 2: Separate Proto Files (Alternative)

Create dedicated proto files:
- `l1.proto` - Core L1 operations
- `tokens.proto` - Token system
- `settlement.proto` - Settlement ops
- `admin.proto` - Admin controls

**Advantages:**
- Clear separation of concerns
- Easier to version independently
- Modular imports

**Disadvantages:**
- More complex build process
- Multiple proto files to maintain

**Recommendation:** Use Option 1 (single extended file) unless you have >100 messages.

---

## Implementation Structure

### File Organization

```
flowcortex-l1/
├── src/
│   ├── grpc.rs              # Main gRPC service handler
│   ├── grpc/
│   │   ├── mod.rs
│   │   ├── tokens.rs        # Token service implementation
│   │   ├── settlement.rs    # Settlement service implementation
│   │   ├── admin.rs         # Admin service implementation
│   │   └── handlers.rs      # Common handlers
│   ├── ledger.rs            # (Updated) Add token registry & methods
│   ├── types.rs             # (Updated) Add token-related types
│   └── main.rs              # (Updated) Initialize services
│
├── proto/
│   └── l1.proto             # (Extended) Add token/settlement messages & services
│
└── build.rs                 # (Updated) Add proto compilation
```

### Step 1: Extend Proto File

Modify `flowcortex-l1/proto/l1.proto` to add:
- Token messages (TokenMetadata, CreateTokenRequest, etc.)
- Settlement messages
- New services (Tokens, Settlement, Admin)

### Step 2: Extend types.rs

Add Rust structs (must match proto messages):

```rust
// Already in types.rs:
pub type Token = String;  // Changed from enum

// Add new:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub creator: AccountId,
    pub token_type: String,     // "Native", "Stablecoin"
    pub status: String,         // "Active", "Frozen"
    pub created_at: u64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Native,
    Stablecoin,
    Governance,
    Utility,
}

// Transaction variants
pub enum TransactionKind {
    // ... existing
    CreateToken { ... },
    Mint { ... },
    Burn { ... },
    FreezeToken { ... },
    SettlementMint { ... },
    SettlementTransfer { ... },
}
```

### Step 3: Create gRPC Service Handlers

Create `flowcortex-l1/src/grpc/tokens.rs`:

```rust
use crate::rpc::SharedNode;
use crate::types::{TokenMetadata, TransactionKind};
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("l1");
}

use proto::tokens_server::Tokens;
use proto::*;

#[derive(Clone)]
pub struct TokensService {
    node: SharedNode,
}

#[tonic::async_trait]
impl Tokens for TokensService {
    async fn create_token(
        &self,
        req: Request<CreateTokenRequest>,
    ) -> Result<Response<CreateTokenResponse>, Status> {
        let req = req.into_inner();
        
        // Create token transaction
        let tx = TransactionKind::CreateToken {
            symbol: req.symbol.clone(),
            name: req.name,
            decimals: req.decimals as u8,
            initial_supply: req.initial_supply,
        };
        
        let mut node = self.node.lock().unwrap();
        match node.execute_transaction(tx) {
            Ok(()) => Ok(Response::new(CreateTokenResponse {
                success: true,
                symbol: req.symbol,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CreateTokenResponse {
                success: false,
                symbol: req.symbol,
                error: e.to_string(),
            })),
        }
    }
    
    async fn mint(
        &self,
        req: Request<MintRequest>,
    ) -> Result<Response<BalanceResponse>, Status> {
        // Implementation
    }
    
    // ... other methods
}
```

### Step 4: Update Main gRPC Server

Update `flowcortex-l1/src/grpc.rs`:

```rust
use tonic::transport::Server;

pub async fn serve_grpc(
    node: SharedNode,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let l1_service = L1Service { node: node.clone() };
    let tokens_service = TokensService { node: node.clone() };
    let settlement_service = SettlementService { node: node.clone() };
    let admin_service = AdminService { node };
    
    Server::builder()
        .add_service(L1Server::new(l1_service))
        .add_service(TokensServer::new(tokens_service))
        .add_service(SettlementServer::new(settlement_service))
        .add_service(AdminServer::new(admin_service))
        .serve(addr)
        .await?;
    
    Ok(())
}
```

---

## gRPC vs REST Comparison

### For Bank Clients (Treasury Settlement Platform)

**Use gRPC:**
```python
# Python gRPC client
from l1_pb2_grpc import SettlementStub
from l1_pb2 import SettlementTransferRequest

stub = SettlementStub(channel)
response = stub.Transfer(SettlementTransferRequest(
    from_account="bank-a.com",
    to_account="bank-b.com",
    token="FLOWER",
    amount=500000000,
    reference="PAY-001"
))

print(f"Confirmed at block {response.block_height}")
```

**vs REST:**
```python
import requests

response = requests.post(
    'http://192.168.29.78:3000/settlement/transfer',
    json={...}
)
data = response.json()
```

**gRPC Advantages:**
- ✅ Type-safe (compile-time checking)
- ✅ Auto-generated client code
- ✅ Faster (~10x)
- ✅ Smaller payloads
- ✅ Built-in streaming
- ✅ Better tooling (grpcurl, etc.)

---

## REST Gateway (Optional)

If browser clients need REST, add gRPC-JSON gateway:

### Setup

```toml
# Cargo.toml
tonic-web = "0.5"
```

### Config

```rust
// in grpc.rs
use tonic_web::enable;

let server = Server::builder()
    .add_service(enable(L1Server::new(l1_service)))
    .add_service(enable(TokensServer::new(tokens_service)))
    // ...
    .serve(addr)
    .await?;
```

### Usage

```bash
# REST → gRPC transcoding
curl -X POST http://192.168.29.78:50051/l1.Tokens/CreateToken \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "FLOWER",
    "name": "Flow Rupee",
    "decimals": 6,
    "initial_supply": 1000000000
  }'
```

**Note:** This is mainly for debugging. Primary clients should use gRPC directly.

---

## Client Libraries

### Auto-Generated Code for Banks

**Python:**
```bash
python -m grpc_tools.protoc -I. --python_out=. --pyi_out=. --grpc_python_out=. proto/l1.proto
```

**Go:**
```bash
protoc --go_out=. --go-grpc_out=. proto/l1.proto
```

**TypeScript:**
```bash
protoc --plugin=protoc-gen-ts=$(which protoc-gen-ts) \
  --ts_out=. proto/l1.proto
```

**Rust (already done):**
- Build script in `build.rs` auto-generates Rust code

---

## Implementation Sequence

### Phase 1: Core Token Service (1-2 weeks)
1. [ ] Extend `l1.proto` with token messages
2. [ ] Add TokenMetadata to types.rs
3. [ ] Implement TokensService in grpc.rs
4. [ ] Add gRPC endpoints:
   - CreateToken
   - Mint
   - Burn
   - ListTokens
   - GetToken
5. [ ] Test with gRPC client
6. [ ] Generate client libs (Python, Go, TS)

### Phase 2: Settlement Service (1-2 weeks)
1. [ ] Add settlement messages to proto
2. [ ] Implement SettlementService
3. [ ] Add bank approval logic
4. [ ] Add daily limits
5. [ ] Test transfer flows
6. [ ] Streaming settlement events

### Phase 3: Admin Service (1 week)
1. [ ] Add admin messages
2. [ ] Implement AdminService
3. [ ] Bank approve/freeze
4. [ ] Token freeze/unfreeze
5. [ ] Daily limit management

### Phase 4: REST Gateway (Optional, 1 week)
1. [ ] Add tonic-web
2. [ ] Test REST transcoding
3. [ ] Update Explorer to use gRPC or REST

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_token() {
        let node = Arc::new(Mutex::new(Node::new("admin".into())));
        let service = TokensService { node };
        
        let req = Request::new(CreateTokenRequest {
            symbol: "FLOWER".into(),
            name: "Flow Rupee".into(),
            decimals: 6,
            initial_supply: 1_000_000_000,
            token_type: "Stablecoin".into(),
            metadata_json: "{}".into(),
        });
        
        let response = service.create_token(req).await.unwrap();
        assert!(response.into_inner().success);
    }
}
```

### Integration Tests
```bash
# Use grpcurl to test endpoints
grpcurl -plaintext -d '{"symbol":"FLOWER",...}' \
  192.168.29.78:50051 l1.Tokens/CreateToken

# Use tonic test client
#[tokio::test]
async fn test_settlement_transfer() {
    // Full end-to-end flow
}
```

### Load Testing
```bash
# ghz for gRPC load testing
ghz --insecure \
  --proto ./proto/l1.proto \
  --call l1.Settlement/Transfer \
  -d @ \
  -c 100 \
  -n 1000 \
  192.168.29.78:50051
```

---

## Performance Characteristics

### Expected Latency

| Operation | gRPC | REST |
|-----------|------|------|
| CreateToken | 5-15ms | 50-100ms |
| Transfer | 10-20ms | 100-150ms |
| GetBalance | 1-5ms | 20-50ms |
| ListTokens | 2-10ms | 30-80ms |

### Throughput

- gRPC: **10,000+ transfers/sec**
- REST API: **1,000-2,000 transfers/sec**

---

## Security Considerations

### Authentication
```protobuf
message Request {
    string caller = 1;
    bytes signature = 2;    // Ed25519 signature
    string nonce = 3;       // Prevent replay
}
```

### Authorization
```rust
// Check admin only
fn authorize_admin(caller: &str) -> Result<(), Status> {
    if caller != "admin" {
        return Err(Status::permission_denied("admin only"));
    }
    Ok(())
}
```

### Rate Limiting
```rust
// Per-bank rate limiter
fn check_rate_limit(bank: &str) -> Result<(), Status> {
    // Check transfers/sec
    // Check amount/day
}
```

---

## Migration from REST

### If Currently Using REST

1. **Keep both running** during transition
2. **Redirect Explorer** to use gRPC or REST gateway
3. **Update bank clients** to use gRPC gradually
4. **Deprecate REST** after 3-6 months

### Backward Compatibility

```rust
// Support both
pub async fn handle_rpc(req: JsonRequest) -> JsonResponse {
    // Convert JSON to gRPC, call service, convert back
}
```

---

## Monitoring & Observability

### gRPC Metrics to Track

```rust
// Using opentelemetry
- request_count (per RPC)
- request_duration (latency)
- error_rate (by error type)
- settlement_volume (amount/token)
- active_streams (for streaming endpoints)
```

### Logging

```rust
info!("Token created: {} by {}", symbol, creator);
warn!("Transfer failed: {} → {}, reason: {}", from, to, error);
error!("Bank freeze requested: {}", bank);
```

---

## Summary: Why gRPC is Better for Your Use Case

✅ **Performance** - 10x faster than REST  
✅ **Type Safety** - Protobuf compiler catches errors  
✅ **Streaming** - Native support for audit logs  
✅ **Code Generation** - Auto client libs for banks  
✅ **Bandwidth** - Binary protocol (better for banks)  
✅ **Built-in Tools** - grpcurl, ghz, etc.  
✅ **Production Ready** - Used by Google, Netflix, Uber  
✅ **Future Proof** - Easy to add new services  

---

## Recommended Implementation Order

1. **Extend `l1.proto`** with token/settlement messages
2. **Add TokensService** to gRPC (minimal RPC endpoints)
3. **Test with grpcurl** (simple curl-like tool for gRPC)
4. **Add SettlementService** with bank logic
5. **Generate client libraries** for Treasury Platform
6. **Add Admin service** for controls
7. **Optional: REST gateway** if needed for Explorer

---

## Example: First Token Creation via gRPC

### 1. Proto Definition
```protobuf
service Tokens {
    rpc CreateToken(CreateTokenRequest) returns (CreateTokenResponse);
}
```

### 2. Rust Handler
```rust
async fn create_token(&self, req: Request<CreateTokenRequest>) -> Result<...> {
    let req = req.into_inner();
    // Create token in ledger
    Ok(Response::new(CreateTokenResponse { success: true, ... }))
}
```

### 3. Test with grpcurl
```bash
grpcurl -plaintext -d '{
  "symbol": "FLOWER",
  "name": "Flow Rupee",
  "decimals": 6,
  "initial_supply": 1000000000
}' 192.168.29.78:50051 l1.Tokens/CreateToken
```

### 4. Result
```json
{
  "success": true,
  "symbol": "FLOWER"
}
```

---

## Questions to Answer

1. **Multi-collateral stablecoins?** → Use metadata JSON field
2. **Audit streaming?** → Add `rpc StreamTokenEvents` with server-push
3. **Rate limits?** → Implement at gRPC middleware level
4. **Signatures required?** → Add caller + signature to all requests
5. **REST gateway needed?** → Probably not (banks use gRPC native)

---

## File Changes Summary

```
Files to Create:
- flowcortex-l1/src/grpc/tokens.rs       (TokensService implementation)
- flowcortex-l1/src/grpc/settlement.rs   (SettlementService implementation)
- flowcortex-l1/src/grpc/admin.rs        (AdminService implementation)

Files to Modify:
- flowcortex-l1/proto/l1.proto           (Extend with token/settlement messages)
- flowcortex-l1/src/types.rs             (Add TokenMetadata, TokenEvent)
- flowcortex-l1/src/ledger.rs            (Add token registry & methods)
- flowcortex-l1/src/grpc.rs              (Register new services)
- flowcortex-l1/src/main.rs              (Initialize gRPC services)

Files to Keep:
- flowcortex-l1/src/rpc.rs               (Optional: REST fallback)
```

---

**Ready to proceed with Proto file extension?**
