# FlowCortex Demo — Quick Start Guide

**Date:** March 2, 2026  
**Status:** All 16 Phases Complete — Demo Ready ✅

---

## Quick Demo Commands

### 1. Start All Services (Recommended)

```bash
cd /home/vijay/demo/workspaces
./deploy-local.sh start       # Start all 13 services
./deploy-local.sh diagnose    # Verify 60/60 checks pass
```

### 2. Start FlowCortex Only

```bash
cd /home/vijay/demo/workspaces/flow-cortex
./scripts/run_servers.sh      # Starts L1 node (:3000) + Explorer (:4000)
```

The L1 node runs on `http://192.168.29.78:3000` (REST + gRPC).  
The Explorer UI runs on `http://192.168.29.78:4000` (web).

---

## Service Port Map

| Service | Port | URL |
|---------|------|-----|
| FlowCortex L1 | 3000 | `http://192.168.29.78:3000` |
| AuthBuddy Admin | 3001 | `http://192.168.29.78:3001` |
| FlowCortex Explorer | 3002/4000 | `http://192.168.29.78:3002` |
| FortressDigital Console | 3003 | `http://192.168.29.78:3003` |
| Treasury Frontend | 3004 | `http://192.168.29.78:3004` |
| KeyCortex Wallet (JS) | 3005 | `http://192.168.29.78:3005` |
| KeyCortex Wallet (WASM) | 3006 | `http://192.168.29.78:3006` |
| AuthBuddy API | 8801 | `http://192.168.29.78:8801` |
| KeyCortex API | 8811 | `http://192.168.29.78:8811` |
| FortressDigital API | 8821 | `http://192.168.29.78:8821` |
| Treasury API | 8831 | `http://192.168.29.78:8831` |
| ProofCortex API | 8841 | `http://192.168.29.78:8841` |

---

## FlowCortex L1 API Examples

### Health Check

```bash
curl http://192.168.29.78:3000/status
```

### Create a Token

```bash
curl -X POST http://192.168.29.78:3000/token/create \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "FLW",
    "name": "FloweR Stablecoin",
    "decimals": 6,
    "initial_supply": 0,
    "token_type": "stablecoin",
    "metadata": {"issuer": "treasury", "description": "INR-pegged stablecoin"}
  }'
```

### Create Account & Mint Tokens

```bash
# Create account
curl -X POST http://192.168.29.78:3000/account \
  -H "Content-Type: application/json" \
  -d '{"id": "bank_a"}'

# Mint tokens
curl -X POST http://192.168.29.78:3000/mint \
  -H "Content-Type: application/json" \
  -d '{"account": "bank_a", "token": "FLW", "amount": 50000000}'

# Check balance
curl http://192.168.29.78:3000/balance/bank_a/FLW
```

### Transfer Tokens

```bash
curl -X POST http://192.168.29.78:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{"from": "bank_a", "to": "bank_b", "token": "FLW", "amount": 10000000}'
```

### Produce a Block

```bash
curl -X POST http://192.168.29.78:3000/block
```

### Anchor a Commitment (FortressDigital Integration)

```bash
curl -X POST http://192.168.29.78:3000/api/anchor_commitment \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
    "policy_id": "treasury_settlement_v1",
    "txn_ref": "SETTLE-2026-03-02-001",
    "amount": 5000000000,
    "metadata": {"sender": "Bank A", "receiver": "Bank B", "currency": "INR"}
  }'
```

### Verify a Proof (ProofCortex Integration)

```bash
curl -X POST http://192.168.29.78:3000/api/verify_proof \
  -H "Content-Type: application/json" \
  -d '{
    "commitment_hash": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
    "proof_hash": "deadbeef...",
    "proof_data": [1,2,3],
    "proof_type": "STARK"
  }'
```

### Query Commitment / Proof / Events

```bash
# Commitment status
curl http://192.168.29.78:3000/api/commitment/a1b2c3d4...

# Proof status
curl http://192.168.29.78:3000/api/proof_status/deadbeef...

# All events
curl http://192.168.29.78:3000/api/events

# Dashboard stats
curl http://192.168.29.78:3000/api/stats
```

