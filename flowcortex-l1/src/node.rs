use crate::ledger::Ledger;
use crate::types::{AccountId, Block, Transaction, TransactionKind, Token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Minimal node implementation maintaining a ledger, transaction pool, and simple block chain.
///
/// No consensus or networking is implemented; this is a single-process prototype.
#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub ledger: Ledger,
    pub admin: AccountId,
    /// current snapshot root of the state (simple hash). In a full QCT
    /// implementation this would be the commitment root. This field allows the
    /// node to operate without needing to scan the whole ledger for verification.
    pub snapshot_root: Vec<u8>,
    /// sequential list of transactions that have been processed. orderingless blockchain means we
    /// do not rely on this order for correctness; it is for auditing.
    pub history: Vec<Transaction>,
    /// transactions waiting to be included in a block
    pub pool: Vec<Transaction>,
    /// simple linear chain of blocks (height starts at 1)
    pub blocks: Vec<Block>,
    /// Uploaded capsules (WASM binaries) keyed by user-defined identifier
    pub capsules: HashMap<String, Vec<u8>>,
    /// Anchored proofs or data uploaded via AnchorProof transactions
    pub anchors: HashMap<String, Vec<u8>>,
}

impl Node {
    pub fn new(admin: AccountId) -> Self {
        Node {
            ledger: Ledger::new(admin.clone()),
            admin: admin,
            snapshot_root: vec![],
            history: Vec::new(),
            pool: Vec::new(),
            blocks: Vec::new(),
            capsules: HashMap::new(),
            anchors: HashMap::new(),
        }
    }

    /// Convenience method to create an account through the underlying ledger.
    pub fn create_account(&mut self, acct: &AccountId) {
        self.ledger.create_account(acct);
    }

    /// Returns the balance of `acct` for the specified token.
    pub fn balance(&self, acct: &AccountId, token: &Token) -> u64 {
        self.ledger.balance(acct, token)
    }

    /// store a capsule binary under the given id
    pub fn store_capsule(&mut self, id: &str, code: Vec<u8>) -> Result<(), crate::types::LedgerError> {
        if self.capsules.contains_key(id) {
            return Err(crate::types::LedgerError::CapsuleError(format!(
                "capsule `{}` already exists", id
            )));
        }
        self.capsules.insert(id.to_string(), code);
        Ok(())
    }

    /// execute a stored capsule; return arbitrary output bytes (STUB — legacy path)
    pub fn execute_capsule(&self, id: &str, _input: &[u8]) -> Result<Vec<u8>, String> {
        // Legacy stub path — kept for backward-compatibility.
        // For real WASM execution, use `execute_capsule_wasm()`.
        if self.capsules.contains_key(id) {
            Ok(b"executed".to_vec())
        } else {
            Err(format!("capsule `{}` not found", id))
        }
    }

    /// Execute a stored WASM capsule using the wasmtime runtime.
    ///
    /// This is the **real** WASM execution path. It compiles the capsule,
    /// provides sandboxed host functions for ledger operations (mint, transfer,
    /// burn, balance-query, logging), and atomically applies the accumulated
    /// operations to the ledger on success.
    ///
    /// Returns the `CapsuleResult` containing return code, logs, ops, and output.
    pub fn execute_capsule_wasm(
        &mut self,
        id: &str,
        input: &[u8],
    ) -> Result<crate::wasm_capsule::CapsuleResult, String> {
        let code = self
            .capsules
            .get(id)
            .ok_or_else(|| format!("capsule `{}` not found", id))?
            .clone();

        // Build a read-only balance snapshot for the guest
        let balances = self.ledger.balance_snapshot();

        let engine =
            crate::wasm_capsule::WasmCapsuleEngine::new().map_err(|e| e.to_string())?;
        let module = engine.compile(&code)?;
        let result = engine.execute(&module, input, balances)?;

        if result.return_code != 0 {
            return Err(format!(
                "capsule `{}` exited with code {}",
                id, result.return_code
            ));
        }

        // Apply accumulated ops to the real ledger
        for op in &result.ops {
            match op {
                crate::wasm_capsule::CapsuleOp::Mint { to, token, amount } => {
                    self.ledger
                        .mint(&self.admin.clone(), &to, token.clone(), *amount)
                        .map_err(|e| format!("wasm op mint failed: {e}"))?;
                }
                crate::wasm_capsule::CapsuleOp::Transfer { from, to, token, amount } => {
                    self.ledger
                        .transfer(&from, &to, token.clone(), *amount)
                        .map_err(|e| format!("wasm op transfer failed: {e}"))?;
                }
                crate::wasm_capsule::CapsuleOp::Burn { token, from, amount } => {
                    self.ledger
                        .burn(&self.admin.clone(), &token, &from, *amount)
                        .map_err(|e| format!("wasm op burn failed: {e}"))?;
                }
                crate::wasm_capsule::CapsuleOp::Log { .. } => { /* already in result.logs */ }
            }
        }

        Ok(result)
    }

