use crate::ledger::Ledger;
use crate::types::{AccountId, Block, Transaction, TransactionKind, Token, ReadWriteSet};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Minimal node implementation maintaining a ledger, transaction pool, and simple block chain.
///
/// No consensus or networking is implemented; this is a single-process prototype.
#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    ledger: Ledger,
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
}

impl Node {
    pub fn new(admin: AccountId) -> Self {
        Node {
            ledger: Ledger::new(admin),
            snapshot_root: vec![],
            history: Vec::new(),
            pool: Vec::new(),
            blocks: Vec::new(),
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

    /// Apply a transaction to the ledger, recording it in history.
    /// This is the primitive used by both pool submission and block creation.
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
                // cheap conflict check: ensure no existing pending tx writes same key
                self.ledger.transfer(from, to, token.clone(), *amount)?;
            }
        }
        // update snapshot root as a simple hash of ledger length + last tx
        self.history.push(tx.clone());
        self.compute_snapshot_root();
        Ok(())
    }

    /// Submit a transaction into the pending pool.
    /// The ledger is validated immediately so invalid txs are rejected.
    pub fn submit_transaction(
        &mut self,
        caller: &AccountId,
        tx: Transaction,
    ) -> Result<(), crate::types::LedgerError> {
        // conflict detection: do not allow two pending transactions to write the same key
        for pending in &self.pool {
            for w in &tx.rw_set.writes {
                if pending.rw_set.writes.contains(w) {
                    // simple conflict: return error using a custom variant or reuse InsufficientBalance
                    return Err(crate::types::LedgerError::Conflict);
                }
            }
        }
        self.apply_transaction(caller, tx.clone())?;
        self.pool.push(tx);
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
                token: Token::Proof,
                amount: 500,
            },
            rw_set: ReadWriteSet {
                reads: vec![admin.clone()],
                writes: vec!["alice".to_string()],
            },
            proof: None,
        };
        assert!(node.apply_transaction(&admin, tx).is_ok());
        assert_eq!(node.balance(&"alice".to_string(), &Token::Proof), 500);
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
                    kind: TransactionKind::Mint { to: "alice".to_string(), token: Token::Proof, amount: 100 },
                    rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
                    proof: None,
                },
            )
            .unwrap();
        let _ = node
            .submit_transaction(
                &"alice".to_string(),
                Transaction {
                    kind: TransactionKind::Transfer { from: "alice".to_string(), to: "bob".to_string(), token: Token::Proof, amount: 30 },
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
            kind: TransactionKind::Mint { to: "alice".to_string(), token: Token::Proof, amount: 10 },
            rw_set: ReadWriteSet { reads: vec![admin.clone()], writes: vec!["alice".to_string()] },
            proof: None,
        };
        assert!(node.submit_transaction(&admin, t1).is_ok());
        // second transaction also writes alice -> conflict
        let t2 = Transaction {
            kind: TransactionKind::Transfer { from: "alice".to_string(), to: "bob".to_string(), token: Token::Proof, amount: 5 },
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
                token: Token::Proof,
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
            kind: TransactionKind::Mint { to: "alice".to_string(), token: Token::Proof, amount: 10 },
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
                kind: TransactionKind::Mint { to: "alice".to_string(), token: Token::Proof, amount: 10 },
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
}
