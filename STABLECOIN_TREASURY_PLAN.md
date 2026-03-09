# FlowCortex Stablecoin & Treasury Settlement Platform - Implementation Plan

**Status:** Planning  
**Date:** February 22, 2026  
**Scope:** PROOF (native token) + Stablecoins (FloweR, USDC, etc.) + Treasury Settlement

---

## Executive Summary

This plan enables banks to mint, manage, and transfer stablecoins (like FloweR) on FlowCortex L1 through Treasury Settlement Platforms, while maintaining PROOF as the native network token.

### Key Capabilities
- ✅ **Multi-token system:** PROOF (native) + multiple stablecoins
- ✅ **Bank operations:** Mint/burn stablecoins, transfer between wallets
- ✅ **Settlement:** Bank-to-bank, bank-to-counterparty transfers
- ✅ **Treasury integration:** Settlement platform controls minting/burning
- ✅ **Compliance:** Audit trails, transaction history, metadata

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│              Treasury Settlement Platform (Off-chain)        │
│  (Manages banks, KYC, minting authorization, settlement)    │
└─────────────────────┬──────────────────────────────────────┘
                      │
        ┌─────────────┴──────────────┐
        │                            │
   Bank Apps                   Admin/Treasury API
   (Wallets)                   (Mint/Burn requests)
        │                            │
        └─────────────┬──────────────┘
                      │
        ┌─────────────▼──────────────────────┐
        │    FlowCortex L1 RPC/GRPC API      │
        │  (REST + GRPC endpoints)           │
        └─────────────┬──────────────────────┘
                      │
     ┌────────────────┼────────────────┐
     │                │                │
  Transfer        Token Ops         Query
  (Bank→Counterparty) (Mint/Burn)    (Balance)
     │                │                │
     └────────────────┼────────────────┘
                      │
        ┌─────────────▼──────────────────┐
        │     FlowCortex L1 Ledger       │
        │  (State, Balances, Metadata)   │
        └────────────────────────────────┘
```

---

## Phase 1: Multi-Token System (Dynamic Stablecoin Support)

### Current State
- Token system uses hardcoded enum (Proof, FloweR)
- Single token type per transaction
- No dynamic token creation
- No token metadata

### Requirements

#### 1. **Dynamic Token Registry**
- Convert `Token` enum → `String` (token symbol)
- Create `TokenMetadata` struct with:
  - Symbol (unique identifier)
  - Name (display name)
  - Decimals (precision for amounts)
  - Total supply
  - Creator/issuer
  - Token type (native, stablecoin, etc.)
  - Status (active, frozen, paused)

#### 2. **Token Creation & Management**
```rust
pub struct TokenMetadata {
    pub symbol: String,           // "FLOWER", "USDC", "USDT"
    pub name: String,             // "Flow Rupee", "USD Coin"
    pub decimals: u8,             // 6 for stablecoins
    pub total_supply: u64,        // Total minted
    pub creator: AccountId,       // Treasury platform
    pub token_type: TokenType,    // Native, Stablecoin, etc.
    pub status: TokenStatus,      // Active, Frozen, Paused
    pub created_at: u64,          // Block height
    pub metadata: Option<String>, // JSON: backing, collateral, etc.
}

pub enum TokenType {
    Native,      // PROOF - used for fees/stake
    Stablecoin,  // FloweR, USDC - pegged to fiat
    Governance,  // Future: voting rights
    Utility,     // Future: other purposes
}

pub enum TokenStatus {
    Active,
    Frozen,      // No transfers
    Paused,      // Can't mint new
    Deprecated,
}
```

#### 3. **Ledger Updates**
```rust
pub struct Ledger {
    // Existing:
    pub admin: AccountId,
    pub height: u64,
    pub balances: HashMap<AccountId, HashMap<String, u64>>,  // Token symbol as String
    
    // New:
    pub tokens: HashMap<String, TokenMetadata>,  // Token registry
    pub token_history: Vec<TokenEvent>,          // Audit log for token ops
}

