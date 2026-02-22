# FlowCortex Tokens Feature - Architecture & Implementation Plan

## Current State

### Token System Architecture
- **Location:** `flowcortex-l1/src/types.rs`
- **Current Tokens:** Hardcoded enum with 2 tokens
  - `Proof` - Native token (proof-of-stake)
  - `FloweR` - Stablecoin
- **Token Storage:** `HashMap<AccountId, HashMap<Token, u64>>`
- **Operations:** Mint (admin only), Transfer, Balance query

### Current Enum Definition
```rust
pub enum Token {
    Proof,
    FloweR,
}
```

**Problem:** Tokens are hardcoded → can't create new token types dynamically

## Proposed Solution: Multi-Phase Approach

### Phase 1: ✅ Simple Extensible Token System (NO Capsules Required)
**Goal:** Allow dynamic token creation without smart contract logic

**Changes:**
1. **Change Token from enum to String**
   - Replace `enum Token` with `type Token = String`
   - Enables unlimited token types
   - Keep backward compatibility with "proof" and "flower"

2. **Add Token Registry**
   ```rust
   pub struct TokenMetadata {
       pub name: String,
       pub symbol: String,
       pub decimals: u8,
       pub total_supply: u64,
       pub creator: AccountId,
       pub created_at: u64,
   }
   
   pub struct Ledger {
       pub balances: HashMap<AccountId, HashMap<Token, u64>>,
       pub tokens: HashMap<Token, TokenMetadata>, // NEW
       pub admin: AccountId,
   }
   ```

3. **New RPC Endpoints**
   - `POST /token/create` - Create new token
   - `GET /tokens` - List all tokens
   - `GET /token/:name` - Get token metadata

4. **Token Creation Request**
   ```json
   {
     "name": "Stable Dollar",
     "symbol": "USDC",
     "decimals": 6,
     "initial_supply": 1000000000,
     "creator": "admin"
   }
   ```

**Benefits:**
- ✅ No capsules needed
- ✅ Simple implementation
- ✅ Backward compatible
- ✅ Extensible
- ✅ Persistent metadata

**NOT a smart contract** - just basic token properties

---

### Phase 2: Token Registry & Management (Optional Enhancement)
**Goal:** Better token organization and discovery

**Features:**
- Token whitelist/approval system
- Token freeze/unfreeze (admin control)
- Token burning (reducing supply)
- Per-token transaction fees
- Transfer caps/limits

**Requires:** Minor ledger extensions (no capsules)

---

### Phase 3: Smart Tokens via Capsules (OPTIONAL - Complex)
**Goal:** Allow custom token logic via smart contracts

**Use Cases:**
- StableCoin with rebase logic
- Yield-bearing tokens
- Governance tokens with voting
- Deflationary tokens with burn mechanics
- Custom mint/burn rules

**How It Works:**
```
Token Creation
    ↓
Deploy Capsule with token logic
    ↓
Register token pointing to capsule
    ↓
On Transfer → Call capsule to validate/execute custom logic
    ↓
Ledger updates balances
```

**Example StableCoin Capsule:**
```rust
// Mint request → capsule validates supply
// Transfer request → capsule checks rate limit
// Burn request → capsule updates supply
```

**Trade-offs:**
- ✅ Powerful - any token logic possible
- ✅ Verifiable - logic on chain
- ❌ More complex
- ❌ Performance overhead
- ❌ Security considerations

---

## Recommended Implementation Path

### ✅ **Phase 1 (RECOMMENDED - Start Here)**
**Effort:** 2-3 hours
**Complexity:** Low  
**Deliverable:** Dynamic token creation without smart contracts

#### Files to Modify

1. **`flowcortex-l1/src/types.rs`**
   ```rust
   // Change: enum Token → String
   pub type Token = String;
   
   // Add: Token metadata
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct TokenMetadata {
       pub name: String,
       pub symbol: String,
       pub decimals: u8,
       pub total_supply: u64,
       pub creator: AccountId,
   }
   
   // Add: Token creation transaction
   pub enum TransactionKind {
       // ... existing variants
       CreateToken {
           symbol: String,
           name: String,
           decimals: u8,
           initial_supply: u64,
       },
   }
   ```

2. **`flowcortex-l1/src/ledger.rs`**
   ```rust
   pub struct Ledger {
       pub balances: HashMap<AccountId, HashMap<Token, u64>>,
       pub tokens: HashMap<Token, TokenMetadata>, // NEW registry
       // ...
   }
   
   impl Ledger {
       pub fn create_token(&mut self, ...) -> Result<(), LedgerError> {
           // Validate token doesn't exist
           // Create metadata
           // Mint initial supply
       }
       
       pub fn get_token_metadata(&self, token: &Token) -> Option<&TokenMetadata> {
           // ...
       }
   }
   ```

3. **`flowcortex-l1/src/rpc.rs`**
   ```rust
   // New endpoints:
   async fn create_token(
       Extension(node): Extension<SharedNode>,
       Json(payload): Json<CreateTokenRequest>,
   ) -> impl IntoResponse {
       // Handle token creation
   }
   
   async fn list_tokens(
       Extension(node): Extension<SharedNode>,
   ) -> impl IntoResponse {
       // Return all tokens with metadata
   }
   ```

4. **`explorer/static/js/modules/api.js`**
   ```javascript
   export const TokenAPI = {
       async createToken(symbol, name, decimals, supply) {
           return apiCall('/token/create', {
               method: 'POST',
               body: JSON.stringify({
                   symbol, name, decimals,
                   initial_supply: supply,
                   creator: 'admin'
               })
           });
       },
       
       async listTokens() {
           return apiCall('/tokens');
       },
       
       async getToken(symbol) {
           return apiCall(`/token/${symbol}`);
       }
   };
   ```