    /// Apply a transaction to the ledger, recording it in history.
    /// This is the primitive used by both pool submission and block creation.
    ///
    /// This method assumes the caller has already been authenticated (e.g.
    /// signature verified) by the caller of `apply_signed_transaction`.
    pub(crate) fn apply_transaction(
        &mut self,
        caller: &AccountId,
        tx: Transaction,
    ) -> Result<(), crate::types::LedgerError> {
        // proof validation: verify QCT proof matches declared read/write set
        if let Some(pf) = &tx.proof {
            if !crate::qct::verify(pf, &tx.rw_set) {
                return Err(crate::types::LedgerError::Conflict); // misuse conflict for proof failure for now
            }
        }

        match &tx.kind {
            TransactionKind::Mint { to, token, amount } => {
                self.ledger.mint(caller, to, token.clone(), *amount)?;
            }
            TransactionKind::Transfer { from, to, token, amount } => {
                self.ledger.transfer(from, to, token.clone(), *amount)?;
            }
            TransactionKind::UploadCapsule { id, code } => {
                self.store_capsule(id, code.clone())?;
            }
            TransactionKind::ExecuteCapsule { id, input } => {
                let _ = self.execute_capsule(id, input);
            }
            TransactionKind::AnchorProof { id, proof } => {
                if self.anchors.contains_key(id) {
                    return Err(crate::types::LedgerError::CapsuleError(format!(
                        "anchor `{}` already exists", id
                    )));
                }
                self.anchors.insert(id.clone(), proof.clone());
            }
            TransactionKind::Trade { from, to, proof_amount, flower_amount } => {
                // simple two‑way transfer; price logic would be added later or
                // provided by capsule code.
                self.ledger.transfer(from, to, "proof".to_string(), *proof_amount)?;
                self.ledger.transfer(to, from, "flower".to_string(), *flower_amount)?;
            }
            // Token management operations (will be implemented in Phase 2)
            TransactionKind::CreateToken { symbol, name, decimals, initial_supply, token_type, metadata } => {
                // TODO: implement token creation in ledger
            }
            TransactionKind::Burn { token, from, amount } => {
                // TODO: implement token burning in ledger
            }
            TransactionKind::FreezeToken { token } => {
                // TODO: implement token freeze in ledger
            }
            TransactionKind::UnfreezeToken { token } => {
                // TODO: implement token unfreeze in ledger
            }
            TransactionKind::SettlementMint { token, to, amount, reference, metadata } => {
                self.ledger.settlement_mint(caller, &token, *amount, reference.clone())?;
            }
            TransactionKind::SettlementBurn { token, from, amount, reference, metadata } => {
                self.ledger.settlement_burn(caller, &token, *amount, reference.clone())?;
            }
            TransactionKind::SettlementTransfer { token, from, to, amount, reference, metadata } => {
                self.ledger.settlement_transfer(from, to, &token, *amount, reference.clone())?;
            }
        }
        // update snapshot root as a simple hash of ledger length + last tx
        self.history.push(tx.clone());
        self.compute_snapshot_root();
        Ok(())
    }

    /// Submit a transaction into the pending pool.
    /// The ledger is validated immediately so invalid txs are rejected.
    /// This is a convenience wrapper around `apply_transaction`.
    pub fn submit_transaction(
        &mut self,
        caller: &AccountId,
        tx: Transaction,
    ) -> Result<(), crate::types::LedgerError> {
        // conflict detection: do not allow two pending transactions to write the same key
        for pending in &self.pool {
            for w in &tx.rw_set.writes {
                if pending.rw_set.writes.contains(w) {
                    return Err(crate::types::LedgerError::Conflict);
                }
            }
        }
        self.apply_transaction(caller, tx.clone())?;
        self.pool.push(tx);
        Ok(())
    }