pub enum TokenEvent {
    Created {
        symbol: String,
        creator: AccountId,
        height: u64,
    },
    Minted {
        symbol: String,
        to: AccountId,
        amount: u64,
        height: u64,
    },
    Burned {
        symbol: String,
        from: AccountId,
        amount: u64,
        height: u64,
    },
    Frozen {
        symbol: String,
        height: u64,
    },
}
```

#### 4. **Transaction Types**
Add new variants to `TransactionKind`:

```rust
pub enum TransactionKind {
    // ... existing Mint, Transfer, etc.
    
    // NEW: Token operations
    CreateToken {
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: u64,
        token_type: TokenType,
        metadata: Option<String>,
    },
    
    Burn {
        token: String,
        from: AccountId,
        amount: u64,
    },
    
    FreezeToken {
        token: String,
    },
    
    UnfreezeToken {
        token: String,
    },
}
```

### Implementation Files

**Backend Changes:**

1. **`flowcortex-l1/src/types.rs`**
   - Change `Token` enum → `type Token = String`
   - Add `TokenMetadata`, `TokenType`, `TokenStatus`
   - Add `TokenEvent`, `CreateToken`, `Burn` transaction variants

2. **`flowcortex-l1/src/ledger.rs`**
   - Add `tokens: HashMap<String, TokenMetadata>`
   - Add `token_history: Vec<TokenEvent>`
   - Implement `create_token()` method
   - Implement `burn_token()` method
   - Implement `freeze/unfreeze_token()` methods
   - Update mint/transfer to work with dynamic tokens
   - Add token validation before operations

3. **`flowcortex-l1/src/node.rs`**
   - Add token event handling in transaction execution
   - Update state commit to include token registry
   - Add token validation middleware

### API Endpoints (Phase 1)

#### Token Management
```
POST   /token/create
GET    /tokens
GET    /token/{symbol}
GET    /token/{symbol}/history
POST   /token/{symbol}/mint
POST   /token/{symbol}/burn
POST   /token/{symbol}/freeze
POST   /token/{symbol}/unfreeze
```

#### Examples

**Create FloweR Stablecoin:**
```json
POST /token/create
{
  "symbol": "FLOWER",
  "name": "Flow Rupee",
  "decimals": 6,
  "initial_supply": 1000000000000,  // 1B tokens
  "token_type": "Stablecoin",
  "metadata": {
    "backing": "1:1 INR reserve at Treasury",
    "issuer": "FTC Treasury",
    "collateral_type": "USD",
    "redemption_rate": "1.0"
  }
}
```

**List All Tokens:**
```json
GET /tokens
[
  {
    "symbol": "PROOF",
    "name": "PROOF",
    "decimals": 0,
    "total_supply": 20000000000,
    "token_type": "Native",
    "status": "Active"
  },
  {
    "symbol": "FLOWER",
    "name": "Flow Rupee",
    "decimals": 6,
    "total_supply": 1000000000000,
    "token_type": "Stablecoin",
    "status": "Active"
  }
]
```

**Get Token Metadata:**
```json
GET /token/FLOWER
{
  "symbol": "FLOWER",
  "name": "Flow Rupee",
  "decimals": 6,
  "total_supply": 1000000000000,
  "creator": "treasury-admin",
  "token_type": "Stablecoin",
  "status": "Active",
  "created_at": 12345,
  "metadata": {
    "backing": "1:1 USD reserve",
    "issuer": "FTC Treasury"
  }
}
```

---

## Phase 2: Treasury Settlement Operations

### Banks & Wallets

```rust
pub struct BankAccount {
    pub account_id: AccountId,      // "bank.finterra.com"
    pub bank_name: String,
    pub swift_code: String,
    pub is_approved: bool,
    pub created_at: u64,
}

