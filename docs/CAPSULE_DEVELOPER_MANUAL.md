# FlowCortex L1 — Capsule Developer Manual

> **Version:** 0.1 · **Last updated:** 2026-03-02

Capsules are user-uploaded WebAssembly (WASM) modules that run in a sandboxed
environment on the FlowCortex L1 chain.  They can mint, transfer, and burn
tokens, query balances, emit logs, and return structured output — all through a
small set of **host functions** injected by the runtime.

---

## Table of Contents

1. [Quick Start](#1-quick-start)
2. [Guest Contract](#2-guest-contract)
3. [Host Function Reference](#3-host-function-reference)
4. [ABI Details](#4-abi-details)
5. [Return Codes & Error Handling](#5-return-codes--error-handling)
6. [Security & Sandbox Model](#6-security--sandbox-model)
7. [Determinism Requirements](#7-determinism-requirements)
8. [Building & Deploying](#8-building--deploying)
9. [HTTP API Reference](#9-http-api-reference)
10. [Example Capsules](#10-example-capsules)

---

## 1. Quick Start

```bash
# 1. Write a capsule in WAT or Rust (see examples below)

# 2. Compile to WASM (Rust example)
cargo build --target wasm32-unknown-unknown --release
wasm-strip target/wasm32-unknown-unknown/release/my_capsule.wasm   # optional

# 3. Base64-encode and upload
BASE64=$(base64 -w0 target/wasm32-unknown-unknown/release/my_capsule.wasm)
curl -s http://192.168.29.78:3000/capsule \
  -H 'Content-Type: application/json' \
  -d "{\"id\": \"my_capsule\", \"code\": \"$BASE64\"}"

# 4. Invoke
INPUT=$(echo -n '{"action":"hello"}' | base64 -w0)
curl -s http://192.168.29.78:3000/capsule/my_capsule/invoke \
  -H 'Content-Type: application/json' \
  -d "{\"input\": \"$INPUT\"}" | jq .
```

---

## 2. Guest Contract

Every capsule WASM module **MUST** export:

| Export | Type | Description |
|--------|------|-------------|
| `memory` | Memory (≥ 1 page) | Linear memory shared between guest and host |
| `alloc(size: i32) -> i32` | Function | Bump allocator; host calls this to write input |
| `capsule_main(ptr: i32, len: i32) -> i32` | Function | Entry point; returns 0 for success |

The runtime calls:
1. `alloc(input.len)` → receives a pointer
2. Writes the input bytes at that pointer in guest memory
3. `capsule_main(ptr, len)` → receives a return code (0 = success)

The module **MAY** import any subset of [host functions](#3-host-function-reference)
from the `"env"` module.

### Minimal WAT Skeleton

```wat
(module
  (memory (export "memory") 1)          ;; 1 page = 64 KiB
  (global $bump (mut i32) (i32.const 1024))

  (func (export "alloc") (param $size i32) (result i32)
    global.get $bump
    global.get $bump
    local.get $size
    i32.add
    global.set $bump
  )

  (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
    ;; Your logic here
    i32.const 0   ;; return success
  )
)
```

---

## 3. Host Function Reference

All host functions are imported from the `"env"` module.  String arguments
are passed as `(ptr: i32, len: i32)` pairs pointing into guest linear memory.

### `host_mint`

```
(import "env" "host_mint"
  (func (param i32 i32 i32 i32 i64) (result i32)))
;;         to  len token len amount
```

| Param | Type | Description |
|-------|------|-------------|
| `to_ptr`, `to_len` | i32, i32 | UTF-8 recipient account name |
| `token_ptr`, `token_len` | i32, i32 | UTF-8 token symbol |
| `amount` | i64 | Amount to mint (unsigned) |
| **Returns** | i32 | 0 = success, -1 = error |

Queues a mint operation.  Not applied until `capsule_main` returns 0.

### `host_transfer`

```
(import "env" "host_transfer"
  (func (param i32 i32 i32 i32 i32 i32 i64) (result i32)))
;;         from len to  len token len amount
```

| Param | Type | Description |
|-------|------|-------------|
| `from_ptr`, `from_len` | i32, i32 | Sender account |
| `to_ptr`, `to_len` | i32, i32 | Recipient account |
| `token_ptr`, `token_len` | i32, i32 | Token symbol |
| `amount` | i64 | Amount to transfer |
| **Returns** | i32 | 0 = success, -1 = error |

### `host_burn`

```
(import "env" "host_burn"
  (func (param i32 i32 i32 i32 i64) (result i32)))
;;         token len from len amount
```

| Param | Type | Description |
|-------|------|-------------|
| `token_ptr`, `token_len` | i32, i32 | Token symbol |
| `from_ptr`, `from_len` | i32, i32 | Account to burn from |
| `amount` | i64 | Amount to burn |
| **Returns** | i32 | 0 = success, -1 = error |

### `host_balance`

```
(import "env" "host_balance"
  (func (param i32 i32 i32 i32) (result i64)))
;;         acct len token len
```

| Param | Type | Description |
|-------|------|-------------|
| `account_ptr`, `account_len` | i32, i32 | Account to query |
| `token_ptr`, `token_len` | i32, i32 | Token symbol |
| **Returns** | i64 | Balance (0 if not found, -1 on error) |

Reads from a **snapshot** taken before capsule execution.  Mutations
(`host_mint`, `host_transfer`, `host_burn`) are NOT reflected until the
capsule finishes and ops are applied to the real ledger.

### `host_log`

```
(import "env" "host_log" (func (param i32 i32)))
;;                                msg  len
```

Appends a UTF-8 log message.  Visible in the invoke response `logs` array.

### `host_output`

```
(import "env" "host_output" (func (param i32 i32)))
;;                                 data len
```

Sets the capsule's output buffer (returned base64-encoded in the response).
Calling multiple times **replaces** the previous output.

---

## 4. ABI Details

| Aspect | Detail |
|--------|--------|
| **Pointer width** | i32 (32-bit linear memory addresses) |
| **Amount type** | i64 (unsigned 64-bit, passed as WASM i64) |
| **String encoding** | UTF-8, not null-terminated |
| **Memory model** | Single linear memory, guest-allocated via `alloc` |
| **Endianness** | Little-endian (WASM spec) |
| **Max memory** | Default 1 page (64 KiB); guest can `memory.grow` |

### Calling Convention

```text
Host → Guest:   alloc(size) → ptr, then memcpy input at ptr
                capsule_main(ptr, len) → return_code

Guest → Host:   Push (ptr, len) args onto WASM stack, call imported fn
                Host reads guest memory at [ptr..ptr+len)
```

---

## 5. Return Codes & Error Handling

| `capsule_main` return | Meaning | Ops applied? |
|----------------------|---------|--------------|
| `0` | Success | ✅ Yes |
| Non-zero | Failure | ❌ No (all ops discarded) |
| Trap (unreachable, OOM, etc.) | Abort | ❌ No |

Host function return codes:
- `0` — operation queued successfully
- `-1` — parameter error (invalid pointer, bad UTF-8, etc.)

**Important:** Ledger operations are accumulated during execution and applied
**atomically** after `capsule_main` returns 0.  If the guest returns non-zero
or traps, ALL accumulated operations are discarded.

---

## 6. Security & Sandbox Model

| Property | Guarantee |
|----------|-----------|
| **Memory isolation** | Guest can only access its own linear memory |
| **No filesystem** | No WASI — no `fd_read`, `fd_write`, etc. |
| **No network** | Cannot open sockets or make HTTP calls |
| **No clock** | No access to system time (determinism) |
| **Ledger access** | Only via host functions; read-only snapshot for queries |
| **Atomic execution** | Ops applied only on success (return 0) |
| **No re-entrancy** | Capsule execution is single-threaded, sequential |

Capsules run in a pure computational sandbox.  They can observe external state
only through the `host_balance` snapshot and the input bytes.

---

## 7. Determinism Requirements

For capsules that participate in consensus or proof-of-commitment flows:

1. **No floating point** — stick to integer arithmetic
2. **No randomness** — no `random()` import available
3. **No timing** — no access to wall-clock time
4. **Deterministic allocation** — the bump allocator in the skeleton is sufficient
5. **Same input → same output** — all inputs come through `capsule_main(ptr, len)`

Non-deterministic capsules will still execute correctly but cannot be used as
verifier capsules in the commitment/proof pipeline.

---

## 8. Building & Deploying

### From Rust

```bash
# Cargo.toml
[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true

# Build
cargo build --target wasm32-unknown-unknown --release

# Optional: strip debug info (requires wasm-strip from wabt)
wasm-strip target/wasm32-unknown-unknown/release/my_capsule.wasm

# Upload
BASE64=$(base64 -w0 target/wasm32-unknown-unknown/release/my_capsule.wasm)
curl -s http://192.168.29.78:3000/capsule \
  -H 'Content-Type: application/json' \
  -d "{\"id\": \"my_capsule\", \"code\": \"$BASE64\"}"
```

### From WAT (WebAssembly Text)

```bash
# Install wabt tools
sudo apt install wabt  # or brew install wabt

# Compile WAT → WASM
wat2wasm my_capsule.wat -o my_capsule.wasm

# Upload
BASE64=$(base64 -w0 my_capsule.wasm)
curl -s http://192.168.29.78:3000/capsule \
  -H 'Content-Type: application/json' \
  -d "{\"id\": \"my_capsule\", \"code\": \"$BASE64\"}"
```

### Rust Guest Helpers

For a clean Rust guest API, define raw extern imports and safe wrappers:

```rust
// src/lib.rs — #![no_std] capsule
#![no_std]
#![no_main]

extern "C" {
    fn host_mint(to: *const u8, to_len: i32, token: *const u8, token_len: i32, amount: i64) -> i32;
    fn host_transfer(from: *const u8, from_len: i32, to: *const u8, to_len: i32, token: *const u8, token_len: i32, amount: i64) -> i32;
    fn host_burn(token: *const u8, token_len: i32, from: *const u8, from_len: i32, amount: i64) -> i32;
    fn host_balance(account: *const u8, account_len: i32, token: *const u8, token_len: i32) -> i64;
    fn host_log(msg: *const u8, msg_len: i32);
    fn host_output(data: *const u8, data_len: i32);
}

/// Safe wrapper: mint tokens
fn mint(to: &str, token: &str, amount: u64) -> i32 {
    unsafe { host_mint(to.as_ptr(), to.len() as i32, token.as_ptr(), token.len() as i32, amount as i64) }
}

/// Safe wrapper: transfer tokens
fn transfer(from: &str, to: &str, token: &str, amount: u64) -> i32 {
    unsafe { host_transfer(from.as_ptr(), from.len() as i32, to.as_ptr(), to.len() as i32, token.as_ptr(), token.len() as i32, amount as i64) }
}

/// Safe wrapper: query balance (snapshot)
fn balance(account: &str, token: &str) -> i64 {
    unsafe { host_balance(account.as_ptr(), account.len() as i32, token.as_ptr(), token.len() as i32) }
}

/// Safe wrapper: emit log
fn log(msg: &str) {
    unsafe { host_log(msg.as_ptr(), msg.len() as i32) }
}

/// Safe wrapper: set output
fn output(data: &[u8]) {
    unsafe { host_output(data.as_ptr(), data.len() as i32) }
}

// Bump allocator
static mut BUMP: usize = 4096;

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    unsafe {
        let ptr = BUMP;
        BUMP += size as usize;
        ptr as i32
    }
}

#[no_mangle]
pub extern "C" fn capsule_main(ptr: i32, len: i32) -> i32 {
    // Read input (unsafe slice from guest memory)
    let input = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };

    // --- Your logic here ---
    log("capsule started");
    mint("treasury", "USDC", 1_000_000);
    transfer("treasury", "alice", "USDC", 500_000);

    let bal = balance("alice", "USDC");
    log("done");

    0 // success
}

// Panic handler for #![no_std]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
```

---

## 9. HTTP API Reference

### Upload Capsule

```
POST /capsule
Content-Type: application/json

{
  "id": "my_capsule",
  "code": "<base64-encoded WASM bytes>"
}
```

**Response:** `201 Created` on success, `400` if WASM compilation fails.

### List Capsules

```
GET /capsule

Response: { "capsules": ["my_capsule", "compliance_check", ...] }
```

### Invoke Capsule (Legacy Path)

```
POST /capsule/:id/invoke
Content-Type: application/json

{
  "input": "<base64-encoded input bytes>"
}

Response: { "output": "<base64-encoded output>" }
```

### Invoke Capsule (WASM Runtime)

```
POST /capsule/:id/invoke_wasm
Content-Type: application/json

{
  "input": "<base64-encoded input bytes>"
}

Response:
{
  "return_code": 0,
  "output": "<base64-encoded output>",
  "logs": ["capsule started", "done"],
  "ops_applied": 2
}
```

The `invoke_wasm` endpoint runs the capsule through the real wasmtime engine
and returns richer output including logs, return code, and the number of ledger
operations applied.

---

## 10. Example Capsules

### Example 1: Hello World (Logging)

```wat
(module
  (import "env" "host_log" (func $log (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "Hello from WASM capsule!")
  (global $bump (mut i32) (i32.const 1024))

  (func (export "alloc") (param $size i32) (result i32)
    global.get $bump
    global.get $bump  local.get $size  i32.add  global.set $bump)

  (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
    i32.const 0  i32.const 23  call $log    ;; "Hello from WASM capsule!"
    i32.const 0)
)
```

### Example 2: Balance Checker

Queries a balance and returns it as output.

```wat
(module
  (import "env" "host_balance" (func $balance (param i32 i32 i32 i32) (result i64)))
  (import "env" "host_output"  (func $output  (param i32 i32)))
  (import "env" "host_log"     (func $log     (param i32 i32)))
  (memory (export "memory") 1)
  ;; Pre-loaded: "alice" at 0, "USDC" at 5, "balance:" at 9
  (data (i32.const 0) "aliceUSDCbalance:")
  (global $bump (mut i32) (i32.const 1024))

  (func (export "alloc") (param $size i32) (result i32)
    global.get $bump
    global.get $bump  local.get $size  i32.add  global.set $bump)

  (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
    ;; Query balance of alice/USDC
    (call $balance (i32.const 0) (i32.const 5) (i32.const 5) (i32.const 4))
    ;; Store the i64 result at memory offset 100
    i64.store (i32.const 100)
    ;; Output the 8-byte balance value
    (call $output (i32.const 100) (i32.const 8))
    (call $log (i32.const 9) (i32.const 8))   ;; "balance:"
    i32.const 0)
)
```

### Example 3: Automated FloweR Distribution

Mints tokens to a treasury and distributes to multiple recipients.

```wat
(module
  (import "env" "host_mint"     (func $mint     (param i32 i32 i32 i32 i64) (result i32)))
  (import "env" "host_transfer" (func $transfer (param i32 i32 i32 i32 i32 i32 i64) (result i32)))
  (import "env" "host_log"      (func $log      (param i32 i32)))
  (memory (export "memory") 1)
  ;; String table:
  ;; 0:  "treasury" (8)
  ;; 8:  "alice"    (5)
  ;; 13: "bob"      (3)
  ;; 16: "flower"   (6)
  ;; 22: "minted"   (6)
  ;; 28: "distributed" (11)
  (data (i32.const 0) "treasuryalicebobflowerminteddistributed")
  (global $bump (mut i32) (i32.const 1024))

  (func (export "alloc") (param $size i32) (result i32)
    global.get $bump
    global.get $bump  local.get $size  i32.add  global.set $bump)

  (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
    ;; Mint 10000 flower to treasury
    (call $mint (i32.const 0) (i32.const 8)     ;; to = "treasury"
               (i32.const 16) (i32.const 6)     ;; token = "flower"
               (i64.const 10000))                ;; amount
    drop
    (call $log (i32.const 22) (i32.const 6))     ;; "minted"

    ;; Transfer 3000 to alice
    (call $transfer (i32.const 0) (i32.const 8)  ;; from = "treasury"
                    (i32.const 8) (i32.const 5)  ;; to = "alice"
                    (i32.const 16) (i32.const 6) ;; token = "flower"
                    (i64.const 3000))
    drop

    ;; Transfer 2000 to bob
    (call $transfer (i32.const 0) (i32.const 8)  ;; from = "treasury"
                    (i32.const 13) (i32.const 3) ;; to = "bob"
                    (i32.const 16) (i32.const 6) ;; token = "flower"
                    (i64.const 2000))
    drop
    (call $log (i32.const 28) (i32.const 11))    ;; "distributed"

    i32.const 0)
)
```

### Example 4: Compliance Threshold Check

Checks a balance threshold before allowing a transfer.  Returns non-zero
(failure) if the sender's balance is below a minimum reserve.

```wat
(module
  (import "env" "host_balance"  (func $balance  (param i32 i32 i32 i32) (result i64)))
  (import "env" "host_transfer" (func $transfer (param i32 i32 i32 i32 i32 i32 i64) (result i32)))
  (import "env" "host_log"      (func $log      (param i32 i32)))
  (memory (export "memory") 1)
  ;; 0: "alice" (5), 5: "bob" (3), 8: "USDC" (4)
  ;; 12: "ok" (2), 14: "below_reserve" (13)
  (data (i32.const 0) "alicebobUSDCokbelow_reserve")
  (global $bump (mut i32) (i32.const 1024))

  (func (export "alloc") (param $size i32) (result i32)
    global.get $bump
    global.get $bump  local.get $size  i32.add  global.set $bump)

  (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
    ;; Check alice's USDC balance
    (call $balance (i32.const 0) (i32.const 5)   ;; "alice"
                   (i32.const 8) (i32.const 4))  ;; "USDC"

    ;; If balance < 10000, abort (maintain minimum reserve)
    i64.const 10000
    i64.lt_s
    (if (result i32)
      (then
        (call $log (i32.const 14) (i32.const 13))  ;; "below_reserve"
        i32.const 1)    ;; FAIL — ops are discarded
      (else
        ;; Transfer 5000 from alice to bob
        (call $transfer (i32.const 0) (i32.const 5)    ;; alice
                        (i32.const 5) (i32.const 3)    ;; bob
                        (i32.const 8) (i32.const 4)    ;; USDC
                        (i64.const 5000))
        drop
        (call $log (i32.const 12) (i32.const 2))       ;; "ok"
        i32.const 0))   ;; SUCCESS — ops are applied
  )
)
```

---

## Appendix: Rust Guest `Cargo.toml` Template

```toml
[package]
name = "my-capsule"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true
strip = true
```

Build with:
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```