5. **`explorer/templates/index.html`**
   - Add "Tokens" tab to UI
   - Forms for creating tokens
   - Token listing/management UI

---

## Explorer UI Components

### New "Tokens" Tab Features:

#### 1. Token Creation Panel
```
┌─────────────────────────────────────┐
│ Create New Token                    │
├─────────────────────────────────────┤
│                                     │
│ Token Symbol    [_______]           │
│ Token Name      [_______]           │
│ Decimals        [_]                 │
│ Initial Supply  [__________]        │
│                                     │
│ [Cancel]  [Create Token]            │
└─────────────────────────────────────┘
```

#### 2. Token List
```
┌─────────────────────────────────────────┐
│ Available Tokens                        │
├─────────────────────────────────────────┤
│ Symbol │ Name        │ Supply │ Created │
├─────────────────────────────────────────┤
│ PROOF  │ Proof Token │ ∞      │ System  │
│ FLOWER │ FloweCoin   │ 1M     │ System  │
│ USDC   │ Stable USD  │ 1B     │ admin   │
│ XFLOW  │ Flow Token  │ 10M    │ alice   │
└─────────────────────────────────────────┘
```

#### 3. Token Operations
- View token metadata
- Mint additional tokens (admin)
- Check total supply
- View holders
- Transfer between accounts

---

## Implementation Steps

### Phase 1 Implementation Checklist

#### Step 1: Backend - Types
- [ ] Change `Token` enum to `String` type alias
- [ ] Add `TokenMetadata` struct
- [ ] Add `CreateToken` transaction variant
- [ ] Update tests

#### Step 2: Backend - Ledger
- [ ] Add `tokens` registry to `Ledger` struct
- [ ] Implement `create_token()` method
- [ ] Implement `get_token_metadata()` method
- [ ] Add token validation

#### Step 3: Backend - RPC
- [ ] Add `POST /token/create` endpoint
- [ ] Add `GET /tokens` endpoint
- [ ] Add `GET /token/:symbol` endpoint
- [ ] Update balance endpoint to work with dynamic tokens
- [ ] Add tests

#### Step 4: Frontend - API
- [ ] Add `TokenAPI` module to app.js
- [ ] Add functions for token creation, listing, querying
- [ ] Update `BalanceAPI` to support dynamic tokens

#### Step 5: Frontend - UI
- [ ] Add "Tokens" tab to Navigator
- [ ] Create token creation panel
- [ ] Create token listing view
- [ ] Add token metadata display
- [ ] Integration with existing balance queries

#### Step 6: Testing & Docs
- [ ] E2E tests for token creation
- [ ] E2E tests for token transfers
- [ ] Update API documentation
- [ ] Update Explorer guide

---

## Data Flow Example: Creating a Stablecoin

```
User clicks "Create Token"
    ↓
Enters: Symbol="USDC", Name="USD Coin", Decimals=6, Supply=1B
    ↓
Explorer calls: POST /token/create
    ↓
Node receives CreateToken transaction
    ↓
Ledger validates (token doesn't exist)
    ↓
Creates TokenMetadata:
{
  "symbol": "USDC",
  "name": "USD Coin",
  "decimals": 6,
  "total_supply": 1_000_000_000,
  "creator": "admin"
}
    ↓
Mints 1B USDC to admin account
    ↓
Token registered in tokens registry
    ↓
Success response: {"token": "USDC", "supply": 1B, ...}
    ↓
Explorer displays "USDC Token Created"
    ↓
Token appears in token list
    ↓
Users can now transfer USDC
```

---

## Phase 3: Smart Tokens via Capsules (Future)

When you want complex token logic:

```
SmartToken Workflow:

1. Write Capsule (WASM)
   - validates transfers
   - implements custom mint/burn
   - handles yield/rebasing
   
2. Deploy Capsule
   - Get capsule ID
   
3. Create Token + Link Capsule
   - POST /token/create with capsule_id
   
4. On Token Operations
   - Transfer: Call capsule for validation
   - Mint: Call capsule for custom logic
   - Custom: Call capsule for any operation

Example: Stablecoin Capsule
- Validates: price within bands
- Rebase: adjusts supply based on price
- Limits: rate limiting on transfers
```

---

## Backward Compatibility

**Important:** Keep existing "proof" and "flower" tokens working

```rust
impl Ledger {
    pub fn new(admin: AccountId) -> Self {
        let mut ledger = Ledger { ... };
        
        // Create built-in tokens
        ledger.tokens.insert("proof".to_string(), TokenMetadata {
            name: "Proof Token".to_string(),
            symbol: "PROOF".to_string(),
            decimals: 0,
            total_supply: 0, // Can mint more
            creator: admin.clone(),
        });
        
        ledger.tokens.insert("flower".to_string(), TokenMetadata {
            name: "FloweCoin".to_string(),
            symbol: "FLOWER".to_string(),
            decimals: 6,
            total_supply: 0,
            creator: admin.clone(),
        });
        
        ledger
    }
}
```

---

## Summary

| Aspect | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|----------|
| **Feature** | Dynamic Tokens | Token Management | Smart Tokens |
| **Capsules?** | ❌ No | ❌ No | ✅ Yes |
| **Complexity** | Low | Medium | High |
| **Time** | 2-3h | 2-3h | 4-6h |
| **Effort** | Small | Medium | Large |
| **Use Case** | Basic tokens | Advanced control | Complex logic |

**Recommendation:** Start with **Phase 1** - 
- ✅ Delivers value immediately
- ✅ No capsule complexity needed
- ✅ Foundation for Phase 3 later
- ✅ Users can create stablecoins with metadata

Would you like me to proceed with implementing Phase 1?
