use crate::node::Node;
use crate::types::{AccountId, Token, TokenMetadata, TokenType, Transaction, TransactionKind, ReadWriteSet, QCTProof, CommitmentRecord, ProofRecord, CommitmentProofEvent};
use crate::demo::{DemoSettlementConfig, DemoSettlementScenario};
use axum::{
    extract::{Extension, Json, Path, Query as AxumQuery},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum::middleware::{self, Next};
use axum::http::{Request, HeaderValue};
use axum::body::Body;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use base64::Engine;

/// Shared application state for handlers
pub type SharedNode = Arc<Mutex<Node>>;

#[derive(Deserialize)]
struct AccountRequest {
    account: AccountId,
}

#[derive(Deserialize)]
struct MintRequest {
    caller: AccountId,
    to: AccountId,
    token: Token,
    amount: u64,
    #[serde(default)]
    rw_set: ReadWriteSet,
    #[serde(default)]
    proof: Option<QCTProof>,
}

#[derive(Deserialize)]
struct TransferRequest {
    from: AccountId,
    to: AccountId,
    token: Token,
    amount: u64,
    #[serde(default)]
    rw_set: ReadWriteSet,
    #[serde(default)]
    proof: Option<QCTProof>,
}

#[derive(Serialize)]
struct BalanceResponse {
    account: AccountId,
    token: Token,
    balance: u64,
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    symbol: String,
    name: String,
    decimals: u8,
    initial_supply: u64,
    token_type: String,
    #[serde(default)]
    metadata_json: String,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    success: bool,
    symbol: String,
    error: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct BlockResponse {
    height: u64,
    transactions: Vec<crate::types::Transaction>,
}

#[derive(Serialize)]
struct PoolResponse {
    pending: Vec<crate::types::Transaction>,
}

#[derive(Serialize)]
struct SnapshotResponse {
    root: String,
}

#[derive(Deserialize)]
struct CapsuleUploadRequest {
    id: String,
    /// Base64-encoded WASM module
    code: String,
}

#[derive(Deserialize)]
struct CapsuleInvokeRequest {
    /// opaque input bytes, base64 encoded
    input: String,
}

#[derive(Serialize)]
struct CapsuleListResponse {
    capsules: Vec<String>,
}

#[derive(Serialize)]
struct CapsuleInvokeResponse {
    output: String,
}

// ============================================================================
// Commitment & Proof API Request/Response Types
// ============================================================================

#[derive(Deserialize)]
struct AnchorCommitmentRequest {
    commitment_hash: String,
    policy_id: String,
    txn_ref: String,
    timestamp: u64,
    #[serde(default)]
    context_ref: Option<String>,
}

#[derive(Serialize)]
struct AnchorCommitmentResponse {
    success: bool,
    commitment_hash: String,
    block_height: u64,
    tx_hash: String,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize)]
struct VerifyProofRequest {
    commitment_hash: String,
    proof_hash: String,
    proof_data: String,  // Base64 encoded
    proof_type: String,
    #[serde(default)]
    capsule_version: Option<String>,
}

#[derive(Serialize)]
struct VerifyProofResponse {
    success: bool,
    commitment_hash: String,
    proof_hash: String,
    verified: bool,
    block_height: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct CommitmentResponse {
    commitment_hash: String,
    policy_id: String,
    txn_ref: String,
    timestamp: u64,
    block_height: u64,
    verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_ref: Option<String>,
}

#[derive(Serialize)]
struct ProofStatusResponse {
    commitment_hash: String,
    verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof: Option<ProofInfo>,
}

#[derive(Serialize)]
struct ProofInfo {
    proof_hash: String,
    verification_block: u64,
    verified_at: u64,
    verifier_capsule_version: String,
}

#[derive(Deserialize)]
struct EventsQuery {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    commitment_hash: Option<String>,
}

#[derive(Serialize)]
struct EventsResponse {
    events: Vec<EventInfo>,
    count: usize,
}

#[derive(Serialize)]
struct EventInfo {
    event_type: String,
    commitment_hash: String,
    block_height: u64,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verified: Option<bool>,
}

#[derive(Serialize)]
struct DashboardStatsResponse {
    total_commitments: usize,
    total_proofs: usize,
    verified_proofs: usize,
    pending_proofs: usize,
    total_events: usize,
    current_block_height: u64,
}

// ============================================================================
// Original Handler Functions
// ============================================================================

async fn create_account(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<AccountRequest>,
) -> impl IntoResponse {
    let mut n = node.lock().unwrap();
    n.create_account(&req.account);
    StatusCode::CREATED
}

async fn mint(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<MintRequest>,
) -> impl IntoResponse {
    let mut n = node.lock().unwrap();
    let mut proof = req.proof.clone();
    if proof.is_none() {
        proof = Some(crate::qct::prove(&req.rw_set));
    }
    let tx = Transaction {
        kind: TransactionKind::Mint {
            to: req.to.clone(),
            token: req.token.clone(),
            amount: req.amount,
        },
        rw_set: req.rw_set.clone(),
        proof,
    };
    let result = n.apply_transaction(&req.caller, tx);
    if result.is_ok() {
        let _ = n.save("node_state.json");
    }
    match result {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            let body: Json<ErrorResponse> = Json(ErrorResponse { error: e.to_string() });
            (StatusCode::BAD_REQUEST, body).into_response()
        }
    }
}

async fn transfer(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<TransferRequest>,
) -> impl IntoResponse {
    let mut n = node.lock().unwrap();
    let mut proof = req.proof.clone();
    if proof.is_none() {
        proof = Some(crate::qct::prove(&req.rw_set));
    }
    let tx = Transaction {
        kind: TransactionKind::Transfer {
            from: req.from.clone(),
            to: req.to.clone(),
            token: req.token.clone(),
            amount: req.amount,
        },
        rw_set: req.rw_set.clone(),
        proof,
    };
    let result = n.submit_transaction(&req.from, tx);
    if result.is_ok() {
        let _ = n.save("node_state.json");
    }
    match result {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            let body: Json<ErrorResponse> = Json(ErrorResponse { error: e.to_string() });
            (StatusCode::BAD_REQUEST, body).into_response()
        }
    }
}

