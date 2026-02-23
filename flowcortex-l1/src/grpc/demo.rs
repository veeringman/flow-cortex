/// Demo Console/Dashboard Backend API
/// Phase 13.7: Demo console/dashboard backend
///
/// Provides REST/gRPC endpoints for the demo UI to interact with
/// settlement scenarios and real-time event streams

use crate::demo::{
    DemoDataFixtures, DemoSettlementConfig, DemoSettlementScenario, 
    FloweRStablecoinConfig, SettlementStep, SettlementStepStatus,
};
use crate::rpc::SharedNode;
use crate::types::{CommitmentRecord, ProofRecord, CommitmentProofEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct DemoService {
    pub node: SharedNode,
    /// Active demo scenarios (scenario_id -> scenario)
    pub scenarios: Arc<RwLock<HashMap<String, DemoSettlementScenario>>>,
}

impl DemoService {
    pub fn new(node: SharedNode) -> Self {
        DemoService {
            node,
            scenarios: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a new demo settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDemoSettlementRequest {
    pub scenario_id: String,
    pub amount: Option<u128>,
    pub sender_id: Option<String>,
    pub receiver_id: Option<String>,
}

/// Response from creating a demo settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDemoSettlementResponse {
    pub success: bool,
    pub scenario_id: String,
    pub config: DemoSettlementConfig,
    pub steps: Vec<SettlementStep>,
    pub message: String,
}

/// Request to execute a settlement step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteStepRequest {
    pub scenario_id: String,
    pub step_number: u8,
}

/// Response from executing a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteStepResponse {
    pub success: bool,
    pub scenario_id: String,
    pub step_number: u8,
    pub step_name: String,
    pub message: String,
    pub commitment_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub block_height: Option<u64>,
}

/// Request to get settlement status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSettlementStatusRequest {
    pub scenario_id: String,
}

/// Response with settlement status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSettlementStatusResponse {
    pub success: bool,
    pub scenario_id: String,
    pub config: DemoSettlementConfig,
    pub steps: Vec<SettlementStep>,
    pub current_step: u8,
    pub completion_percentage: u8,
    pub is_complete: bool,
    pub commitment_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub block_height: Option<u64>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

/// Request to list all settlements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSettlementsRequest {
    pub limit: Option<usize>,
}

/// Response with all settlements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSettlementsResponse {
    pub success: bool,
    pub scenarios: Vec<SettlementSummary>,
    pub total_count: usize,
}

/// Settlement summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSummary {
    pub scenario_id: String,
    pub amount: String,
    pub sender: String,
    pub receiver: String,
    pub current_step: u8,
    pub completion_percentage: u8,
    pub is_complete: bool,
    pub started_at: Option<u64>,
}

/// Request to get real-time events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEventsRequest {
    pub scenario_id: Option<String>,
    pub limit: Option<usize>,
}

/// Response with events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEventsResponse {
    pub success: bool,
    pub events: Vec<DemoEventDetails>,
    pub total_count: usize,
}

/// Demo event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoEventDetails {
    pub event_id: String,
    pub event_type: String,
    pub scenario_id: Option<String>,
    pub commitment_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub block_height: u64,
    pub timestamp: u64,
    pub details: String,
}

// ============================================================================
// Demo API Implementation
// ============================================================================

impl DemoService {
    /// Create a new demo settlement scenario
    /// API: POST /demo/settlements
    pub fn create_demo_settlement(
        &self,
        req: CreateDemoSettlementRequest,
    ) -> CreateDemoSettlementResponse {
        // Create config based on request
        let config = if let Some(amount) = req.amount {
            DemoSettlementConfig::with_amount(amount)
        } else if let (Some(sender), Some(receiver)) = (&req.sender_id, &req.receiver_id) {
            DemoSettlementConfig::with_parties(sender, receiver)
        } else {
            DemoSettlementConfig::default()
        };

        // Create scenario
        let scenario = DemoSettlementScenario::new(req.scenario_id.clone(), config.clone());
        let steps = scenario.steps.clone();

        // Store scenario
        let mut scenarios = self.scenarios.write().unwrap();
        scenarios.insert(req.scenario_id.clone(), scenario);

        CreateDemoSettlementResponse {
            success: true,
            scenario_id: req.scenario_id,
            config,
            steps,
            message: "Demo settlement created successfully".to_string(),
        }
    }

