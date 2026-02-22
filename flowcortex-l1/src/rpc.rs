use crate::node::Node;
use crate::types::{AccountId, Token, Transaction, TransactionKind, ReadWriteSet, QCTProof};
use axum::{
    extract::{Extension, Json, Path},
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
        .route("/pool", get(get_pool))
        .route("/block", post(create_block))
        .route("/blocks", get(list_blocks))
        .route("/snapshot", get(snapshot))
        // generic transaction submission
        .route("/tx", post(submit_tx))
        // anchor queries
        .route("/anchors", get(list_anchors))
        .route("/anchor/{id}", get(get_anchor))
        // capsule management
        .route("/capsule", post(upload_capsule))
        .route("/capsule", get(list_capsules))
        .route("/capsule/{id}/invoke", post(invoke_capsule))
        .layer(middleware::from_fn(cors))
        .layer(Extension(node))
}