async fn balance(
    Path((acct, token_str)): Path<(AccountId, String)>,
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let token = token_str.to_lowercase();
    let n = node.lock().unwrap();
    let bal = n.balance(&acct, &token);
    Json(BalanceResponse { account: acct, token, balance: bal }).into_response()
}

async fn create_token(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let token_type = match req.token_type.to_lowercase().as_str() {
        "native" => TokenType::Native,
        "stablecoin" => TokenType::Stablecoin,
        "governance" => TokenType::Governance,
        "utility" => TokenType::Utility,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: "invalid token_type".to_string() }),
            )
                .into_response();
        }
    };

    let mut n = node.lock().unwrap();
    let admin = n.admin.clone();
    let metadata = if req.metadata_json.trim().is_empty() {
        None
    } else {
        Some(req.metadata_json)
    };

    let symbol = req.symbol.clone();
    match n.ledger.create_token(
        &admin,
        req.symbol,
        req.name,
        req.decimals,
        req.initial_supply,
        token_type,
        metadata,
    ) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(CreateTokenResponse {
                success: true,
                symbol,
                error: String::new(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(CreateTokenResponse {
                success: false,
                symbol,
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn list_tokens(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let n = node.lock().unwrap();
    let tokens: Vec<TokenMetadata> = n.ledger.list_tokens().into_iter().cloned().collect();
    Json(tokens)
}

async fn get_token(
    Path(symbol): Path<String>,
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let n = node.lock().unwrap();
    let token = symbol.to_lowercase();
    match n.ledger.get_token(&token) {
        Some(meta) => Json(meta).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: format!("token not found: {}", token) }),
        )
            .into_response(),
    }
}

async fn get_pool(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let n = node.lock().unwrap();
    Json(PoolResponse {
        pending: n.pending_pool(),
    })
}

#[derive(Deserialize)]
struct TxRequest {
    caller: String,
    pubkey: Vec<u8>,
    signature: Vec<u8>,
    tx: crate::types::Transaction,
}

async fn submit_tx(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<TxRequest>,
) -> impl IntoResponse {
    let stx = crate::types::SignedTransaction {
        caller: req.caller,
        pubkey: req.pubkey,
        signature: req.signature,
        tx: req.tx,
    };
    let mut n = node.lock().unwrap();
    match n.submit_signed_transaction(stx) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response(),
    }
}

async fn list_anchors(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let n = node.lock().unwrap();
    let ids: Vec<String> = n.anchors.keys().cloned().collect();
    Json(serde_json::json!({"anchors": ids}))
}

async fn get_anchor(
    Path(id): Path<String>,
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let n = node.lock().unwrap();
    let resp = if let Some(data) = n.anchors.get(&id) {
        let val = serde_json::json!({"id": id, "proof": base64::engine::general_purpose::STANDARD.encode(data)});
        Json(val).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "not found".into() })).into_response()
    };
    resp
}

