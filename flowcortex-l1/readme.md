# flowcortex-l1

A minimal L1 blockchain prototype derived from the original `flowcortex-l0` crate.

This version provides:

- In-memory ledger with two tokens: **PROOF** (native) and **FloweR** (stablecoin).
- Simple node struct that maintains state and transaction history.
- HTTP RPC server (via **axum**) exposing endpoints for account creation, minting, transfers, and balance queries.
- No distributed consensus; transactions are applied sequentially in the order received, consistent with the "orderingless" vision.

## Getting started

```sh
cd flowcortex-l1
cargo run
```

The server listens on `127.0.0.1:3000` and starts with an `admin` account and two sample accounts (`alice`, `bob`).
State is automatically saved to `node_state.json` after each operation and reloaded on startup if the file exists.

### Binding and access

The node listens on the address specified by the `BIND_ADDR` environment variable (default `0.0.0.0:3000`). Use this to expose the RPC API on a public interface, or restrict it to `127.0.0.1` for local-only testing. The `scripts/run_servers.sh` helper sets this variable for you.

A separate append-only block log (`blocks.log`) records each block in newline-delimited JSON; this file is written alongside state.

Transactions now declare their read/write sets and may carry a `proof` object compatible with the Quantum Cascade Tree API. Proofs are generated automatically by the node and are verified when present; a mismatched proof rejects the transaction. This begins the path toward a stateless verification model.

Conflict detection is performed when transactions are submitted to the pool: two pending transactions that write the same key are rejected. This is a first step toward FlowGraph semantics.

A background block producer runs every 5 seconds, automatically cutting blocks from the pool (no external trigger needed). This mimics a trivial consensus/leader mechanism.

### RPC endpoints

- `POST /account`  - create a new account
  ```json
  { "account": "foo" }
  ```

- `POST /mint`     - mint tokens (admin only)
  **NOTE:** requests may include `rw_set` and an optional `proof` field for future stateless verification.
  ```json
  { "caller": "admin", "to": "alice", "token": "proof", "amount": 100 }
  ```

- `POST /transfer` - send tokens between accounts
  ```json
  { "from": "alice", "to": "bob", "token": "flower", "amount": 25 }
  ```

- `GET /balance/:account/:token` - query account balance
  example: `/balance/alice/proof`
- `GET /pool` - list pending transactions waiting for a block
- `POST /block` - create a new block from the current pool (returns height/tx list)
- `GET /blocks` - list all blocks created since start
- `GET /snapshot` - obtain current state snapshot root (hex)

> **Persistence & logging**
> - `node_state.json` holds the full node snapshot (ledger, pool, blocks) and is reloaded at startup.
> - `blocks.log` is an append‑only JSON‑lines log of every block produced.
> - A background producer runs every 5 s and automatically seals new blocks when the pool is non‑empty.

## Running tests

```sh
cargo test
```

### End-to-End integration

A basic async integration test lives in `tests/e2e.rs` and exercises the HTTP API by
spawning a temporary node process. The test runs automatically with `cargo test`.



```