---

## WASM Capsule Examples

### Upload a Capsule

```bash
# Base64-encode a .wasm file and upload
curl -X POST http://192.168.29.78:3000/capsule \
  -H "Content-Type: application/json" \
  -d '{"id": "hello_capsule", "code": "<base64-wasm>"}'
```

### Invoke a Capsule

```bash
curl -X POST http://192.168.29.78:3000/capsule/hello_capsule/invoke_wasm \
  -H "Content-Type: application/json" \
  -d '{"input": ""}'
```

For authoring capsules, see the [Capsule Developer Manual](docs/CAPSULE_DEVELOPER_MANUAL.md) or use the Capsule Editor IDE in the Explorer UI.

---

## Settlement Routes (Bank Operations)

### Approve a Bank

```bash
curl -X POST http://192.168.29.78:3000/bank/approve \
  -H "Content-Type: application/json" \
  -d '{"account_id": "bank_a"}'
```

### Set Daily Mint Limit

```bash
curl -X POST http://192.168.29.78:3000/bank/daily_limit \
  -H "Content-Type: application/json" \
  -d '{"account_id": "bank_a", "token": "FLW", "limit": 100000000}'
```

### Settlement Mint / Redeem / Transfer

```bash
# Settlement mint (approved banks only)
curl -X POST http://192.168.29.78:3000/settlement/mint \
  -H "Content-Type: application/json" \
  -d '{"caller": "bank_a", "token": "FLW", "amount": 50000000}'

# Settlement redeem (burn)
curl -X POST http://192.168.29.78:3000/settlement/redeem \
  -H "Content-Type: application/json" \
  -d '{"caller": "bank_a", "token": "FLW", "amount": 10000000}'

# Settlement transfer
curl -X POST http://192.168.29.78:3000/settlement/transfer \
  -H "Content-Type: application/json" \
  -d '{"from": "bank_a", "to": "bank_b", "token": "FLW", "amount": 5000000}'
```

---

## FortressDigital Integration Example

```bash
# Ensure FortressDigital is running with real integrations
FLOW_ANCHOR_MODE=http \
PROOF_MODE=http \
CUSTODY_MODE=http \
cargo run --manifest-path FortressDigital/Cargo.toml

# Submit a settlement through FortressDigital
curl -X POST http://192.168.29.78:8821/v1/settlements \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer demo-alice" \
  -H "x-device-id: corp-laptop-17" \
  -H "x-geo-region: corp_hq" \
  -d '{
    "amount": 12000,
    "currency": "FLOWER",
    "counterparty_wallet": "wallet_abc123",
    "purpose_code": "vendor_payout",
    "user_id": "alice",
    "user_role": "treasury_ops"
  }'
```

---

## Demo Scenarios

For guided walkthroughs, see the [demo-scenarios/](../demo-scenarios/) folder:

1. **Identity & Access Setup** — AuthBuddy admin configuration
2. **Wallet Creation** — KeyCortex wallet generation
3. **Token Minting** — Create stablecoins on FlowCortex Explorer
4. **Treasury Settlement** — End-to-end settlement via Treasury UI
5. **Proof Anchoring** — Cryptographic proof on L1 chain
6. **Live Monitoring** — FortressDigital Console real-time feed
7. **RBAC & Access Policy** — Security model deep dive

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Service won't start | Run `./deploy-local.sh diagnose` to identify which check fails |
| Port already in use | `lsof -i :3000` to find and kill the process |
| Token not found | Create the token first with `POST /token/create` |
| Balance is 0 | Mint tokens with `POST /mint` and produce a block with `POST /block` |
| FortressDigital returns mock data | Set `FLOW_ANCHOR_MODE=http` env var before starting |
| Capsule compile fails | Check WAT syntax; use the Example Gallery in the Explorer Capsule Editor |

---

**Ready to Demo!** ✅