async fn create_block(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let mut n = node.lock().unwrap();
    let block = n.create_block();
    let _ = n.save("node_state.json");
    Json(BlockResponse {
        height: block.height,
        transactions: block.transactions,
    })
}

async fn list_blocks(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let n = node.lock().unwrap();
    let resp: Vec<BlockResponse> = n
        .blocks
        .iter()
        .map(|b| BlockResponse {
            height: b.height,
            transactions: b.transactions.clone(),
        })
        .collect();
    Json(resp)
}

async fn snapshot(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let n = node.lock().unwrap();
    Json(SnapshotResponse { root: hex::encode(&n.snapshot_root) })
}

async fn upload_capsule(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<CapsuleUploadRequest>,
) -> impl IntoResponse {
    // decode using the new Engine API
    use base64::engine::general_purpose::STANDARD;
    let code = match STANDARD.decode(&req.code) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }
    };
    let mut n = node.lock().unwrap();
    match n.store_capsule(&req.id, code) {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
        )
            .into_response(),
    }
}

async fn list_capsules(Extension(node): Extension<SharedNode>) -> impl IntoResponse {
    let n = node.lock().unwrap();
    let ids: Vec<String> = n.capsules.keys().cloned().collect();
    Json(CapsuleListResponse { capsules: ids })
}

async fn invoke_capsule(
    Path(id): Path<String>,
    Extension(node): Extension<SharedNode>,
    Json(req): Json<CapsuleInvokeRequest>,
) -> impl IntoResponse {
    use base64::engine::general_purpose::STANDARD;
    let input = match STANDARD.decode(&req.input) {
        Ok(i) => i,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e.to_string() }),
            )
                .into_response();
        }
    };
    let n = node.lock().unwrap();
    match n.execute_capsule(&id, &input) {
        Ok(output) => {
            let encoded = STANDARD.encode(&output);
            Json(CapsuleInvokeResponse { output: encoded }).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}

// ============================================================================
// Commitment & Proof Handlers
// ============================================================================

async fn anchor_commitment(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<AnchorCommitmentRequest>,
) -> impl IntoResponse {
    let mut n = node.lock().unwrap();
    
    match n.ledger.anchor_commitment(
        req.commitment_hash.clone(),
        req.policy_id.clone(),
        req.txn_ref.clone(),
        req.timestamp,
        req.context_ref.clone(),
    ) {
        Ok((block_height, tx_hash)) => {
            Json(AnchorCommitmentResponse {
                success: true,
                commitment_hash: req.commitment_hash,
                block_height,
                tx_hash,
                timestamp: req.timestamp,
                error: None,
            }).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(AnchorCommitmentResponse {
                success: false,
                commitment_hash: req.commitment_hash,
                block_height: 0,
                tx_hash: String::new(),
                timestamp: req.timestamp,
                error: Some(format!("{:?}", e)),
            })).into_response()
        }
    }
}

