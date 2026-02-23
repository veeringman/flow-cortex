use serde::{Deserialize, Serialize};

/// A simple identifier for accounts. For this prototype we use strings.
pub type AccountId = String;

/// Token identifier - now uses strings instead of enum to support dynamic token creation
/// Examples: "proof", "flower", "usdc", "usdt", etc.
pub type Token = String;

/// Token type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Native proof-of-stake currency (PROOF)
    Native,
    /// Stablecoin pegged to fiat (FLOWER, USDC, USDT, etc.)
    Stablecoin,
    /// Governance/voting token
    Governance,
    /// Utility/other purposes
    Utility,
}

/// Token operational status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenStatus {
    /// Token is active and can be transferred
    Active,
    /// Token transfers are frozen (emergency control)
    Frozen,
    /// New minting is paused
    Paused,
    /// Token is deprecated
    Deprecated,
}

/// Metadata about a token in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// Token symbol ("FLOWER", "PROOF", "USDC")
    pub symbol: String,
    /// Display name ("Flow Dollar", "Proof Token", "USD Coin")
    pub name: String,
    /// Decimal places for amounts (6 for stablecoins, 0 for native)
    pub decimals: u8,
    /// Total minted supply
    pub total_supply: u64,
    /// Creator/issuer account
    pub creator: AccountId,
    /// Token type classification
    pub token_type: TokenType,
    /// Current operational status
    pub status: TokenStatus,
    /// Block height when created
    pub created_at: u64,
    /// Additional metadata (JSON): backing, collateral, redemption rate, etc.
    pub metadata: Option<String>,
}

/// Events in the token system for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenEvent {
    /// Token was created
    Created {
        symbol: String,
        creator: AccountId,
        name: String,
        decimals: u8,
        block_height: u64,
    },
    /// Tokens were minted
    Minted {
        symbol: String,
        to: AccountId,
        amount: u64,
        block_height: u64,
    },
    /// Tokens were burned
    Burned {
        symbol: String,
        from: AccountId,
        amount: u64,
        block_height: u64,
    },
    /// Token transfers frozen
    Frozen {
        symbol: String,
        block_height: u64,
    },
    /// Token transfers unfrozen
    Unfrozen {
        symbol: String,
        block_height: u64,
    },
}

/// Bank account for settlement operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    /// Account identifier ("bank-a.institution.com")
    pub account_id: AccountId,
    /// Bank name for display
    pub bank_name: String,
    /// SWIFT code for identification
    pub swift_code: String,
    /// Whether the bank is approved to transact
    pub is_approved: bool,
    /// Block height when created
    pub created_at: u64,
    /// Daily mint limits per token: token symbol -> daily limit
    pub daily_mint_limits: std::collections::HashMap<Token, u64>,
    /// Daily minted amount tracking (resets daily): token symbol -> amount today
    pub daily_minted: std::collections::HashMap<Token, u64>,
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
/// Includes basic operations (mint/transfer), capsule operations, proof anchoring,
/// token management, and settlement operations for treasury/banking use cases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    Mint {
        to: AccountId,
        token: Token,
        amount: u64,
    },
    Transfer {
        from: AccountId,
        to: AccountId,
        token: Token,
        amount: u64,
    },
    UploadCapsule {
        id: String,
        code: Vec<u8>,
    },
    ExecuteCapsule {
        id: String,
        input: Vec<u8>,
    },
    AnchorProof {
        id: String,
        proof: Vec<u8>,
    },
    Trade {
        from: AccountId,
        to: AccountId,
        proof_amount: u64,
        flower_amount: u64,
    },
    CreateToken {
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: u64,
        token_type: TokenType,
        metadata: Option<String>,
    },
    Burn {
        token: Token,
        from: AccountId,
        amount: u64,
    },
    FreezeToken {
        token: Token,
    },
    UnfreezeToken {
        token: Token,
    },
    SettlementMint {
        token: Token,
        to: AccountId,
        amount: u64,
        reference: String,
        metadata: Option<String>,
    },
    SettlementBurn {
        token: Token,
        from: AccountId,
        amount: u64,
        reference: String,
        metadata: Option<String>,
    },
    SettlementTransfer {
        token: Token,
        from: AccountId,
        to: AccountId,
        amount: u64,
        reference: String,
        metadata: Option<String>,
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
    #[error("token `{0}` does not exist")]
    TokenNotFound(Token),
    #[error("token `{0}` already exists")]
    TokenAlreadyExists(Token),
    #[error("token `{0}` is frozen and cannot be transferred")]
    TokenFrozen(Token),
    #[error("token `{0}` minting is paused")]
    TokenMintingPaused(Token),
    #[error("bank `{0}` is not approved")]
    BankNotApproved(AccountId),
    #[error("daily mint limit exceeded for bank {bank}: limit {limit}, minted {minted}")]
    DailyLimitExceeded { bank: AccountId, limit: u64, minted: u64 },
    #[error("invalid token symbol `{0}`: must be non-empty")]
    InvalidTokenSymbol(String),
    #[error("invalid settlement reference `{0}`")]
    InvalidSettlementReference(String),
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

// ============================================================================
// DEMO: FortressDigital + ProofCortex Integration Data Models
// ============================================================================

/// Represents the verification status of a proof
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProofVerificationStatus {
    /// Proof has been submitted but not yet verified
    Pending,
    /// Proof has been cryptographically verified and bound to commitment
    Verified,
    /// Proof verification failed
    Failed,
}

/// Authorization commitment record anchored on FlowCortex
///
/// This immutable record represents the exact authorization context under which
/// a decision (e.g., treasury settlement approval) was made by FortressDigital.
///
/// Once anchored, a commitment cannot be modified or deleted, providing a
/// cryptographic audit trail for regulatory compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentRecord {
    /// Cryptographic hash of the authorization context (from FortressDigital)
    /// Format: hex-encoded string of H(user_id, device_trust, risk_score, policy_id, decision, txn_bucket, timestamp)
    pub commitment_hash: String,

    /// Policy identifier that governed this authorization (e.g., "treasury_settlement_v1")
    pub policy_id: String,

    /// Transaction reference from the originating system (e.g., "TXN-90877")
    pub txn_ref: String,

    /// Unix timestamp when commitment was generated (from FortressDigital)
    pub timestamp: u64,

    /// Block height on FlowCortex where this commitment was anchored
    /// Set when commitment is first persisted; immutable thereafter
    pub block_height: u64,

    /// Additional context reference for external audit trail lookup
    pub context_ref: Option<String>,

    /// Whether this commitment has been verified by a proof
    pub verified: bool,
}