pub struct TreasuryWallet {
    pub account_id: AccountId,              // "treasury.settlement"
    pub name: String,
    pub balances: HashMap<String, u64>,     // Token → amount
    pub daily_mint_limits: HashMap<String, u64>,  // Per token
    pub daily_minted: HashMap<String, u64>,       // Daily counter
}
```

### Settlement Operations

#### 1. **Mint/Buy Stablecoin (Bank → Treasury)**
```
Bank Request:
  Account: "bank-a.institution.com"
  Token: "FLOWER"
  Amount: 1000000000  (1M tokens = $1M)
  Purpose: "Initial funding"
  
Flow:
  1. Treasury validates bank is approved
  2. Treasury validates bank hasn't exceeded daily limit
  3. Treasury validates collateral (USD available)
  4. Ledger mints 1M FLOWER to bank-a account
  5. Treasury receives USD off-chain (banking settlement)
  
Result:
  Bank A balance: +1M FLOWER
  Treasury keeps USD
```

#### 2. **Transfer between Banks (Bank A → Bank B)**
```
Transfer Request:
  From: "bank-a.institution.com"
  To: "bank-b.institution.com"
  Token: "FLOWER"
  Amount: 500000000  (500K FLOWER = $500K)
  
Flow:
  1. Validate both banks are approved
  2. Validate bank-a has sufficient FLOWER balance
  3. Ledger transfers 500K FLOWER: bank-a → bank-b
  4. Settlement complete on-chain instantly
  
Result:
  Bank A: -500K FLOWER
  Bank B: +500K FLOWER
  Transaction recorded on-chain
```

#### 3. **Burn/Sell Stablecoin (Bank → Treasury)**
```
Burn Request:
  Account: "bank-a.institution.com"
  Token: "FLOWER"
  Amount: 100000000  (100K = $100K)
  Purpose: "Withdraw collateral"
  
Flow:
  1. Bank submits burn request
  2. Treasury validates (can redeem)
  3. Ledger burns 100K FLOWER from bank-a
  4. Treasury sends USD off-chain to bank
  
Result:
  Bank A: -100K FLOWER
  Treasury: -equivalent USD (off-chain)
  Total supply: -100K FLOWER
```

### Transaction Types (Phase 2)

```rust
pub enum TransactionKind {
    // ... Phase 1 variants
    
    // Settlement operations
    SettlementMint {
        token: String,
        to: AccountId,          // bank account
        amount: u64,
        reference: String,      // off-chain reference
    },
    
    SettlementBurn {
        token: String,
        from: AccountId,
        amount: u64,
        reference: String,
    },
    
    SettlementTransfer {
        token: String,
        from: AccountId,
        to: AccountId,
        amount: u64,
        reference: String,      // Settlement reference
        metadata: Option<String>, // Purpose, notes, etc.
    },
}
```

### API Endpoints (Phase 2)

```
// Settlement operations
POST   /settlement/mint
POST   /settlement/burn
POST   /settlement/transfer
POST   /settlement/approve-bank
GET    /settlement/banks
GET    /settlement/status/{reference}
GET    /settlement/history/{bank-account}

// Treasury controls
POST   /treasury/set-limit
GET    /treasury/limits
GET    /treasury/daily-usage

// Compliance
GET    /audit/transactions/{bank}
GET    /audit/token-supply/{token}
```

### Example Flows

**Settle Payment: Bank A → Bank B via FloweR**
```json
POST /settlement/transfer
{
  "token": "FLOWER",
  "from": "bank-a.institution.com",
  "to": "bank-b.institution.com",
  "amount": 500000000,
  "reference": "PAYREF-2026-02-22-001",
  "metadata": {
    "purpose": "Payment for goods",
    "invoice": "INV-2024-001",
    "settlement_date": "2026-02-22"
  }
}

