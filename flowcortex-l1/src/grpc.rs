use crate::rpc::SharedNode;
use crate::types::{SignedTransaction, TokenType};
use tonic::{Request, Response, Status};
use hex;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub mod proto {
    tonic::include_proto!("l1");
}

use proto::l1_server::{L1, L1Server};
use proto::tokens_server::{Tokens, TokensServer};
use proto::settlement_server::{Settlement, SettlementServer};
use proto::admin_server::{Admin, AdminServer};
use proto::commitment_anchor_server::{CommitmentAnchor, CommitmentAnchorServer};
use proto::proof_verifier_server::{ProofVerifier, ProofVerifierServer};
use proto::*;

mod tokens;
mod settlement;
mod admin;
mod demo;
mod commitment;
mod proof;

use tokens::TokensService;
use settlement::SettlementService;
use admin::AdminService;
use demo::DemoService;
use commitment::CommitmentAnchorService;
use proof::ProofVerifierService;

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
        let token = req.token.to_lowercase();
        let n = self.node.lock().unwrap();
        let bal = n.balance(&req.account, &token);
        Ok(Response::new(BalanceResponse {
            account: req.account,
            token: token,
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

#[tonic::async_trait]
impl Tokens for TokensService {
    async fn create_token(
        &self,
        req: Request<CreateTokenRequest>,
    ) -> Result<Response<CreateTokenResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();
        let admin = node.admin.clone();

        let token_type = match req.token_type.to_lowercase().as_str() {
            "native" => TokenType::Native,
            "stablecoin" => TokenType::Stablecoin,
            "governance" => TokenType::Governance,
            "utility" => TokenType::Utility,
            _ => return Ok(Response::new(CreateTokenResponse {
                success: false,
                symbol: req.symbol,
                error: "invalid token_type".to_string(),
            })),
        };

        let metadata = req.metadata_json;

        match node.ledger.create_token(
            &admin,
            req.symbol.clone(),
            req.name,
            req.decimals as u8,
            req.initial_supply,
            token_type,
            if metadata.is_empty() { None } else { Some(metadata) },
        ) {
            Ok(_) => Ok(Response::new(CreateTokenResponse {
                success: true,
                symbol: req.symbol,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CreateTokenResponse {
                success: false,
                symbol: req.symbol,
                error: e.to_string(),
            })),
        }
    }

    async fn list_tokens(
        &self,
        _req: Request<TokenListRequest>,
    ) -> Result<Response<TokenListResponse>, Status> {
        let node = self.node.lock().unwrap();
        let tokens = node.ledger.list_tokens();

        let token_list = tokens
            .into_iter()
            .map(|tm| TokenMetadata {
                symbol: tm.symbol.clone(),
                name: tm.name.clone(),
                decimals: tm.decimals as u32,
                total_supply: tm.total_supply,
                creator: tm.creator.clone(),
                token_type: format!("{:?}", tm.token_type).to_lowercase(),
                status: format!("{:?}", tm.status).to_lowercase(),
                created_at: tm.created_at as i64,
                metadata_json: tm.metadata.clone().unwrap_or_else(|| "{}".to_string()),
            })
            .collect();

        Ok(Response::new(TokenListResponse { tokens: token_list }))
    }

    async fn get_token(
        &self,
        req: Request<TokenRequest>,
    ) -> Result<Response<TokenMetadata>, Status> {
        let req = req.into_inner();
        let node = self.node.lock().unwrap();

        if let Some(tm) = node.ledger.get_token(&req.symbol) {
            Ok(Response::new(TokenMetadata {
                symbol: tm.symbol.clone(),
                name: tm.name.clone(),
                decimals: tm.decimals as u32,
                total_supply: tm.total_supply,
                creator: tm.creator.clone(),
                token_type: format!("{:?}", tm.token_type).to_lowercase(),
                status: format!("{:?}", tm.status).to_lowercase(),
                created_at: tm.created_at as i64,
                metadata_json: tm.metadata.clone().unwrap_or_else(|| "{}".to_string()),
            }))
        } else {
            Err(Status::not_found(format!("Token not found: {}", req.symbol)))
        }
    }

    async fn mint(
        &self,
        req: Request<MintRequest>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();

        match node.ledger.mint(&req.caller, &req.to, req.token.clone(), req.amount) {
            Ok(_) => {
                let balance = node.balance(&req.to, &req.token);
                Ok(Response::new(BalanceResponse {
                    account: req.to,
                    token: req.token,
                    balance,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn burn(
        &self,
        req: Request<BurnRequest>,
    ) -> Result<Response<BurnResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();

        match node.ledger.burn(&req.caller, &req.token, &req.from, req.amount) {
            Ok(_) => {
                let remaining = node
                    .ledger
                    .get_token(&req.token)
                    .map(|tm| tm.total_supply)
                    .unwrap_or(0);

                Ok(Response::new(BurnResponse {
                    success: true,
                    remaining_supply: remaining,
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(BurnResponse {
                success: false,
                remaining_supply: 0,
                error: e.to_string(),
            })),
        }
    }

    async fn get_token_history(
        &self,
        req: Request<TokenHistoryRequest>,
    ) -> Result<Response<TokenHistoryResponse>, Status> {
        let req = req.into_inner();
        let node = self.node.lock().unwrap();

        let events = node
            .ledger
            .token_events
            .iter()
            .filter(|evt| {
                match evt {
                    crate::types::TokenEvent::Created { symbol, .. } => symbol == &req.token,
                    crate::types::TokenEvent::Minted { symbol, .. } => symbol == &req.token,
                    crate::types::TokenEvent::Burned { symbol, .. } => symbol == &req.token,
                    crate::types::TokenEvent::Frozen { symbol, .. } => symbol == &req.token,
                    crate::types::TokenEvent::Unfrozen { symbol, .. } => symbol == &req.token,
                }
            })
            .take(req.limit as usize)
            .map(|evt| {
                match evt {
                    crate::types::TokenEvent::Created { symbol, creator, name: _, decimals: _, block_height } => TokenEvent {
                        event_type: "created".to_string(),
                        token: symbol.clone(),
                        account: creator.clone(),
                        amount: 0,
                        block_height: *block_height,
                        timestamp: *block_height as i64 * 12,
                    },
                    crate::types::TokenEvent::Minted { symbol, to, amount, block_height } => TokenEvent {
                        event_type: "minted".to_string(),
                        token: symbol.clone(),
                        account: to.clone(),
                        amount: *amount,
                        block_height: *block_height,
                        timestamp: *block_height as i64 * 12,
                    },
                    crate::types::TokenEvent::Burned { symbol, from, amount, block_height } => TokenEvent {
                        event_type: "burned".to_string(),
                        token: symbol.clone(),
                        account: from.clone(),
                        amount: *amount,
                        block_height: *block_height,
                        timestamp: *block_height as i64 * 12,
                    },
                    crate::types::TokenEvent::Frozen { symbol, block_height } => TokenEvent {
                        event_type: "frozen".to_string(),
                        token: symbol.clone(),
                        account: String::new(),
                        amount: 0,
                        block_height: *block_height,
                        timestamp: *block_height as i64 * 12,
                    },
                    crate::types::TokenEvent::Unfrozen { symbol, block_height } => TokenEvent {
                        event_type: "unfrozen".to_string(),
                        token: symbol.clone(),
                        account: String::new(),
                        amount: 0,
                        block_height: *block_height,
                        timestamp: *block_height as i64 * 12,
                    },
                }
            })
            .collect();

        Ok(Response::new(TokenHistoryResponse { events }))
    }

    async fn get_transaction_history(
        &self,
        req: Request<TransactionHistoryRequest>,
    ) -> Result<Response<TransactionHistoryResponse>, Status> {
        let req = req.into_inner();
        let node = self.node.lock().unwrap();

        let mut txs = Vec::new();
        for block in node.blocks.iter().rev() {
            for tx in &block.transactions {
                // Extract transfer info from transaction
                match &tx.kind {
                    crate::types::TransactionKind::Transfer { from, to, token, amount } => {
                        if from == &req.account || to == &req.account {
                            txs.push(TransactionRecord {
                                tx_hash: hex::encode(&[0u8; 32]), // placeholder
                                kind: "transfer".to_string(),
                                from: from.clone(),
                                to: to.clone(),
                                token: token.clone(),
                                amount: *amount,
                                block_height: block.height,
                                timestamp: block.height as i64 * 12,
                                status: "success".to_string(),
                            });
                        }
                    },
                    _ => {}
                }
            }
        }

        let total_count = txs.len() as u64;
        let txs = txs
            .into_iter()
            .skip(req.offset as usize)
            .take(req.limit as usize)
            .collect();

        Ok(Response::new(TransactionHistoryResponse {
            transactions: txs,
            total_count,
        }))
    }
}

#[tonic::async_trait]
impl Settlement for SettlementService {
    async fn mint(
        &self,
        req: Request<SettlementMintRequest>,
    ) -> Result<Response<SettlementResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();

        match node.ledger.settlement_mint(
            &req.bank_account,
            &req.token,
            req.amount,
            req.reference.clone(),
        ) {
            Ok(_) => {
                let from_balance = node.balance(&req.bank_account, &req.token);
                let block_height = node.blocks.last().map(|b| b.height).unwrap_or(0);

                Ok(Response::new(SettlementResponse {
                    success: true,
                    tx_hash: format!("settlement_mint_{}", req.reference),
                    block_height,
                    from_balance: from_balance.to_string(),
                    to_balance: String::new(),
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SettlementResponse {
                success: false,
                tx_hash: String::new(),
                block_height: 0,
                from_balance: String::new(),
                to_balance: String::new(),
                error: e.to_string(),
            })),
        }
    }

    async fn burn(
        &self,
        req: Request<SettlementBurnRequest>,
    ) -> Result<Response<SettlementResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();

        match node.ledger.settlement_burn(
            &req.bank_account,
            &req.token,
            req.amount,
            req.reference.clone(),
        ) {
            Ok(_) => {
                let from_balance = node.balance(&req.bank_account, &req.token);
                let block_height = node.blocks.last().map(|b| b.height).unwrap_or(0);

                Ok(Response::new(SettlementResponse {
                    success: true,
                    tx_hash: format!("settlement_burn_{}", req.reference),
                    block_height,
                    from_balance: from_balance.to_string(),
                    to_balance: String::new(),
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SettlementResponse {
                success: false,
                tx_hash: String::new(),
                block_height: 0,
                from_balance: String::new(),
                to_balance: String::new(),
                error: e.to_string(),
            })),
        }
    }

    async fn transfer(
        &self,
        req: Request<SettlementTransferRequest>,
    ) -> Result<Response<SettlementResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();

        match node.ledger.settlement_transfer(
            &req.from_account,
            &req.to_account,
            &req.token,
            req.amount,
            req.reference.clone(),
        ) {
            Ok(_) => {
                let from_balance = node.balance(&req.from_account, &req.token);
                let to_balance = node.balance(&req.to_account, &req.token);
                let block_height = node.blocks.last().map(|b| b.height).unwrap_or(0);

                Ok(Response::new(SettlementResponse {
                    success: true,
                    tx_hash: format!("settlement_transfer_{}", req.reference),
                    block_height,
                    from_balance: from_balance.to_string(),
                    to_balance: to_balance.to_string(),
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(SettlementResponse {
                success: false,
                tx_hash: String::new(),
                block_height: 0,
                from_balance: String::new(),
                to_balance: String::new(),
                error: e.to_string(),
            })),
        }
    }

    async fn get_status(
        &self,
        req: Request<SettlementStatusRequest>,
    ) -> Result<Response<SettlementStatusResponse>, Status> {
        let _req = req.into_inner();
        let node = self.node.lock().unwrap();

        if let Some(evt) = node.ledger.token_events.last() {
            match evt {
                crate::types::TokenEvent::Minted { block_height, .. } |
                crate::types::TokenEvent::Burned { block_height, .. } => {
                    Ok(Response::new(SettlementStatusResponse {
                        status: "confirmed".to_string(),
                        block_height: *block_height,
                        timestamp: *block_height as i64 * 12,
                        tx_hash: "settlement_tx".to_string(),
                    }))
                },
                _ => Ok(Response::new(SettlementStatusResponse {
                    status: "pending".to_string(),
                    block_height: 0,
                    timestamp: 0,
                    tx_hash: String::new(),
                })),
            }
        } else {
            Ok(Response::new(SettlementStatusResponse {
                status: "pending".to_string(),
                block_height: 0,
                timestamp: 0,
                tx_hash: String::new(),
            }))
        }
    }

    async fn list_banks(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<BankListResponse>, Status> {
        let node = self.node.lock().unwrap();
        let banks = node
            .ledger
            .banks
            .values()
            .map(|bank| BankAccountResponse {
                account_id: bank.account_id.clone(),
                bank_name: bank.bank_name.clone(),
                swift_code: bank.swift_code.clone(),
                is_approved: bank.is_approved,
                created_at: bank.created_at as i64,
            })
            .collect();

        Ok(Response::new(BankListResponse { banks }))
    }

    async fn get_bank(
        &self,
        req: Request<BankAccountRequest>,
    ) -> Result<Response<BankAccountResponse>, Status> {
        let req = req.into_inner();
        let node = self.node.lock().unwrap();

        if let Some(bank) = node.ledger.banks.get(&req.account_id) {
            Ok(Response::new(BankAccountResponse {
                account_id: bank.account_id.clone(),
                bank_name: bank.bank_name.clone(),
                swift_code: bank.swift_code.clone(),
                is_approved: bank.is_approved,
                created_at: bank.created_at as i64,
            }))
        } else {
            Err(Status::not_found(format!("Bank not found: {}", req.account_id)))
        }
    }
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn approve_bank(
        &self,
        req: Request<BankAccountResponse>,
    ) -> Result<Response<BankAccountResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();
        let admin = node.admin.clone();

        match node.ledger.approve_bank(
            &admin,
            req.account_id.clone(),
            req.bank_name.clone(),
            req.swift_code.clone(),
        ) {
            Ok(_) => {
                if let Some(bank) = node.ledger.banks.get(&req.account_id) {
                    Ok(Response::new(BankAccountResponse {
                        account_id: bank.account_id.clone(),
                        bank_name: bank.bank_name.clone(),
                        swift_code: bank.swift_code.clone(),
                        is_approved: bank.is_approved,
                        created_at: bank.created_at as i64,
                    }))
                } else {
                    Err(Status::internal("Bank approval failed"))
                }
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn freeze_token(
        &self,
        req: Request<FreezeTokenRequest>,
    ) -> Result<Response<FreezeTokenResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();
        let admin = node.admin.clone();

        match node.ledger.freeze_token(&admin, &req.token) {
            Ok(_) => Ok(Response::new(FreezeTokenResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(FreezeTokenResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn unfreeze_token(
        &self,
        req: Request<FreezeTokenRequest>,
    ) -> Result<Response<FreezeTokenResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();
        let admin = node.admin.clone();

        match node.ledger.unfreeze_token(&admin, &req.token) {
            Ok(_) => Ok(Response::new(FreezeTokenResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(FreezeTokenResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn set_daily_limit(
        &self,
        req: Request<DailyLimitRequest>,
    ) -> Result<Response<DailyLimitResponse>, Status> {
        let req = req.into_inner();
        let mut node = self.node.lock().unwrap();
        let admin = node.admin.clone();

        match node.ledger.set_daily_limit(
            &admin,
            &req.bank_account,
            &req.token,
            req.daily_limit,
        ) {
            Ok(_) => Ok(Response::new(DailyLimitResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(DailyLimitResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }
}

/// ===== CommitmentAnchor gRPC service =====
#[tonic::async_trait]
impl CommitmentAnchor for CommitmentAnchorService {
    async fn anchor_commitment(
        &self,
        req: Request<AnchorCommitmentRequest>,
    ) -> Result<Response<AnchorCommitmentResponse>, Status> {
        let req = req.into_inner();
        let mut n = self.node.lock().unwrap();

        match n.ledger.anchor_commitment(
            req.commitment_hash.clone(),
            req.policy_id,
            req.txn_ref,
            req.timestamp,
            if req.context_ref.is_empty() { None } else { Some(req.context_ref) },
        ) {
            Ok((block_height, tx_hash)) => Ok(Response::new(AnchorCommitmentResponse {
                success: true,
                commitment_hash: req.commitment_hash,
                block_height,
                tx_hash,
                anchored_at: req.timestamp,
                error_code: String::new(),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(AnchorCommitmentResponse {
                success: false,
                commitment_hash: req.commitment_hash,
                block_height: 0,
                tx_hash: String::new(),
                anchored_at: 0,
                error_code: "ANCHOR_FAILED".to_string(),
                error_message: format!("{:?}", e),
            })),
        }
    }

    async fn get_commitment(
        &self,
        req: Request<GetCommitmentRequest>,
    ) -> Result<Response<GetCommitmentResponse>, Status> {
        let hash = req.into_inner().commitment_hash;
        let n = self.node.lock().unwrap();

        match n.ledger.query_commitment(&hash) {
            Some(record) => Ok(Response::new(GetCommitmentResponse {
                success: true,
                commitment_hash: record.commitment_hash,
                policy_id: record.policy_id,
                txn_ref: record.txn_ref,
                timestamp: record.timestamp,
                block_height: record.block_height,
                verified: record.verified,
                context_ref: record.context_ref.unwrap_or_default(),
                error: String::new(),
            })),
            None => Ok(Response::new(GetCommitmentResponse {
                success: false,
                commitment_hash: hash,
                policy_id: String::new(),
                txn_ref: String::new(),
                timestamp: 0,
                block_height: 0,
                verified: false,
                context_ref: String::new(),
                error: "Commitment not found".to_string(),
            })),
        }
    }
}

/// ===== ProofVerifier gRPC service =====
#[tonic::async_trait]
impl ProofVerifier for ProofVerifierService {
    async fn verify_proof(
        &self,
        req: Request<VerifyProofRequest>,
    ) -> Result<Response<VerifyProofResponse>, Status> {
        let req = req.into_inner();
        let mut n = self.node.lock().unwrap();

        let proof_data = req.proof_data; // already bytes in proto
        let public_inputs = if req.public_inputs_json.is_empty() {
            None
        } else {
            Some(req.public_inputs_json)
        };
        let capsule_version = "winterfell_v1".to_string();

        match n.ledger.verify_proof(
            req.commitment_hash.clone(),
            req.proof_hash.clone(),
            proof_data,
            req.proof_type,
            public_inputs,
            capsule_version.clone(),
        ) {
            Ok(record) => {
                let verified = matches!(
                    record.verification_status,
                    crate::types::ProofVerificationStatus::Verified
                );
                Ok(Response::new(VerifyProofResponse {
                    success: true,
                    commitment_hash: record.commitment_hash,
                    proof_hash: record.proof_hash,
                    verified,
                    block_height: record.verification_block.unwrap_or(0),
                    verifier_capsule_version: capsule_version,
                    error_code: String::new(),
                    error_message: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(VerifyProofResponse {
                success: false,
                commitment_hash: req.commitment_hash,
                proof_hash: req.proof_hash,
                verified: false,
                block_height: 0,
                verifier_capsule_version: capsule_version,
                error_code: "VERIFY_FAILED".to_string(),
                error_message: format!("{:?}", e),
            })),
        }
    }

    async fn get_proof(
        &self,
        req: Request<GetProofRequest>,
    ) -> Result<Response<GetProofResponse>, Status> {
        let proof_hash = req.into_inner().proof_hash;
        let n = self.node.lock().unwrap();

        // Search through all proofs to find by proof_hash
        // The ledger stores proofs keyed by proof_hash
        if let Some(proof_record) = n.ledger.proofs.get(&proof_hash) {
            let status = match proof_record.verification_status {
                crate::types::ProofVerificationStatus::Verified => "VERIFIED",
                crate::types::ProofVerificationStatus::Failed => "FAILED",
                crate::types::ProofVerificationStatus::Pending => "PENDING",
            };
            return Ok(Response::new(GetProofResponse {
                success: true,
                proof_hash: proof_record.proof_hash.clone(),
                commitment_hash: proof_record.commitment_hash.clone(),
                verification_status: status.to_string(),
                submitted_at: proof_record.submitted_at,
                verification_block: proof_record.verification_block.unwrap_or(0),
                error_message: proof_record.error_message.clone().unwrap_or_default(),
                error: String::new(),
            }));
        }

        Ok(Response::new(GetProofResponse {
            success: false,
            proof_hash,
            commitment_hash: String::new(),
            verification_status: String::new(),
            submitted_at: 0,
            verification_block: 0,
            error_message: String::new(),
            error: "Proof not found".to_string(),
        }))
    }
}

/// start the gRPC server on given address
pub async fn serve_grpc(node: SharedNode, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let l1_svc = L1Service { node: node.clone() };
    let tokens_svc = TokensService { node: node.clone() };
    let settlement_svc = SettlementService { node: node.clone() };
    let admin_svc = AdminService { node: node.clone() };
    let commitment_svc = CommitmentAnchorService { node: node.clone() };
    let proof_svc = ProofVerifierService { node: node.clone() };

    tonic::transport::Server::builder()
        .add_service(L1Server::new(l1_svc))
        .add_service(TokensServer::new(tokens_svc))
        .add_service(SettlementServer::new(settlement_svc))
        .add_service(AdminServer::new(admin_svc))
        .add_service(CommitmentAnchorServer::new(commitment_svc))
        .add_service(ProofVerifierServer::new(proof_svc))
        .serve(addr)
        .await?;
    Ok(())
}
