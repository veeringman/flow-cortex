use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A simple identifier for accounts. For this prototype we use strings.
pub type AccountId = String;

/// Tokens supported by the L1 chain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Token {
    /// Native proof-of-stake currency used to pay fees, etc.
    Proof,
    /// Stablecoin with fixed supply controlled by an admin.
    FloweR,
}

/// A set of keys read / written by a transaction. For our simple payment
/// prototype the keys are just account identifiers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadWriteSet {
    pub reads: Vec<AccountId>,
    pub writes: Vec<AccountId>,
}

/// Placeholder for a QCT proof attached to each transaction. In a real
/// implementation this would be a structured proof object; here it's just
/// raw bytes produced by the QCT layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QCTProof(pub Vec<u8>);

/// Two basic kinds of actions supported by the chain.  Future extensions (e.g.
/// smart contract calls) would be added here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    /// Mint tokens (caller must be admin)
    Mint {
        to: AccountId,
        token: Token,
        amount: u64,
    },
    /// Transfer tokens between accounts
    Transfer {
        from: AccountId,
        to: AccountId,
        token: Token,
        amount: u64,
    },
}

/// A transaction envelope.  It includes the action, an explicit read/write set
/// (the key declarations required by FlowGraph), and an optional proof which
/// the node will verify before executing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub kind: TransactionKind,
    #[serde(default)]
    pub rw_set: ReadWriteSet,
    #[serde(default)]
    pub proof: Option<QCTProof>,
}

/// A block containing a batch of transactions and a simple height counter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub transactions: Vec<Transaction>,
}

/// Error type used by ledger operations
#[derive(thiserror::Error, Debug)]
pub enum LedgerError {
    #[error("account `{0}` does not exist")] 
    AccountNotFound(AccountId),
    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },
    #[error("transaction conflict detected")]
    Conflict,
    #[error("only admin can mint tokens")]
    UnauthorizedMint,
}