    /// Same as `submit_transaction` but takes a signed transaction and
    /// verifies the signature first.
    pub fn submit_signed_transaction(
        &mut self,
        stx: crate::types::SignedTransaction,
    ) -> Result<(), crate::types::LedgerError> {
        // conflict detection still uses inner tx
        let tx = &stx.tx;
        for pending in &self.pool {
            for w in &tx.rw_set.writes {
                if pending.rw_set.writes.contains(w) {
                    return Err(crate::types::LedgerError::Conflict);
                }
            }
        }
        self.apply_signed_transaction(stx.clone())?;
        self.pool.push(tx.clone());
        Ok(())
    }

    /// Create a block capturing whatever is currently in the pool.
    /// The block height increments automatically. The pool is cleared.
    pub fn create_block(&mut self) -> Block {
        let height = (self.blocks.len() as u64) + 1;
        let block = Block {
            height,
            transactions: self.pool.clone(),
        };
        self.blocks.push(block.clone());
        // append to a simple on-disk log file
        let _ = self.append_block_log("blocks.log");
        self.pool.clear();
        // snapshot root already updated by individual tx applications
        block
    }

    /// Return a copy of pending transactions
    pub fn pending_pool(&self) -> Vec<Transaction> {
        self.pool.clone()
    }

    /// recalc the snapshot root from ledger state.
    fn compute_snapshot_root(&mut self) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        // naive: hash account count and balances
        let bals = self.ledger.all_balances();
        hasher.update(bals.len().to_le_bytes());
        for (acct, m) in bals {
            hasher.update(acct.as_bytes());
            for (tok, amt) in m {
                hasher.update(format!("{:?}:{}", tok, amt).as_bytes());
            }
        }
        self.snapshot_root = hasher.finalize().to_vec();
    }

    /// save node state to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let encoded = serde_json::to_string_pretty(self)?;
        fs::write(path, encoded)
    }

    /// append a single block to the on-disk log (jsonlines)
    pub fn append_block_log<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let encoded = serde_json::to_string(&self.blocks.last())?;
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", encoded)
    }

    /// load node state from a file
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let node = serde_json::from_str(&data)?;
        Ok(node)
    }

    /// Verify a signed transaction and apply it if the signature is valid.
    ///
    /// The message that is signed is simply the serialized `Transaction` object;
    /// in a full implementation this would be canonicalized or hashed separately.
    pub fn apply_signed_transaction(
        &mut self,
        stx: crate::types::SignedTransaction,
    ) -> Result<(), crate::types::LedgerError> {
        let msg = serde_json::to_vec(&stx.tx).map_err(|e| {
            crate::types::LedgerError::CapsuleError(format!("serde: {}", e))
        })?;
        use ed25519_dalek::{PublicKey, Signature, Verifier};
        let pk = PublicKey::from_bytes(&stx.pubkey)
            .map_err(|_| crate::types::LedgerError::InvalidSignature)?;
        let sig = Signature::from_bytes(&stx.signature)
            .map_err(|_| crate::types::LedgerError::InvalidSignature)?;
        if pk.verify(&msg, &sig).is_err() {
            return Err(crate::types::LedgerError::InvalidSignature);
        }
        self.apply_transaction(&stx.caller, stx.tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Token, Transaction, TransactionKind, ReadWriteSet, QCTProof};

    #[test]
    fn node_mint_and_query() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        node.create_account(&"alice".to_string());

        // mint 500 proof into alice
        let tx = Transaction {
            kind: TransactionKind::Mint {
                to: "alice".to_string(),
                token: "proof".to_string(),
                amount: 500,
            },
            rw_set: ReadWriteSet {
                reads: vec![admin.clone()],
                writes: vec!["alice".to_string()],
            },
            proof: None,
        };
        assert!(node.apply_transaction(&admin, tx).is_ok());
        assert_eq!(node.balance(&"alice".to_string(), &"proof".to_string()), 500);
    }

    #[test]
    fn pool_and_block_flow() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        node.create_account(&"alice".to_string());
        node.create_account(&"bob".to_string());

        // submit a transfer (will mint implicit proof to alice for test)
        let _ = node
            .submit_transaction(
                &admin,
                Transaction {
                    kind: TransactionKind::Mint { to: "alice".to_string(), token: "proof".to_string(), amount: 100 },
                    rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
                    proof: None,
                },
            )
            .unwrap();
        let _ = node
            .submit_transaction(
                &"alice".to_string(),
                Transaction {
                    kind: TransactionKind::Transfer { from: "alice".to_string(), to: "bob".to_string(), token: "proof".to_string(), amount: 30 },
                    rw_set: ReadWriteSet { reads: vec!["alice".to_string()], writes: vec!["bob".to_string()] },
                    proof: None,
                },
            )
            .unwrap();

        assert_eq!(node.pending_pool().len(), 2);
        let block = node.create_block();
        assert_eq!(block.height, 1);
        assert_eq!(block.transactions.len(), 2);
        assert!(node.pending_pool().is_empty());
        assert_eq!(node.blocks.len(), 1);
    }

    #[test]
    fn conflicting_transactions_are_rejected() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        node.create_account(&"alice".to_string());
        // first transaction writes alice
        let t1 = Transaction {
            kind: TransactionKind::Mint { to: "alice".to_string(), token: "proof".to_string(), amount: 10 },
            rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
            proof: None,
        };
        assert!(node.submit_transaction(&admin, t1).is_ok());
        // second transaction also writes alice -> conflict
        let t2 = Transaction {
            kind: TransactionKind::Transfer { from: "alice".to_string(), to: "bob".to_string(), token: "proof".to_string(), amount: 5 },
            rw_set: ReadWriteSet { reads: vec!["alice".to_string()], writes: vec!["alice".to_string()] },
            proof: None,
        };
        assert!(matches!(node.submit_transaction(&"alice".to_string(), t2), Err(crate::types::LedgerError::Conflict)));
    }

    #[test]
    fn snapshot_root_changes_after_tx() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        let before = node.snapshot_root.clone();
        let tx = Transaction {
            kind: TransactionKind::Mint {
                to: "alice".to_string(),
                token: "proof".to_string(),
                amount: 1,
            },
            rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
            proof: None,
        };
        assert!(node.apply_transaction(&admin, tx).is_ok());
        assert_ne!(node.snapshot_root, before);
    }

    #[test]
    fn proof_verification_fails_with_bad_data() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        // create a transaction with mismatched proof
        let mut tx = Transaction {
            kind: TransactionKind::Mint { to: "alice".to_string(), token: "proof".to_string(), amount: 10 },
            rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
            proof: Some(QCTProof(vec![0,1,2])),
        };
        assert!(matches!(node.apply_transaction(&admin, tx.clone()), Err(crate::types::LedgerError::Conflict)));
    }

    #[test]
    fn log_file_is_written() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        // clear any existing log file
        let _ = std::fs::remove_file("blocks.log");
        // add one transaction to pool and create block
        let _ = node.submit_transaction(
            &admin,
            Transaction {
                kind: TransactionKind::Mint { to: "alice".to_string(), token: "proof".to_string(), amount: 10 },
                rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
                proof: None,
            },
        )
        .unwrap();
        let block = node.create_block();
        // append_block_log called inside create_block should have created the file
        let contents = std::fs::read_to_string("blocks.log").unwrap();
        assert!(contents.contains(&format!("\"height\":{}", block.height)));
        let _ = std::fs::remove_file("blocks.log");
    }

    #[test]
    fn anchor_proof_stored_and_retrievable() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        let tx = Transaction {
            kind: TransactionKind::AnchorProof { id: "proof1".to_string(), proof: vec![1,2,3] },
            rw_set: ReadWriteSet::default(),
            proof: None,
        };
        assert!(node.apply_transaction(&admin, tx).is_ok());
        assert_eq!(node.anchors.get("proof1"), Some(&vec![1,2,3]));
    }

    #[test]
    fn signed_transaction_rejected_with_wrong_key() {
        use ed25519_dalek::{Keypair, Signer, SecretKey, PublicKey};

        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);

        // construct a keypair from a constant secret bytes (not secure, used only
        // for tests).  Public key is derived automatically.
        let sk_bytes = [42u8; 32];
        let secret = SecretKey::from_bytes(&sk_bytes).unwrap();
        let public = PublicKey::from(&secret);
        let kp = Keypair { secret, public };

        // build a simple mint transaction
        let tx = Transaction {
            kind: TransactionKind::Mint { to: "alice".to_string(), token: "proof".to_string(), amount: 5 },
            rw_set: ReadWriteSet::default(),
            proof: None,
        };
        let msg = serde_json::to_vec(&tx).unwrap();
        let sig = kp.sign(&msg).to_bytes().to_vec();
        let stx = crate::types::SignedTransaction {
            caller: admin.clone(),
            pubkey: kp.public.to_bytes().to_vec(),
            signature: sig,
            tx: tx.clone(),
        };
        // flip a byte to create an invalid signature
        let mut bad = stx.clone();
        bad.signature[0] ^= 0xff;
        assert!(matches!(node.apply_signed_transaction(bad), Err(crate::types::LedgerError::InvalidSignature)));
    }
}
