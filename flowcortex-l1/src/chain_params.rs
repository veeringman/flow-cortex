/// FlowCortex L1 — Canonical Chain Parameters
///
/// This module is the single source of truth for all chain identity, timing,
/// asset, signing, and proof-coordination parameters.  Every service that
/// integrates with FlowCortex L1 MUST derive its configuration from these
/// constants rather than hard-coding values.
///
/// Companion document (consumer view):
///   KeyCortex/FlowCortex_L1_Chain_Parameters_ProofCortex_Expectation.md

// ─── Chain Identity ────────────────────────────────────────────────────────

/// Human-readable canonical chain identifier.
pub const CHAIN_SLUG: &str = "flowcortex-l1";

/// Numeric chain ID — used for tx domain separation and replay protection.
/// 1337 = devnet convention; assign a unique value per network at launch.
pub const CHAIN_ID_NUMERIC: u64 = 1337;

/// Network ID (matches chain ID for single-network devnet).
pub const NETWORK_ID: u64 = 1337;

/// Deterministic genesis block hash.
/// Computed as SHA-256("flowcortex-l1:devnet:genesis:v1") — see scripts/gen_genesis.sh.
pub const GENESIS_HASH: &str =
    "a6e2b404caa93426a8f608aa8e633d63f6a5b1d44772d59fc230bb505bdbb4ff";

/// Active protocol/upgrade epoch version.
pub const PROTOCOL_VERSION: &str = "v1";

/// Address scheme: free-form UTF-8 string identifiers (no on-chain checksum in MVP).
/// Format: lowercase alphanumeric + hyphens/underscores, max 64 chars.
pub const ADDRESS_SCHEME: &str = "fc-string-v1";

/// Signature scheme used for wallet key operations.
pub const SIGNATURE_SCHEME: &str = "ed25519";

// ─── Transaction Domain & Signing ─────────────────────────────────────────

/// Domain tag prepended to transaction signing payloads.
/// Prevents cross-domain replay between tx/auth/proof contexts.
pub const TX_DOMAIN_TAG: &str = "flowcortex:tx:v1";

/// Domain tag prepended to authentication challenge payloads.
pub const AUTH_DOMAIN_TAG: &str = "flowcortex:auth:v1";

/// Domain tag prepended to proof/commitment verification inputs.
pub const PROOF_DOMAIN_TAG: &str = "flowcortex:proof:v1";

/// Hash algorithm for transaction IDs and commitment hashes.
pub const TX_HASH_ALGORITHM: &str = "sha256";

/// Canonical serialization standard for deterministic hashing.
/// Keys sorted lexicographically, no whitespace (RFC 8785 JSON Canonicalization).
pub const CANONICAL_SERIALIZATION: &str = "json-canonical-rfc8785";

/// Nonce model: idempotency guaranteed by commitment_hash key uniqueness.
/// Duplicate commitment_hash → "idempotent" acknowledgment, no double-execution.
pub const NONCE_MODEL: &str = "commitment-hash-idempotent";

/// Fee model: no transaction fees in MVP.
pub const FEE_MODEL: &str = "none-mvp";

/// Finality rule: deterministic single-node — every committed block is final.
/// Downstream services may treat 1-block confirmation as final.
pub const FINALITY_RULE: &str = "single-node-immediate-1-block";

// ─── Block Production ─────────────────────────────────────────────────────

/// Default block production interval in milliseconds.
/// Override with BLOCK_INTERVAL_MS env var.
pub const BLOCK_INTERVAL_MS: u64 = 1_000; // 1 second

/// Number of blocks per epoch (used for future validator rotation / reward accounting).
pub const EPOCH_LENGTH_BLOCKS: u64 = 100;

// ─── Assets ───────────────────────────────────────────────────────────────

// ── PROOF (native coin of FlowCortex Network) ──

