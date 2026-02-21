use serde::{Deserialize, Serialize};

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

/// A transaction type indicating what state transition should occur.
/// Previously we only supported mint/transfer; we now add capsule-related
/// actions so that the ledger/history can record uploads and executes as part
/// of the chain. Additional variants support proof anchoring and future
/// stablecoin/exchange operations.
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
    /// Store a WASM capsule in the node's registry
    UploadCapsule {
        id: String,
        /// binary code of the module; serialized as base64 by serde
        code: Vec<u8>,
    },
    /// Execute a stored capsule; `input` is opaque bytes passed to the module
    ExecuteCapsule {
        id: String,
        input: Vec<u8>,
    },
    /// Anchor a cryptographic or ZKP proof on the chain. The `id` can be used
    /// later to query or verify inclusion.
    AnchorProof {
        id: String,
        proof: Vec<u8>,
    },
    /// Placeholder for future trade/buy‑sell operations between Proof and
    /// FloweR tokens; smart contract logic may be provided by capsules.
    Trade {
        from: AccountId,
        to: AccountId,
        proof_amount: u64,
        flower_amount: u64,
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
    #[error("capsule error: {0}")]
    CapsuleError(String),
    #[error("invalid transaction signature")]
    InvalidSignature,
}

// allow converting a string into a LedgerError for convenience
impl From<String> for LedgerError {
    fn from(s: String) -> Self {
        LedgerError::CapsuleError(s)
    }
}

/// Public key type for wallet authentication (raw bytes of an ed25519 key).
pub type PubKey = Vec<u8>;

/// Marshalled transaction together with an explicit signature.  The ledger
/// verifies that the signature is valid for the `caller` and corresponding
/// public key before applying the transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub caller: AccountId,
    pub pubkey: PubKey,
    pub signature: Vec<u8>,
    pub tx: Transaction,
}