async fn verify_proof(
    Extension(node): Extension<SharedNode>,
    Json(req): Json<VerifyProofRequest>,
) -> impl IntoResponse {
    let mut n = node.lock().unwrap();
    
    // Decode base64 proof data
    let proof_data = match base64::engine::general_purpose::STANDARD.decode(&req.proof_data) {
        Ok(data) => data,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(VerifyProofResponse {
                success: false,
                commitment_hash: req.commitment_hash,
                proof_hash: req.proof_hash,
                verified: false,
                block_height: 0,
                error: Some("Invalid base64 proof data".to_string()),
            })).into_response();
        }
    };
    
    let capsule_version = req.capsule_version.clone().unwrap_or_else(|| "verifier_v1".to_string());
    match n.ledger.verify_proof(
        req.commitment_hash.clone(),
        req.proof_hash.clone(),
        proof_data,
        req.proof_type.clone(),
        None,
        capsule_version,
    ) {
        Ok(record) => {
            let verified = matches!(record.verification_status, crate::types::ProofVerificationStatus::Verified);
            Json(VerifyProofResponse {
                success: true,
                commitment_hash: record.commitment_hash,
                proof_hash: record.proof_hash,
                verified,
                block_height: record.verification_block.unwrap_or(0),
                error: None,
            }).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, Json(VerifyProofResponse {
                success: false,
                commitment_hash: req.commitment_hash,
                proof_hash: req.proof_hash,
                verified: false,
                block_height: 0,
                error: Some(format!("{:?}", e)),
            })).into_response()
        }
    }
}

async fn query_commitment(
    Path(hash): Path<String>,
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let n = node.lock().unwrap();
    
    match n.ledger.query_commitment(&hash) {
        Some(record) => {
            Json(CommitmentResponse {
                commitment_hash: record.commitment_hash,
                policy_id: record.policy_id,
                txn_ref: record.txn_ref,
                timestamp: record.timestamp,
                block_height: record.block_height,
                verified: record.verified,
                context_ref: record.context_ref,
            }).into_response()
        }
        None => {
            (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: "Commitment not found".to_string(),
            })).into_response()
        }
    }
}

async fn query_proof_status(
    Path(hash): Path<String>,
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let n = node.lock().unwrap();
    
    match n.ledger.query_proof_status(&hash) {
        Some((proof_opt, verified)) => {
            let proof_info = proof_opt.map(|p| ProofInfo {
                proof_hash: p.proof_hash,
                verification_block: p.verification_block.unwrap_or(0),
                verified_at: p.submitted_at,
                verifier_capsule_version: p.verifier_capsule_version,
            });
            Json(ProofStatusResponse {
                commitment_hash: hash,
                verified,
                proof: proof_info,
            }).into_response()
        }
        None => {
            (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: "Commitment not found".to_string(),
            })).into_response()
        }
    }
}

async fn list_events(
    AxumQuery(query): AxumQuery<EventsQuery>,
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let n = node.lock().unwrap();
    
    let events: Vec<EventInfo> = n.ledger.query_events(
        query.commitment_hash.as_deref(),
        None,
        query.limit.unwrap_or(100),
        0,
    ).iter().map(|e| match e {
        CommitmentProofEvent::CommitmentAnchored { commitment_hash, block_height, timestamp, .. } => {
            EventInfo {
                event_type: "commitment.anchored".to_string(),
                commitment_hash: commitment_hash.clone(),
                block_height: *block_height,
                timestamp: *timestamp,
                proof_hash: None,
                verified: None,
            }
        }
        CommitmentProofEvent::ProofVerified { commitment_hash, proof_hash, verification_block, verified_at, .. } => {
            EventInfo {
                event_type: "proof.verified".to_string(),
                commitment_hash: commitment_hash.clone(),
                block_height: *verification_block,
                timestamp: *verified_at,
                proof_hash: Some(proof_hash.clone()),
                verified: Some(true),
            }
        }
        CommitmentProofEvent::ProofVerificationFailed { commitment_hash, proof_hash, block_height, failed_at, .. } => {
            EventInfo {
                event_type: "proof.failed".to_string(),
                commitment_hash: commitment_hash.clone(),
                block_height: *block_height,
                timestamp: *failed_at,
                proof_hash: Some(proof_hash.clone()),
                verified: Some(false),
            }
        }
        CommitmentProofEvent::CommitmentNotFound { commitment_hash, proof_hash, submitted_at } => {
            EventInfo {
                event_type: "commitment.missing".to_string(),
                commitment_hash: commitment_hash.clone(),
                block_height: 0,
                timestamp: *submitted_at,
                proof_hash: Some(proof_hash.clone()),
                verified: None,
            }
        }
        CommitmentProofEvent::InvalidProofFormat { error_description: _, submitted_at } => {
            EventInfo {
                event_type: "proof.invalid".to_string(),
                commitment_hash: "".to_string(),
                block_height: 0,
                timestamp: *submitted_at,
                proof_hash: None,
                verified: None,
            }
        }
        CommitmentProofEvent::DuplicateProof { commitment_hash, proof_hash, .. } => {
            EventInfo {
                event_type: "proof.duplicate".to_string(),
                commitment_hash: commitment_hash.clone(),
                block_height: 0,
                timestamp: 0,
                proof_hash: Some(proof_hash.clone()),
                verified: None,
            }
        }
        _ => {
            EventInfo {
                event_type: "unknown".to_string(),
                commitment_hash: "".to_string(),
                block_height: 0,
                timestamp: 0,
                proof_hash: None,
                verified: None,
            }
        }
    }).collect();
    
    Json(EventsResponse {
        count: events.len(),
        events,
    })
}

