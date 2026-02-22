use crate::rpc::SharedNode;
use crate::types::{Token, SignedTransaction};
use tonic::{Request, Response, Status};
use hex;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub mod proto {
    tonic::include_proto!("l1");
}

use proto::l1_server::{L1, L1Server};
use proto::*;

#[derive(Clone)]
pub struct L1Service {
    node: SharedNode,
}

#[tonic::async_trait]
impl L1 for L1Service {
    async fn get_balance(
        &self,
        req: Request<BalanceRequest>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let req = req.into_inner();
        let token = match req.token.as_str() {
            "Proof" | "proof" => Token::Proof,
            "FloweR" | "flower" | "flowr" => Token::FloweR,
            other => return Err(Status::invalid_argument(format!("unknown token {}", other))),
        };
        let n = self.node.lock().unwrap();
        let bal = n.balance(&req.account, &token);
        Ok(Response::new(BalanceResponse {
            account: req.account,
            token: format!("{:?}", token),
            balance: bal,
        }))
    }

    async fn submit_tx(
        &self,
        req: Request<TxRequest>,
    ) -> Result<Response<TxResponse>, Status> {
        let r = req.into_inner();
        let stx = SignedTransaction {
            caller: r.caller,
            pubkey: r.pubkey,
            signature: r.signature,
            tx: serde_json::from_str(&r.tx_json).map_err(|e| Status::invalid_argument(e.to_string()))?,
        };
        let mut n = self.node.lock().unwrap();
        match n.submit_signed_transaction(stx) {
            Ok(()) => Ok(Response::new(TxResponse { success: true, error: "".into() })),
            Err(e) => Ok(Response::new(TxResponse { success: false, error: e.to_string() })),
        }
    }

    async fn list_pool(&self, _req: Request<Empty>) -> Result<Response<PoolResponse>, Status> {
        let n = self.node.lock().unwrap();
        let json = serde_json::to_string(&n.pending_pool()).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PoolResponse { pending_json: json }))
    }

    async fn list_blocks(&self, _req: Request<Empty>) -> Result<Response<BlocksResponse>, Status> {
        let n = self.node.lock().unwrap();
        let mut resp = BlocksResponse { blocks: vec![] };
        for b in &n.blocks {
            let tx_json = serde_json::to_string(&b.transactions).map_err(|e| Status::internal(e.to_string()))?;
            resp.blocks.push(Block { height: b.height, txs_json: tx_json });
        }
        Ok(Response::new(resp))
    }

    async fn snapshot(&self, _req: Request<Empty>) -> Result<Response<SnapshotResponse>, Status> {
        let n = self.node.lock().unwrap();
        Ok(Response::new(SnapshotResponse { root: hex::encode(&n.snapshot_root) }))
    }

    async fn list_anchors(&self, _req: Request<Empty>) -> Result<Response<AnchorListResponse>, Status> {
        let n = self.node.lock().unwrap();
        Ok(Response::new(AnchorListResponse { ids: n.anchors.keys().cloned().collect() }))
    }

    async fn get_anchor(&self, req: Request<AnchorRequest>) -> Result<Response<AnchorResponse>, Status> {
        let id = req.into_inner().id;
        let n = self.node.lock().unwrap();
        if let Some(data) = n.anchors.get(&id) {
            Ok(Response::new(AnchorResponse { id, proof_base64: STANDARD.encode(data), error: "".into() }))
        } else {
            Ok(Response::new(AnchorResponse { id, proof_base64: "".into(), error: "not found".into() }))
        }
    }
}

/// start the gRPC server on given address
pub async fn serve_grpc(node: SharedNode, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let svc = L1Service { node };
    tonic::transport::Server::builder()
        .add_service(L1Server::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