pub const PROOF_SYMBOL: &str = "PROOF";
pub const PROOF_NAME: &str = "PROOF";
/// Decimal precision: 6 (amounts stored as micro-PROOF, 1 PROOF = 1_000_000 units).
pub const PROOF_DECIMALS: u8 = 6;
/// Total supply minted at TGE (Token Generation Event): 10,000,000,000 PROOF.
/// In micro-PROOF units (6 decimals): 20_000_000_000 * 1_000_000.
pub const PROOF_TGE_SUPPLY: u64 = 20_000_000_000_000_000;
/// Minimum indivisible transfer unit (1 micro-PROOF).
pub const PROOF_MIN_TRANSFER_UNIT: u64 = 1;
/// PROOF is not used for fee payment in MVP (no fee model).
pub const PROOF_FEE_PAYMENT_SUPPORT: bool = false;
pub const PROOF_TYPE: &str = "native";

// ── FloweR (stablecoin) ──

pub const FLOWER_SYMBOL: &str = "FloweR";
pub const FLOWER_NAME: &str = "Flow Rupee";
/// Decimal precision: 6 (amounts stored as micro-FloweR, 1 FloweR = 1_000_000 units).
pub const FLOWER_DECIMALS: u8 = 6;
/// Minimum indivisible transfer unit (1 micro-FloweR = ₹0.000001).
pub const FLOWER_MIN_TRANSFER_UNIT: u64 = 1;
/// FloweR is a native INR-pegged stablecoin on FlowCortex L1 (minted on demand, 1 FloweR = ₹1).
pub const FLOWER_TYPE: &str = "native-stablecoin";
/// Minting authority: only approved settlement banks via /bank/approve API.
/// Each mint requires an equal INR reserve backing.
pub const FLOWER_MINT_BURN_AUTHORITY: &str = "settlement-banks-only";
/// Pause/freeze supported: yes (freeze_token / unfreeze_token in Ledger).
pub const FLOWER_PAUSE_FREEZE_SUPPORTED: bool = true;

// ─── Endpoints (devnet defaults, override via env vars) ───────────────────

/// Default HTTP REST endpoint for read/write operations.
pub const DEFAULT_HTTP_ENDPOINT: &str = "http://192.168.29.78:8082";

/// Default gRPC endpoint for streaming and high-throughput clients.
pub const DEFAULT_GRPC_ENDPOINT: &str = "http://192.168.29.78:50051";

/// Explorer UI default endpoint.
pub const DEFAULT_EXPLORER_ENDPOINT: &str = "http://192.168.29.78:8082";

// ─── Proof Coordination (FlowCortex ↔ ProofCortex alignment) ─────────────

/// Versioned schema identifier for proof input payloads.
/// Must match `PC_FLOWCORTEX_CAPSULE_VERSION` in ProofCortex config.
pub const PROOF_INPUT_SCHEMA_VERSION: &str = "proofcortex-mvp-v1";

/// Commitment hash formula.
/// commitment_hash = sha256(json_canonical(witness_fields))
/// where witness_fields = { user_id, device_id, auth_level, risk_score,
///                          device_trust, policy_id, challenge_id, timestamp_ms }
pub const COMMITMENT_HASH_RULE: &str = "sha256(json_canonical(witness_fields))";

/// Capsule version identifier used in FlowCortex proof verification endpoint.
pub const CAPSULE_VERSION: &str = "verifier_v1";

/// Clock skew tolerance for proof attestation timestamps (milliseconds).
pub const ATTESTATION_TIMESTAMP_TOLERANCE_MS: u64 = 10_000; // 10 seconds

/// Deterministic proof failure reason codes returned by FlowCortex.
/// These map to HTTP 422 sub-codes in the /api/v1/proofs/verify response.
pub const PROOF_FAILURE_CODE_COMMITMENT_MISMATCH: &str = "commitment_hash_mismatch";
pub const PROOF_FAILURE_CODE_SCHEMA_VERSION: &str = "unsupported_schema_version";
pub const PROOF_FAILURE_CODE_EXPIRED: &str = "attestation_timestamp_expired";
pub const PROOF_FAILURE_CODE_INVALID_CAPSULE: &str = "invalid_capsule_version";

// ─── Environment Rate Limits (devnet) ─────────────────────────────────────

/// Maximum requests per second to HTTP endpoints (soft limit, no enforcement in MVP).
pub const RATE_LIMIT_QPS: u32 = 100;

/// Maximum burst size above QPS limit.
pub const RATE_LIMIT_BURST: u32 = 200;