/// Proof record linked to a commitment
///
/// This record represents a STARK zero-knowledge proof that validates
/// the authorization context captured in a CommitmentRecord.
///
/// The proof is cryptographically bound to the commitment hash to ensure
/// no replay attacks or commitment tampering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRecord {
    /// Hash of the anchored commitment this proof validates
    /// Must correspond to an existing CommitmentRecord
    pub commitment_hash: String,

    /// Cryptographic hash of the STARK proof submitted by ProofCortex
    /// Used for deduplication and proof reference
    pub proof_hash: String,

    /// Current verification status of this proof
    pub verification_status: ProofVerificationStatus,

    /// Block height on FlowCortex where proof was verified
    /// Set when verification succeeds; None if still pending/failed
    pub verification_block: Option<u64>,

    /// Version identifier of the Verifier Capsule used (e.g., "verifier_v1")
    pub verifier_capsule_version: String,

    /// Unix timestamp when proof was submitted
    pub submitted_at: u64,

    /// Public inputs supplied with the proof (JSON serialized)
    /// Example: {"policy_id": "treasury_settlement_v1", "txn_amount_bucket": "HIGH"}
    pub public_inputs: Option<String>,

    /// Error message if verification failed
    pub error_message: Option<String>,
}

/// Events emitted by FlowCortex during commitment and proof operations
/// These events drive UI updates and external system synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommitmentProofEvent {
    /// Commitment has been immutably anchored on FlowCortex
    CommitmentAnchored {
        commitment_hash: String,
        policy_id: String,
        txn_ref: String,
        block_height: u64,
        timestamp: u64,
    },

    /// STARK proof has been successfully verified and bound to commitment
    ProofVerified {
        commitment_hash: String,
        proof_hash: String,
        verification_block: u64,
        verified_at: u64,
        verifier_capsule_version: String,
    },

    /// Proof verification failed (cryptographic validation or binding error)
    ProofVerificationFailed {
        commitment_hash: String,
        proof_hash: String,
        error_reason: String,
        block_height: u64,
        failed_at: u64,
    },

    /// Proof submission was made for a missing commitment (error condition)
    CommitmentNotFound {
        commitment_hash: String,
        proof_hash: String,
        submitted_at: u64,
    },

    /// Proof format or structure was invalid
    InvalidProofFormat {
        error_description: String,
        submitted_at: u64,
    },

    /// Proof already exists for this commitment (duplicate submission)
    DuplicateProof {
        commitment_hash: String,
        proof_hash: String,
        previous_verification_status: ProofVerificationStatus,
    },
}