Response:
{
  "tx_hash": "0x123abc...",
  "block_height": 50234,
  "status": "confirmed",
  "timestamp": "2026-02-22T15:30:45Z",
  "from_balance": "1500000000",
  "to_balance": "2500000000"
}
```

---

## Phase 3: Advanced Features (Future)

### 1. **Rate Limiting & Daily Caps**
- Per-bank daily mint limits
- Per-token rate limits
- Anti-flood protection

### 2. **Multi-Collateral Stablecoins**
- Track backing assets
- Collateral ratios
- Reserve validation

### 3. **Token Swaps (PROOF ↔ Stablecoins)**
```rust
pub enum TransactionKind {
    Swap {
        from_token: String,
        to_token: String,
        from_amount: u64,
        to_amount: u64,
        from: AccountId,
        to: Option<AccountId>,  // DEX if None
    },
}
```

### 4. **Governance & Policy**
- Token freeze/unfreeze
- Supply caps
- Fee collection
- Redistribution

### 5. **Smart Contracts (via Capsules)**
- Rebase logic for stablecoins
- Algorithmic minting/burning
- Custom settlement rules
- Yield distribution

---

## Implementation Roadmap

### Immediate (Week 1-2): Phase 1
- [ ] Update types.rs: Token → String, add TokenMetadata
- [ ] Update ledger.rs: add token registry & methods
- [ ] Update node.rs: handle token transactions
- [ ] Add RPC endpoints: /token/*, /tokens
- [ ] Add E2E tests for token creation/transfer
- [ ] Update Explorer UI with token management

### Short-term (Week 3-4): Phase 2
- [ ] Add BankAccount & TreasuryWallet structures
- [ ] Implement settlement operations
- [ ] Add settlement RPC endpoints
- [ ] Add bank approval & limit controls
- [ ] Build Treasury dashboard UI
- [ ] Add audit logging

### Medium-term (Week 5-6): Banking Features
- [ ] Add daily limit enforcement
- [ ] Add transaction fee collection
- [ ] Add settlement status tracking
- [ ] Build bank portal UI
- [ ] Add compliance reports

### Long-term: Phase 3
- [ ] Rate limiting system
- [ ] Multi-collateral tracking
- [ ] Token swap mechanism
- [ ] Capsule-based smart tokens

---

## Data Models

### Core Additions

```rust
// Token System
HashMap<String, TokenMetadata> = Token Registry
Vec<TokenEvent> = Token Audit Log

// Settlement System
HashMap<String, BankAccount> = Approved Banks
HashMap<String, TreasuryWallet> = Treasury Wallets
Vec<SettlementRecord> = Settlement History

// Balances (updated)
HashMap<AccountId, HashMap<String, u64>> = All token balances
```

### Example Database Schema (Pseudocode)

```sql
-- Tokens table
tokens (
  symbol: String (PK),
  name: String,
  decimals: i32,
  total_supply: i64,
  creator: String,
  token_type: String,
  status: String,
  created_at: i64,
  metadata: JSON
)

-- Token events (audit log)
token_events (
  id: Int (PK),
  symbol: String (FK),
  event_type: String,
  account: String,
  amount: i64,
  block_height: i64,
  timestamp: i64
)

-- Accounts/Banks
bank_accounts (
  account_id: String (PK),
  bank_name: String,
  swift_code: String,
  is_approved: Boolean,
  created_at: i64
)

-- Transactions
transactions (
  tx_hash: String (PK),
  kind: String,
  from: String,
  to: String,
  token: String,
  amount: i64,
  block_height: i64,
  timestamp: i64,
  status: String,
  metadata: JSON
)
```

---

## Security & Compliance

### Access Control
```
Admin/Treasury Only:
  - Create tokens
  - Approve banks
  - Set limits
  - Freeze tokens
  
Banks:
  - Mint (within limits)
  - Transfer
  - Burn
  - Query balances
  
Any Account:
  - Query token metadata
  - Query transaction history
  - Transfer tokens
