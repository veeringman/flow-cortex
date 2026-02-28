///! WASM Capsule Runtime — deterministic execution of user-uploaded WASM modules
///! on FlowCortex L1 with sandboxed host functions for ledger operations.
///!
///! # Architecture
///!
///! ```text
///! ┌──────────────────────────────────────────────────┐
///! │  WASM Capsule  (guest)                           │
///! │  ┌──────────────────────────────────────────┐    │
///! │  │  User logic (Rust/C/AssemblyScript → WASM)│    │
///! │  │  Calls host_* imports for ledger access   │    │
///! │  └──────────────┬───────────────────────────┘    │
///! └─────────────────┼────────────────────────────────┘
///!                   │ host function imports
///! ┌─────────────────▼────────────────────────────────┐
///! │  Host Functions (Rust, wasmtime)                  │
///! │  ┌────────────┐ ┌───────────┐ ┌───────────────┐  │
///! │  │ host_mint  │ │host_xfer  │ │host_balance   │  │
///! │  │ host_burn  │ │host_anchor│ │host_log       │  │
///! │  └────────────┘ └───────────┘ └───────────────┘  │
///! │  All operate on a mutable ledger reference via    │
///! │  wasmtime Store data                              │
///! └──────────────────────────────────────────────────┘
///! ```
///!
///! The current path (native Rust `VerifierCapsule` trait objects) remains the
///! **primary** execution model. WASM capsules are an **alternative option**
///! for user-defined, sandboxed logic.

use std::collections::HashMap;
use wasmtime::*;

// ============================================================================
// Host State — passed into the WASM Store for host-function access
// ============================================================================

/// Operations that the WASM guest accumulated during execution.
/// These are applied to the real ledger atomically after the capsule exits.
#[derive(Debug, Clone)]
pub enum CapsuleOp {
    Mint { to: String, token: String, amount: u64 },
    Transfer { from: String, to: String, token: String, amount: u64 },
    Burn { token: String, from: String, amount: u64 },
    Log { message: String },
}

/// Outcome of a capsule execution.
#[derive(Debug, Clone)]
pub struct CapsuleResult {
    /// Return code from the guest `capsule_main` (0 = success).
    pub return_code: i32,
    /// Accumulated ledger operations to be applied by the caller.
    pub ops: Vec<CapsuleOp>,
    /// Log messages emitted by the guest.
    pub logs: Vec<String>,
    /// Raw output bytes (written by the guest to a shared buffer).
    pub output: Vec<u8>,
}

/// Internal data stored in the wasmtime `Store`.
struct HostState {
    /// Accumulated operations (not yet applied to ledger).
    ops: Vec<CapsuleOp>,
    /// Log messages from the guest.
    logs: Vec<String>,
    /// Read-only balance snapshot for the guest to query.
    balances: HashMap<String, HashMap<String, u64>>,
    /// Shared memory buffer for guest output.
    output_buf: Vec<u8>,
    /// Guest-allocated memory (set once guest exports "memory").
    memory: Option<Memory>,
}

impl HostState {
    fn new(balances: HashMap<String, HashMap<String, u64>>) -> Self {
        HostState {
            ops: Vec::new(),
            logs: Vec::new(),
            balances,
            output_buf: Vec::new(),
            memory: None,
        }
    }
}

// ============================================================================
// WASM Capsule Engine
// ============================================================================

/// The engine that compiles and runs WASM capsule modules.
pub struct WasmCapsuleEngine {
    engine: Engine,
}

impl WasmCapsuleEngine {
    pub fn new() -> Result<Self, String> {
        let mut config = Config::new();
        // Deterministic execution: no fuel metering for now but limit memory.
        config.wasm_bulk_memory(true);
        config.wasm_multi_value(true);
        // Cranelift is the default compiler backend.
        let engine = Engine::new(&config).map_err(|e| format!("wasmtime engine init: {e}"))?;
        Ok(WasmCapsuleEngine { engine })
    }