    /// Execute a specific step in the settlement flow
    /// API: POST /demo/settlements/{id}/steps/{step_number}
    pub fn execute_step(&self, req: ExecuteStepRequest) -> ExecuteStepResponse {
        let mut scenarios = self.scenarios.write().unwrap();
        
        if let Some(scenario) = scenarios.get_mut(&req.scenario_id) {
            let node = self.node.clone();
            let n = node.lock().unwrap();
            let current_height = n.ledger.block_height;
            let timestamp = current_height;

            // Execute the specific step
            match req.step_number {
                1 => {
                    // Step 1: Anchor commitment
                    let commitment_hash = DemoDataFixtures::sample_commitment_hash(&req.scenario_id);
                    scenario.commitment_hash = Some(commitment_hash.clone());
                    scenario.block_height = Some(current_height);
                    
                    // Start the scenario if not started
                    if scenario.started_at.is_none() {
                        scenario.start(timestamp);
                    }
                    
                    scenario.complete_step(1, timestamp).ok();
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 1,
                        step_name: "Anchor Settlement Commitment".to_string(),
                        message: "Commitment anchored successfully".to_string(),
                        commitment_hash: Some(commitment_hash),
                        proof_hash: None,
                        block_height: Some(current_height),
                    }
                }
                2 => {
                    // Step 2: Wait for confirmation (auto-complete)
                    scenario.complete_step(2, timestamp).ok();
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 2,
                        step_name: "Wait for Blockchain Confirmation".to_string(),
                        message: "Blockchain confirmation received".to_string(),
                        commitment_hash: scenario.commitment_hash.clone(),
                        proof_hash: None,
                        block_height: scenario.block_height,
                    }
                }
                3 => {
                    // Step 3: Submit proof
                    if let Some(commitment_hash) = &scenario.commitment_hash {
                        let proof_hash = DemoDataFixtures::sample_proof_hash(commitment_hash);
                        scenario.proof_hash = Some(proof_hash.clone());
                        scenario.complete_step(3, timestamp).ok();
                        
                        ExecuteStepResponse {
                            success: true,
                            scenario_id: req.scenario_id,
                            step_number: 3,
                            step_name: "Submit STARK Proof".to_string(),
                            message: "Proof submitted successfully".to_string(),
                            commitment_hash: scenario.commitment_hash.clone(),
                            proof_hash: Some(proof_hash),
                            block_height: scenario.block_height,
                        }
                    } else {
                        ExecuteStepResponse {
                            success: false,
                            scenario_id: req.scenario_id,
                            step_number: 3,
                            step_name: "Submit STARK Proof".to_string(),
                            message: "Cannot submit proof: commitment not anchored".to_string(),
                            commitment_hash: None,
                            proof_hash: None,
                            block_height: None,
                        }
                    }
                }
                4 => {
                    // Step 4: Verify proof
                    scenario.complete_step(4, timestamp).ok();
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 4,
                        step_name: "Verify Proof".to_string(),
                        message: "Proof verified successfully".to_string(),
                        commitment_hash: scenario.commitment_hash.clone(),
                        proof_hash: scenario.proof_hash.clone(),
                        block_height: scenario.block_height,
                    }
                }
                5 => {
                    // Step 5: Mint FloweR tokens
                    scenario.complete_step(5, timestamp).ok();
                    
                    let flower_amount = scenario.flower_config.inr_to_flower(scenario.config.amount);
                    let formatted = scenario.flower_config.format_flower(flower_amount);
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 5,
                        step_name: "Mint FloweR Tokens".to_string(),
                        message: format!("Minted {} to {}", formatted, scenario.config.receiver.name),
                        commitment_hash: scenario.commitment_hash.clone(),
                        proof_hash: scenario.proof_hash.clone(),
                        block_height: scenario.block_height,
                    }
                }
                6 => {
                    // Step 6: Burn collateral
                    scenario.complete_step(6, timestamp).ok();
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 6,
                        step_name: "Burn Collateral".to_string(),
                        message: format!("Burned collateral from {}", scenario.config.sender.name),
                        commitment_hash: scenario.commitment_hash.clone(),
                        proof_hash: scenario.proof_hash.clone(),
                        block_height: scenario.block_height,
                    }
                }
                7 => {
                    // Step 7: Update settlement status
                    scenario.complete_step(7, timestamp).ok();
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 7,
                        step_name: "Update Settlement Status".to_string(),
                        message: "Settlement status updated to COMPLETE".to_string(),
                        commitment_hash: scenario.commitment_hash.clone(),
                        proof_hash: scenario.proof_hash.clone(),
                        block_height: scenario.block_height,
                    }
                }
                8 => {
                    // Step 8: Emit completion event
                    scenario.complete_step(8, timestamp).ok();
                    
                    ExecuteStepResponse {
                        success: true,
                        scenario_id: req.scenario_id,
                        step_number: 8,
                        step_name: "Emit Completion Event".to_string(),
                        message: "Settlement completed successfully".to_string(),
                        commitment_hash: scenario.commitment_hash.clone(),
                        proof_hash: scenario.proof_hash.clone(),
                        block_height: scenario.block_height,
                    }
                }
                _ => {
                    ExecuteStepResponse {
                        success: false,
                        scenario_id: req.scenario_id,
                        step_number: req.step_number,
                        step_name: "Unknown Step".to_string(),
                        message: format!("Invalid step number: {}", req.step_number),
                        commitment_hash: None,
                        proof_hash: None,
                        block_height: None,
                    }
                }
            }
        } else {
            ExecuteStepResponse {
                success: false,
                scenario_id: req.scenario_id.clone(),
                step_number: req.step_number,
                step_name: "".to_string(),
                message: format!("Scenario not found: {}", req.scenario_id),
                commitment_hash: None,
                proof_hash: None,
                block_height: None,
            }
        }
    }

    /// Get settlement status
    /// API: GET /demo/settlements/{id}
    pub fn get_settlement_status(&self, req: GetSettlementStatusRequest) -> GetSettlementStatusResponse {
        let scenarios = self.scenarios.read().unwrap();
        
        if let Some(scenario) = scenarios.get(&req.scenario_id) {
            GetSettlementStatusResponse {
                success: true,
                scenario_id: req.scenario_id,
                config: scenario.config.clone(),
                steps: scenario.steps.clone(),
                current_step: scenario.current_step,
                completion_percentage: scenario.completion_percentage(),
                is_complete: scenario.is_complete(),
                commitment_hash: scenario.commitment_hash.clone(),
                proof_hash: scenario.proof_hash.clone(),
                block_height: scenario.block_height,
                started_at: scenario.started_at,
                completed_at: scenario.completed_at,
            }
        } else {
            GetSettlementStatusResponse {
                success: false,
                scenario_id: req.scenario_id,
                config: DemoSettlementConfig::default(),
                steps: vec![],
                current_step: 0,
                completion_percentage: 0,
                is_complete: false,
                commitment_hash: None,
                proof_hash: None,
                block_height: None,
                started_at: None,
                completed_at: None,
            }
        }
    }

    /// List all settlements
    /// API: GET /demo/settlements
    pub fn list_settlements(&self, req: ListSettlementsRequest) -> ListSettlementsResponse {
        let scenarios = self.scenarios.read().unwrap();
        let limit = req.limit.unwrap_or(100);
        
        let summaries: Vec<SettlementSummary> = scenarios
            .iter()
            .take(limit)
            .map(|(id, scenario)| SettlementSummary {
                scenario_id: id.clone(),
                amount: scenario.config.formatted_amount(),
                sender: scenario.config.sender.name.clone(),
                receiver: scenario.config.receiver.name.clone(),
                current_step: scenario.current_step,
                completion_percentage: scenario.completion_percentage(),
                is_complete: scenario.is_complete(),
                started_at: scenario.started_at,
            })
            .collect();
        
        let total_count = summaries.len();
        
        ListSettlementsResponse {
            success: true,
            scenarios: summaries,
            total_count,
        }
    }

    /// Get real-time events
    /// API: GET /demo/events
    pub fn get_events(&self, req: GetEventsRequest) -> GetEventsResponse {
        let node = self.node.clone();
        let n = node.lock().unwrap();
        let limit = req.limit.unwrap_or(50);
        
        // Get events from ledger
        let events_data = n.ledger.get_all_events();
        
        // Convert to demo event format
        let mut demo_events: Vec<DemoEventDetails> = vec![];
        
        for (idx, event) in events_data.iter().enumerate().take(limit) {
            let (event_type, commitment_hash, proof_hash, block_height, timestamp, details) = 
                match event {
                    CommitmentProofEvent::CommitmentAnchored { 
                        commitment_hash, policy_id, txn_ref, block_height, timestamp 
                    } => {
                        (
                            "commitment.anchored".to_string(),
                            commitment_hash.clone(),
                            None,
                            *block_height,
                            *timestamp,
                            format!("Commitment anchored: {} (txn_ref: {}, policy: {})", 
                                commitment_hash, txn_ref, policy_id),
                        )
                    },
                    CommitmentProofEvent::ProofVerified { 
                        commitment_hash, proof_hash, verification_block, verified_at, verifier_capsule_version 
                    } => {
                        (
                            "proof.verified".to_string(),
                            commitment_hash.clone(),
                            Some(proof_hash.clone()),
                            *verification_block,
                            *verified_at,
                            format!("Proof verified successfully (capsule: {})", 
                                verifier_capsule_version),
                        )
                    },
                    CommitmentProofEvent::ProofVerificationFailed { 
                        commitment_hash, proof_hash, error_reason, block_height, failed_at 
                    } => {
                        (
                            "proof.verification_failed".to_string(),
                            commitment_hash.clone(),
                            Some(proof_hash.clone()),
                            *block_height,
                            *failed_at,
                            format!("Proof verification failed: {}", error_reason),
                        )
                    },
                    CommitmentProofEvent::CommitmentNotFound { 
                        commitment_hash, proof_hash, submitted_at 
                    } => {
                        (
                            "error.commitment_not_found".to_string(),
                            commitment_hash.clone(),
                            Some(proof_hash.clone()),
                            0,
                            *submitted_at,
                            "Commitment not found for proof submission".to_string(),
                        )
                    },
                    CommitmentProofEvent::InvalidProofFormat { 
                        error_description, submitted_at 
                    } => {
                        (
                            "error.invalid_proof_format".to_string(),
                            "".to_string(),
                            None,
                            0,
                            *submitted_at,
                            format!("Invalid proof format: {}", error_description),
                        )
                    },
                    CommitmentProofEvent::DuplicateProof { 
                        commitment_hash, proof_hash, previous_verification_status 
                    } => {
                        (
                            "error.duplicate_proof".to_string(),
                            commitment_hash.clone(),
                            Some(proof_hash.clone()),
                            0,
                            0,
                            format!("Duplicate proof (previous status: {:?})", 
                                previous_verification_status),
                        )
                    },
                };

            let demo_event = DemoEventDetails {
                event_id: format!("evt_{:04}", idx),
                event_type,
                scenario_id: None,  // Could be mapped from commitment_hash
                commitment_hash: if commitment_hash.is_empty() { None } else { Some(commitment_hash) },
                proof_hash,
                block_height,
                timestamp,
                details,
            };
            demo_events.push(demo_event);
        }
        
        // Filter by scenario_id if provided
        if let Some(scenario_id) = req.scenario_id {
            let scenarios = self.scenarios.read().unwrap();
            if let Some(scenario) = scenarios.get(&scenario_id) {
                if let Some(commitment_hash) = &scenario.commitment_hash {
                    demo_events.retain(|e| {
                        e.commitment_hash.as_ref() == Some(commitment_hash)
                    });
                }
            }
        }
        
        let total_count = demo_events.len();
        
        GetEventsResponse {
            success: true,
            events: demo_events,
            total_count,
        }
    }

    /// Auto-execute entire settlement flow (for quick demo)
    /// API: POST /demo/settlements/{id}/auto-execute
    pub fn auto_execute_settlement(&self, scenario_id: String) -> GetSettlementStatusResponse {
        // Execute all 8 steps in sequence
        for step in 1..=8 {
            let req = ExecuteStepRequest {
                scenario_id: scenario_id.clone(),
                step_number: step,
            };
            let _response = self.execute_step(req);
            
            // Small delay simulation (in real scenario, steps would have natural delays)
            // For demo purposes, we'll execute immediately
        }
        
        // Return final status
        self.get_settlement_status(GetSettlementStatusRequest {
            scenario_id,
        })
    }

    /// Reset a settlement scenario
    /// API: DELETE /demo/settlements/{id}
    pub fn reset_settlement(&self, scenario_id: String) -> (bool, String) {
        let mut scenarios = self.scenarios.write().unwrap();
        if scenarios.remove(&scenario_id).is_some() {
            (true, format!("Settlement {} reset successfully", scenario_id))
        } else {
            (false, format!("Settlement {} not found", scenario_id))
        }
    }

    /// Get demo dashboard stats
    /// API: GET /demo/stats
    pub fn get_dashboard_stats(&self) -> DashboardStats {
        let scenarios = self.scenarios.read().unwrap();
        let node = self.node.clone();
        let n = node.lock().unwrap();
        
        let total_settlements = scenarios.len();
        let completed_settlements = scenarios.values().filter(|s| s.is_complete()).count();
        let in_progress_settlements = total_settlements - completed_settlements;
        let total_events = n.ledger.get_all_events().len();
        let total_commitments = n.ledger.commitments.len();
        let total_proofs = n.ledger.proofs.len();
        
        // Calculate total value
        let total_value: u128 = scenarios.values().map(|s| s.config.amount).sum();
        let flower_config = FloweRStablecoinConfig::default();
        let total_value_formatted = format!(
            "₹{} / {}",
            total_value / 100,
            flower_config.format_flower(flower_config.inr_to_flower(total_value))
        );
        
        DashboardStats {
            total_settlements,
            completed_settlements,
            in_progress_settlements,
            total_events,
            total_commitments,
            total_proofs,
            total_value_formatted,
            block_height: n.ledger.block_height,
        }
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_settlements: usize,
    pub completed_settlements: usize,
    pub in_progress_settlements: usize,
    pub total_events: usize,
    pub total_commitments: usize,
    pub total_proofs: usize,
    pub total_value_formatted: String,
    pub block_height: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::{Arc, Mutex};

    fn create_test_service() -> DemoService {
        let node = Arc::new(Mutex::new(Node::new("admin".to_string())));
        DemoService::new(node)
    }

    #[test]
    fn test_create_demo_settlement() {
        let service = create_test_service();
        let req = CreateDemoSettlementRequest {
            scenario_id: "test_001".to_string(),
            amount: Some(100_000_000_00),
            sender_id: None,
            receiver_id: None,
        };
        
        let response = service.create_demo_settlement(req);
        assert!(response.success);
        assert_eq!(response.scenario_id, "test_001");
        assert_eq!(response.steps.len(), 8);
    }

    #[test]
    fn test_execute_steps() {
        let service = create_test_service();
        
        // Create settlement
        let create_req = CreateDemoSettlementRequest {
            scenario_id: "test_002".to_string(),
            amount: None,
            sender_id: None,
            receiver_id: None,
        };
        service.create_demo_settlement(create_req);
        
        // Execute step 1
        let step_req = ExecuteStepRequest {
            scenario_id: "test_002".to_string(),
            step_number: 1,
        };
        let response = service.execute_step(step_req);
        assert!(response.success);
        assert_eq!(response.step_number, 1);
        assert!(response.commitment_hash.is_some());
    }

    #[test]
    fn test_auto_execute() {
        let service = create_test_service();
        
        // Create settlement
        let create_req = CreateDemoSettlementRequest {
            scenario_id: "test_003".to_string(),
            amount: None,
            sender_id: None,
            receiver_id: None,
        };
        service.create_demo_settlement(create_req);
        
        // Auto-execute all steps
        let response = service.auto_execute_settlement("test_003".to_string());
        assert!(response.success);
        assert!(response.is_complete);
        assert_eq!(response.completion_percentage, 100);
    }
}