```

### Validation Rules
1. **Token Creation:** Treasury only, non-duplicate symbol
2. **Mint:** Treasury or authorized bank, within limits
3. **Burn:** Account owner, sufficient balance
4. **Transfer:** Both accounts exist, sufficient balance, token not frozen
5. **Settlement:** Both parties approved, limits checked

### Audit Trail
- Every token operation logged with:
  - Timestamp / block height
  - Account / bank initiating
  - Token & amount
  - Transaction hash
  - Status (success/failed)

### Compliance Features
- ✅ Full transaction history
- ✅ Failed transaction logging
- ✅ Daily settlement reports
- ✅ Collateral tracking
- ✅ Bank approval workflow
- ✅ Immutable audit logs

---

## UI Components (Explorer Enhancement)

### New Tabs/Sections

#### 1. **Tokens Management**
- Create new token
- List all tokens with metadata
- View token history
- Burn tokens
- Freeze/unfreeze

#### 2. **Treasury Dashboard**
- Total supply per token
- Active banks
- Daily minting usage
- Settlement volume
- Collateral status

#### 3. **Bank Portal**
- Bank balance per token
- Mint/burn requests
- Transfer interface
- Transaction history
- Limits & usage

#### 4. **Settlement Tracking**
- Pending settlements
- Completed settlements
- Failed transactions
- Settlement dates
- Reference tracking

#### 5. **Compliance Reports**
- Transaction audit log
- Bank activity report
- Token supply audit
- Daily settlement report
- Anomaly detection

---

## Testing Strategy

### Unit Tests
- Token creation validation
- Balance updates
- Token existence checks
- Limit enforcement
- Error cases

### Integration Tests
- Multi-token transfers
- Settlement flows
- Limit enforcement
- Bank approval workflow
- Token freeze behavior

### E2E Tests (Explorer)
- Create stablecoin (FloweR)
- Bank mints FLOWER
- Bank A transfers to Bank B
- Bank B burns FLOWER for withdrawal
- Query settlement history
- View compliance reports

---

## Rollout Plan

### Testnet Phase
1. Deploy token system (Phase 1)
2. Deploy settlement ops (Phase 2)
3. Run E2E tests with mock banks
4. Load test with multiple tokens
5. Security audit

### Pilot Phase
1. Go live with PROOF + FloweR
2. Onboard 2-3 pilot banks
3. Test real USD↔FLOWER flow
4. Monitor settlement performance
5. Gather bank feedback

### Production Phase
1. Full bank network
2. Multiple stablecoins (USDC, USDT, etc.)
3. Production limits & monitoring
4. Compliance reporting
5. Regular audits

---

## Success Metrics

- ✅ 10+ banks onboarded
- ✅ $100M+ daily settlement volume
- ✅ <1 second settlement time
- ✅ 99.99% uptime
- ✅ Zero settlement failures
- ✅ Full audit compliance
- ✅ <100ms token operation latency

---

## Questions for Treasury System Design

### For Your Team
1. **Stablecoin backing:** 1:1 fiat reserve? Multi-collateral? Algorithm?
2. **Mint authority:** Treasury only? Banks? Smart contracts?
3. **Daily limits:** Per-bank? Per-token? Tiered by bank size?
4. **Fees:** Transaction fees? Mint/burn fees? Settlement fees?
5. **Collateral:** USD reserve at bank? On-chain proof? Off-chain verification?
6. **Redemption:** Instant? 1-2 days? Conditions?
7. **Compliance:** KYC required? AML checks? Sanctions?
8. **Settlement:** Same-instant? T+0? T+1?
9. **Regulatory:** Which jurisdiction? Which stablecoin standards?
10. **Emergency:** What if FLOWER loses peg? How to handle?

---

## Next Steps

1. **Confirm Phase 1 scope** with team
2. **Review data models** for settlement operations
3. **Define API specifications** with bank partners
4. **Design UI/UX** for bank portal
5. **Plan security audit** for stablecoin system
6. **Create detailed sprint plan** for implementation

---

## References

- Current token plan: `TOKENS_FEATURE_PLAN.md`
- FlowCortex types: `flowcortex-l1/src/types.rs`
- Ledger implementation: `flowcortex-l1/src/ledger.rs`
- Node logic: `flowcortex-l1/src/node.rs`
- RPC API: `flowcortex-l1/src/rpc.rs`

---

**Ready to proceed with Phase 1 implementation?**
