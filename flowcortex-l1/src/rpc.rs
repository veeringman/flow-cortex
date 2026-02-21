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
    let token = match token_str.as_str() {
        "Proof" | "proof" => Token::Proof,
        "FloweR" | "flower" | "flowr" => Token::FloweR,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("unknown token `{}`", other),
                }),
            )
                .into_response();
        }
    };
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

/// Build the router for the RPC server.
pub fn make_router(node: SharedNode) -> Router {
    // middleware to inject permissive CORS headers
    async fn cors(req: Request<Body>, next: Next) -> Response {
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
        .layer(middleware::from_fn(cors))
        .layer(Extension(node))
}