    /// Compile a WASM module from raw bytes. Returns a serialized module
    /// that can be cached for fast re-instantiation.
    pub fn compile(&self, wasm_bytes: &[u8]) -> Result<Module, String> {
        Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("wasm compilation failed: {e}"))
    }

    /// Execute a compiled capsule module.
    ///
    /// # Parameters
    /// - `module`: compiled WASM module
    /// - `input`: opaque input bytes passed to the guest via shared memory
    /// - `balances`: read-only snapshot of account balances (for `host_balance`)
    ///
    /// # Guest Contract
    /// The WASM module MUST export:
    /// - `memory`: linear memory
    /// - `capsule_main(input_ptr: i32, input_len: i32) -> i32`: entry point
    /// - `alloc(size: i32) -> i32`: allocator for the host to write input
    ///
    /// The WASM module MAY import (from `"env"` module):
    /// - `host_mint(to_ptr, to_len, token_ptr, token_len, amount) -> i32`
    /// - `host_transfer(from_ptr, from_len, to_ptr, to_len, token_ptr, token_len, amount) -> i32`
    /// - `host_burn(token_ptr, token_len, from_ptr, from_len, amount) -> i32`
    /// - `host_balance(account_ptr, account_len, token_ptr, token_len) -> i64`
    /// - `host_log(msg_ptr, msg_len)`
    /// - `host_output(data_ptr, data_len)`
    pub fn execute(
        &self,
        module: &Module,
        input: &[u8],
        balances: HashMap<String, HashMap<String, u64>>,
    ) -> Result<CapsuleResult, String> {
        let mut store = Store::new(&self.engine, HostState::new(balances));
        let mut linker = Linker::new(&self.engine);

        // ----- Register host functions -----
        Self::register_host_functions(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| format!("wasm instantiation failed: {e}"))?;

        // Grab guest memory export
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| "guest does not export 'memory'".to_string())?;
        store.data_mut().memory = Some(memory);

        // Allocate space in guest for the input buffer
        let alloc = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .map_err(|e| format!("guest missing 'alloc' export: {e}"))?;

        let input_ptr = alloc
            .call(&mut store, input.len() as i32)
            .map_err(|e| format!("alloc failed: {e}"))?;

        // Write input bytes into guest memory
        memory
            .write(&mut store, input_ptr as usize, input)
            .map_err(|e| format!("memory write failed: {e}"))?;

        // Call the guest entry point
        let capsule_main = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "capsule_main")
            .map_err(|e| format!("guest missing 'capsule_main' export: {e}"))?;

        let return_code = capsule_main
            .call(&mut store, (input_ptr, input.len() as i32))
            .map_err(|e| format!("capsule_main trapped: {e}"))?;

        let state = store.into_data();
        Ok(CapsuleResult {
            return_code,
            ops: state.ops,
            logs: state.logs,
            output: state.output_buf,
        })
    }

    fn register_host_functions(linker: &mut Linker<HostState>) -> Result<(), String> {
        // ---- host_mint(to_ptr, to_len, token_ptr, token_len, amount) -> i32 ----
        linker
            .func_wrap(
                "env",
                "host_mint",
                |mut caller: Caller<'_, HostState>,
                 to_ptr: i32,
                 to_len: i32,
                 token_ptr: i32,
                 token_len: i32,
                 amount: i64|
                 -> i32 {
                    let to = match read_guest_string(&caller, to_ptr, to_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    let token = match read_guest_string(&caller, token_ptr, token_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    caller.data_mut().ops.push(CapsuleOp::Mint {
                        to,
                        token,
                        amount: amount as u64,
                    });
                    0 // success
                },
            )
            .map_err(|e| format!("link host_mint: {e}"))?;

        // ---- host_transfer(from_ptr, from_len, to_ptr, to_len, token_ptr, token_len, amount) -> i32 ----
        linker
            .func_wrap(
                "env",
                "host_transfer",
                |mut caller: Caller<'_, HostState>,
                 from_ptr: i32,
                 from_len: i32,
                 to_ptr: i32,
                 to_len: i32,
                 token_ptr: i32,
                 token_len: i32,
                 amount: i64|
                 -> i32 {
                    let from = match read_guest_string(&caller, from_ptr, from_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    let to = match read_guest_string(&caller, to_ptr, to_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    let token = match read_guest_string(&caller, token_ptr, token_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    caller.data_mut().ops.push(CapsuleOp::Transfer {
                        from,
                        to,
                        token,
                        amount: amount as u64,
                    });
                    0
                },
            )
            .map_err(|e| format!("link host_transfer: {e}"))?;

        // ---- host_burn(token_ptr, token_len, from_ptr, from_len, amount) -> i32 ----
        linker
            .func_wrap(
                "env",
                "host_burn",
                |mut caller: Caller<'_, HostState>,
                 token_ptr: i32,
                 token_len: i32,
                 from_ptr: i32,
                 from_len: i32,
                 amount: i64|
                 -> i32 {
                    let token = match read_guest_string(&caller, token_ptr, token_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    let from = match read_guest_string(&caller, from_ptr, from_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    caller.data_mut().ops.push(CapsuleOp::Burn {
                        token,
                        from,
                        amount: amount as u64,
                    });
                    0
                },
            )
            .map_err(|e| format!("link host_burn: {e}"))?;

        // ---- host_balance(account_ptr, account_len, token_ptr, token_len) -> i64 ----
        linker
            .func_wrap(
                "env",
                "host_balance",
                |caller: Caller<'_, HostState>,
                 account_ptr: i32,
                 account_len: i32,
                 token_ptr: i32,
                 token_len: i32|
                 -> i64 {
                    let account = match read_guest_string(&caller, account_ptr, account_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    let token = match read_guest_string(&caller, token_ptr, token_len) {
                        Ok(s) => s,
                        Err(_) => return -1,
                    };
                    let bal = caller
                        .data()
                        .balances
                        .get(&account)
                        .and_then(|m| m.get(&token))
                        .cloned()
                        .unwrap_or(0);
                    bal as i64
                },
            )
            .map_err(|e| format!("link host_balance: {e}"))?;

        // ---- host_log(msg_ptr, msg_len) ----
        linker
            .func_wrap(
                "env",
                "host_log",
                |mut caller: Caller<'_, HostState>, msg_ptr: i32, msg_len: i32| {
                    if let Ok(msg) = read_guest_string(&caller, msg_ptr, msg_len) {
                        caller.data_mut().logs.push(msg);
                    }
                },
            )
            .map_err(|e| format!("link host_log: {e}"))?;

        // ---- host_output(data_ptr, data_len) ----
        linker
            .func_wrap(
                "env",
                "host_output",
                |mut caller: Caller<'_, HostState>, data_ptr: i32, data_len: i32| {
                    if let Ok(mem) = get_memory(&caller) {
                        let mut buf = vec![0u8; data_len as usize];
                        if mem.read(&caller, data_ptr as usize, &mut buf).is_ok() {
                            caller.data_mut().output_buf = buf;
                        }
                    }
                },
            )
            .map_err(|e| format!("link host_output: {e}"))?;

        Ok(())
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Read a UTF-8 string from guest linear memory at `[ptr..ptr+len)`.
fn read_guest_string(caller: &Caller<'_, HostState>, ptr: i32, len: i32) -> Result<String, String> {
    let mem = get_memory(caller)?;
    let mut buf = vec![0u8; len as usize];
    mem.read(caller, ptr as usize, &mut buf)
        .map_err(|e| format!("read guest memory: {e}"))?;
    String::from_utf8(buf).map_err(|e| format!("invalid utf-8: {e}"))
}

/// Retrieve the "memory" export cached in HostState.
fn get_memory(caller: &Caller<'_, HostState>) -> Result<Memory, String> {
    caller
        .data()
        .memory
        .ok_or_else(|| "guest memory not set".to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal WAT (WebAssembly Text) that exports memory, alloc, capsule_main.
    /// capsule_main ignores input and returns 0 (success).
    const TRIVIAL_WAT: &str = r#"
        (module
            (memory (export "memory") 1)
            (global $bump (mut i32) (i32.const 1024))
            (func (export "alloc") (param $size i32) (result i32)
                global.get $bump
                global.get $bump
                local.get $size
                i32.add
                global.set $bump
            )
            (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
                i32.const 0
            )
        )
    "#;

    /// WAT that calls host_log with a message.
    const LOG_WAT: &str = r#"
        (module
            (import "env" "host_log" (func $host_log (param i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "hello from wasm")
            (global $bump (mut i32) (i32.const 1024))
            (func (export "alloc") (param $size i32) (result i32)
                global.get $bump
                global.get $bump
                local.get $size
                i32.add
                global.set $bump
            )
            (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
                i32.const 0
                i32.const 15
                call $host_log
                i32.const 0
            )
        )
    "#;

    /// WAT that calls host_mint and host_transfer.
    const OPS_WAT: &str = r#"
        (module
            (import "env" "host_mint" (func $host_mint (param i32 i32 i32 i32 i64) (result i32)))
            (import "env" "host_transfer" (func $host_transfer (param i32 i32 i32 i32 i32 i32 i64) (result i32)))
            (import "env" "host_balance" (func $host_balance (param i32 i32 i32 i32) (result i64)))
            (memory (export "memory") 1)
            ;; Pre-loaded strings at fixed offsets:
            ;; offset 0:  "alice"   (5 bytes)
            ;; offset 5:  "bob"     (3 bytes)
            ;; offset 8:  "flower"  (6 bytes)
            (data (i32.const 0)  "alicebobflower")
            (global $bump (mut i32) (i32.const 1024))
            (func (export "alloc") (param $size i32) (result i32)
                global.get $bump
                global.get $bump
                local.get $size
                i32.add
                global.set $bump
            )
            (func (export "capsule_main") (param $ptr i32) (param $len i32) (result i32)
                ;; mint 1000 flower to alice
                i32.const 0   ;; to_ptr = "alice"
                i32.const 5   ;; to_len
                i32.const 8   ;; token_ptr = "flower"
                i32.const 6   ;; token_len
                i64.const 1000 ;; amount
                call $host_mint
                drop

                ;; transfer 500 flower from alice to bob
                i32.const 0   ;; from_ptr = "alice"
                i32.const 5   ;; from_len
                i32.const 5   ;; to_ptr = "bob"
                i32.const 3   ;; to_len
                i32.const 8   ;; token_ptr = "flower"
                i32.const 6   ;; token_len
                i64.const 500 ;; amount
                call $host_transfer
                drop

                ;; query balance of alice/flower (just to exercise the host fn)
                i32.const 0   ;; account_ptr = "alice"
                i32.const 5   ;; account_len
                i32.const 8   ;; token_ptr = "flower"
                i32.const 6   ;; token_len
                call $host_balance
                drop

                i32.const 0  ;; return success
            )
        )
    "#;

    #[test]
    fn test_trivial_capsule() {
        let engine = WasmCapsuleEngine::new().expect("engine init");
        let wasm = wat::parse_str(TRIVIAL_WAT).expect("parse trivial WAT");
        let module = engine.compile(&wasm).expect("compile");
        let result = engine
            .execute(&module, b"hello", HashMap::new())
            .expect("execute");
        assert_eq!(result.return_code, 0);
        assert!(result.ops.is_empty());
        assert!(result.logs.is_empty());
    }

    #[test]
    fn test_capsule_with_logging() {
        let engine = WasmCapsuleEngine::new().expect("engine init");
        let wasm = wat::parse_str(LOG_WAT).expect("parse log WAT");
        let module = engine.compile(&wasm).expect("compile");
        let result = engine
            .execute(&module, b"", HashMap::new())
            .expect("execute");
        assert_eq!(result.return_code, 0);
        assert_eq!(result.logs, vec!["hello from wasm"]);
    }

    #[test]
    fn test_capsule_with_ledger_ops() {
        let engine = WasmCapsuleEngine::new().expect("engine init");
        let wasm = wat::parse_str(OPS_WAT).expect("parse ops WAT");
        let module = engine.compile(&wasm).expect("compile");

        // Provide a balance snapshot so host_balance can read it
        let mut balances = HashMap::new();
        let mut alice_bal = HashMap::new();
        alice_bal.insert("flower".to_string(), 10_000u64);
        balances.insert("alice".to_string(), alice_bal);

        let result = engine
            .execute(&module, b"", balances)
            .expect("execute");

        assert_eq!(result.return_code, 0);
        assert_eq!(result.ops.len(), 2);

        // First op: mint 1000 flower to alice
        match &result.ops[0] {
            CapsuleOp::Mint { to, token, amount } => {
                assert_eq!(to, "alice");
                assert_eq!(token, "flower");
                assert_eq!(*amount, 1000);
            }
            other => panic!("expected Mint, got {:?}", other),
        }

        // Second op: transfer 500 flower from alice to bob
        match &result.ops[1] {
            CapsuleOp::Transfer { from, to, token, amount } => {
                assert_eq!(from, "alice");
                assert_eq!(to, "bob");
                assert_eq!(token, "flower");
                assert_eq!(*amount, 500);
            }
            other => panic!("expected Transfer, got {:?}", other),
        }
    }

    #[test]
    fn test_capsule_not_found() {
        let engine = WasmCapsuleEngine::new().expect("engine init");
        // Invalid WASM should fail compilation
        let result = engine.compile(b"not valid wasm");
        assert!(result.is_err());
    }
}