async fn dashboard_stats(
    Extension(node): Extension<SharedNode>,
) -> impl IntoResponse {
    let n = node.lock().unwrap();
    
    let stats = n.ledger.get_stats();
    
    Json(DashboardStatsResponse {
        total_commitments: stats.total_commitments,
        total_proofs: stats.total_proofs,
        verified_proofs: stats.verified_proofs,
        pending_proofs: stats.pending_proofs,
        total_events: stats.total_events,
        current_block_height: n.ledger.block_height,
    })
}

/// Build the router for the RPC server.
pub fn make_router(node: SharedNode) -> Router {
    // middleware to inject permissive CORS headers
    async fn cors(req: Request<Body>, next: Next) -> Response {
        // Handle preflight OPTIONS requests
        if req.method() == axum::http::Method::OPTIONS {
            return Response::builder()
                .status(StatusCode::OK)
                .header("access-control-allow-origin", "*")
                .header("access-control-allow-methods", "*")
                .header("access-control-allow-headers", "*")
                .header("access-control-max-age", "3600")
                .body(Body::empty())
                .unwrap();
        }
        
        let mut res = next.run(req).await;
        let headers = res.headers_mut();
        headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
        headers.insert("access-control-allow-methods", HeaderValue::from_static("*"));
        headers.insert("access-control-allow-headers", HeaderValue::from_static("*"));
        res
    }

    Router::new()
        .route("/account", post(create_account))
        .route("/mint", post(mint))
        .route("/transfer", post(transfer))
        .route("/balance/{account}/{token}", get(balance))
        .route("/token/create", post(create_token))
        .route("/tokens", get(list_tokens))
        .route("/token/{symbol}", get(get_token))
        .route("/pool", get(get_pool))
        .route("/block", post(create_block))
        .route("/blocks", get(list_blocks))
        .route("/snapshot", get(snapshot))
        // generic transaction submission
        .route("/tx", post(submit_tx))
        // anchor queries (legacy)
        .route("/anchors", get(list_anchors))
        .route("/anchor/{id}", get(get_anchor))
        // capsule management
        .route("/capsule", post(upload_capsule))
        .route("/capsule", get(list_capsules))
        .route("/capsule/{id}/invoke", post(invoke_capsule))
        // commitment & proof APIs
        .route("/api/anchor_commitment", post(anchor_commitment))
        .route("/api/verify_proof", post(verify_proof))
        .route("/api/commitment/{hash}", get(query_commitment))
        .route("/api/proof_status/{hash}", get(query_proof_status))
        .route("/api/events", get(list_events))
        .route("/api/stats", get(dashboard_stats))
        .layer(middleware::from_fn(cors))
        .layer(Extension(node))
}
