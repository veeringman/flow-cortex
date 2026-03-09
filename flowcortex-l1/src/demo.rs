/// Demo-Specific Implementation Module
/// Phase 13: Demo-Specific Features
/// 
/// This module contains all demo-specific configuration, orchestration,
/// and utilities for demonstrating FlowCortex settlement with FloweR stablecoin

use crate::ledger::Ledger;
use crate::types::{CommitmentRecord, CommitmentProofEvent, ProofRecord, ProofVerificationStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Phase 13.1: Mock Settlement Configuration
// ============================================================================

/// Settlement currency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    INR,  // Indian Rupee
    USD,  // US Dollar
    EUR,  // Euro
}

impl Currency {
    pub fn symbol(&self) -> &str {
        match self {
            Currency::INR => "₹",
            Currency::USD => "$",
            Currency::EUR => "€",
        }
    }
}

/// Settlement party (bank or financial institution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementParty {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub bic_code: String,  // Bank Identifier Code
}

/// Demo settlement configuration
/// Subtask 13.1: Create mock settlement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoSettlementConfig {
    /// Settlement amount (in base units, e.g., paise for INR)
    pub amount: u128,
    /// Settlement currency
    pub currency: Currency,
    /// Sending bank
    pub sender: SettlementParty,
    /// Receiving bank
    pub receiver: SettlementParty,
    /// Settlement window: "T+0" (real-time), "T+1", "T+2", etc.
    pub settlement_window: String,
    /// Settlement reference ID
    pub reference_id: String,
}

impl Default for DemoSettlementConfig {
    fn default() -> Self {
        // Default: ₹50 Million settlement from Bank A to Bank B
        DemoSettlementConfig {
            amount: 50_000_000_00,  // ₹50M in paise (100 paise = 1 rupee)
            currency: Currency::INR,
            sender: SettlementParty {
                id: "BANK_A".to_string(),
                name: "Bank A - Commercial Bank".to_string(),
                account_id: "bank_a".to_string(),
                bic_code: "BANKA001".to_string(),
            },
            receiver: SettlementParty {
                id: "BANK_B".to_string(),
                name: "Bank B - Investment Bank".to_string(),
                account_id: "bank_b".to_string(),
                bic_code: "BANKB002".to_string(),
            },
            settlement_window: "T+0".to_string(),
            reference_id: "SETTLE-2026-02-23-001".to_string(),
        }
    }
}

impl DemoSettlementConfig {
    /// Create a new demo settlement with custom amount
    pub fn with_amount(amount: u128) -> Self {
        DemoSettlementConfig {
            amount,
            ..Default::default()
        }
    }

    /// Create a settlement with custom parties
    pub fn with_parties(sender_id: &str, receiver_id: &str) -> Self {
        let mut config = Self::default();
        config.sender.id = sender_id.to_string();
        config.sender.account_id = sender_id.to_lowercase();
        config.receiver.id = receiver_id.to_string();
        config.receiver.account_id = receiver_id.to_lowercase();
        config
    }

    /// Get formatted amount with currency symbol
    pub fn formatted_amount(&self) -> String {
        let amount_in_major = self.amount / 100;  // Convert paise to rupees
        format!("{}{}", self.currency.symbol(), amount_in_major)
    }

    /// Generate transaction reference
    pub fn transaction_reference(&self) -> String {
        format!(
            "TXN-{}-{}-{}",
            self.sender.id,
            self.receiver.id,
            self.reference_id
        )
    }
}

// ============================================================================
// Phase 13.2: FloweR Stablecoin Configuration
// ============================================================================

/// FloweR stablecoin configuration
/// Subtask 13.2: Create FloweR stablecoin interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloweRStablecoinConfig {
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Total supply (in base units)
    pub total_supply: u128,
    /// Decimal places
    pub decimals: u8,
    /// Minting authority account ID
    pub mint_authority: String,
    /// Burning authority account ID
    pub burn_authority: String,
    /// Peg currency
    pub peg_currency: Currency,
    /// Peg ratio (1:1 for INR)
    pub peg_ratio: f64,
}

impl Default for FloweRStablecoinConfig {
    fn default() -> Self {
        FloweRStablecoinConfig {
            symbol: "FLOWER".to_string(),
            name: "Flow Rupee".to_string(),
            total_supply: 250_000_000_000_000,  // 250M with 6 decimals
            decimals: 6,
            mint_authority: "fortress_digital".to_string(),
            burn_authority: "fortress_digital".to_string(),
            peg_currency: Currency::INR,
            peg_ratio: 1.0,  // 1 FLOWER = 1 INR
        }
    }
}

impl FloweRStablecoinConfig {
    /// Convert INR amount (in paise) to FLOWER tokens (in base units)
    pub fn inr_to_flower(&self, inr_paise: u128) -> u128 {
        // 1 INR = 100 paise
        // 1 FLOWER = 1,000,000 base units (6 decimals)
        // So 100 paise = 1,000,000 FLOWER base units
        inr_paise * 10_000  // paise * 10,000 = FLOWER base units
    }

    /// Convert FLOWER tokens (in base units) to INR amount (in paise)
    pub fn flower_to_inr(&self, flower_base_units: u128) -> u128 {
        flower_base_units / 10_000
    }

    /// Format FLOWER amount (base units to display)
    pub fn format_flower(&self, base_units: u128) -> String {
        let divisor = 10u128.pow(self.decimals as u32);
        let major = base_units / divisor;
        let minor = base_units % divisor;
        format!("{}.{:06} FLOWER", major, minor)
    }
}

// ============================================================================
// Phase 13.3: Demo Scenario Orchestrator
// ============================================================================

/// Settlement step status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SettlementStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Individual settlement step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementStep {
    pub step_number: u8,
    pub name: String,
    pub description: String,
    pub status: SettlementStepStatus,
    pub timestamp: Option<u64>,
    pub error: Option<String>,
}

/// Demo settlement scenario
/// Subtask 13.3: Implement demo scenario orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoSettlementScenario {
    pub scenario_id: String,
    pub config: DemoSettlementConfig,
    pub flower_config: FloweRStablecoinConfig,
    pub steps: Vec<SettlementStep>,
    pub current_step: u8,
    pub commitment_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub block_height: Option<u64>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

impl DemoSettlementScenario {
    /// Create a new demo settlement scenario
    pub fn new(scenario_id: String, config: DemoSettlementConfig) -> Self {
        let steps = Self::create_steps();
        DemoSettlementScenario {
            scenario_id,
            config,
            flower_config: FloweRStablecoinConfig::default(),
            steps,
            current_step: 1,
            commitment_hash: None,
            proof_hash: None,
            block_height: None,
            started_at: None,
            completed_at: None,
        }
    }

    /// Create the 8-step settlement flow
    fn create_steps() -> Vec<SettlementStep> {
        vec![
            SettlementStep {
                step_number: 1,
                name: "Anchor Settlement Commitment".to_string(),
                description: "FortressDigital calls FlowCortex AnchorCommitment API".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 2,
                name: "Wait for Blockchain Confirmation".to_string(),
                description: "L1 node confirms commitment anchoring at block height".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 3,
                name: "Submit STARK Proof".to_string(),
                description: "ProofCortex submits cryptographic proof to FlowCortex".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 4,
                name: "Verify Proof".to_string(),
                description: "Verifier capsule validates STARK proof correctness".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 5,
                name: "Mint FloweR Tokens".to_string(),
                description: "Mint stablecoins to receiving bank (Bank B)".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 6,
                name: "Burn Collateral".to_string(),
                description: "Burn settlement collateral from sending bank (Bank A)".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 7,
                name: "Update Settlement Status".to_string(),
                description: "Mark settlement as COMPLETE in ledger".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
            SettlementStep {
                step_number: 8,
                name: "Emit Completion Event".to_string(),
                description: "Broadcast settlement.completed event to subscribers".to_string(),
                status: SettlementStepStatus::Pending,
                timestamp: None,
                error: None,
            },
        ]
    }

    /// Start the settlement scenario
    pub fn start(&mut self, timestamp: u64) {
        self.started_at = Some(timestamp);
        self.current_step = 1;
        if let Some(step) = self.steps.first_mut() {
            step.status = SettlementStepStatus::InProgress;
        }
    }

    /// Complete current step and move to next
    pub fn complete_step(&mut self, step_number: u8, timestamp: u64) -> Result<(), String> {
        if step_number != self.current_step {
            return Err(format!(
                "Cannot complete step {}. Current step is {}",
                step_number, self.current_step
            ));
        }

        if let Some(step) = self.steps.get_mut((step_number - 1) as usize) {
            step.status = SettlementStepStatus::Completed;
            step.timestamp = Some(timestamp);
        }

        // Move to next step
        if step_number < 8 {
            self.current_step += 1;
            if let Some(next_step) = self.steps.get_mut(step_number as usize) {
                next_step.status = SettlementStepStatus::InProgress;
            }
        } else {
            // All steps completed
            self.completed_at = Some(timestamp);
        }

        Ok(())
    }

    /// Mark step as failed
    pub fn fail_step(&mut self, step_number: u8, error: String, timestamp: u64) {
        if let Some(step) = self.steps.get_mut((step_number - 1) as usize) {
            step.status = SettlementStepStatus::Failed;
            step.error = Some(error);
            step.timestamp = Some(timestamp);
        }
    }

    /// Check if scenario is complete
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> u8 {
        let completed_steps = self
            .steps
            .iter()
            .filter(|s| s.status == SettlementStepStatus::Completed)
            .count();
        ((completed_steps as f64 / self.steps.len() as f64) * 100.0) as u8
    }

    /// Get current step details
    pub fn get_current_step(&self) -> Option<&SettlementStep> {
        if self.current_step > 0 && (self.current_step as usize) <= self.steps.len() {
            self.steps.get((self.current_step - 1) as usize)
        } else {
            None
        }
    }
}

// ============================================================================
// Phase 13.4: Demo Data Fixtures
// ============================================================================

/// Demo data fixtures for testing and demonstration
/// Subtask 13.4: Create demo data fixtures
pub struct DemoDataFixtures;

impl DemoDataFixtures {
    /// Generate 10 sample settlements with varying amounts
    pub fn generate_sample_settlements() -> Vec<DemoSettlementConfig> {
        vec![
            DemoSettlementConfig::with_amount(50_000_000_00),  // ₹50M
            DemoSettlementConfig::with_amount(25_000_000_00),  // ₹25M
            DemoSettlementConfig::with_amount(100_000_000_00), // ₹100M
            DemoSettlementConfig::with_amount(75_000_000_00),  // ₹75M
            DemoSettlementConfig::with_amount(10_000_000_00),  // ₹10M
            DemoSettlementConfig::with_amount(150_000_000_00), // ₹150M
            DemoSettlementConfig::with_amount(5_000_000_00),   // ₹5M
            DemoSettlementConfig::with_amount(200_000_000_00), // ₹200M
            DemoSettlementConfig::with_amount(30_000_000_00),  // ₹30M
            DemoSettlementConfig::with_amount(90_000_000_00),  // ₹90M
        ]
    }

    /// Generate sample commitment hash
    pub fn sample_commitment_hash(settlement_id: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("commitment_{}", settlement_id));
        format!("{:x}", hasher.finalize())
    }

    /// Generate sample proof hash
    pub fn sample_proof_hash(commitment_hash: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("proof_{}", commitment_hash));
        format!("{:x}", hasher.finalize())
    }

    /// Generate sample proof data (deterministic for testing)
    pub fn sample_proof_data(seed: u8) -> Vec<u8> {
        // Generate proof data where last byte determines validation result
        // Even last byte = valid, odd = invalid
        let mut proof = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        proof.push(seed);  // Seed determines if proof is valid
        proof
    }

    /// Create pre-computed verification results
    pub fn precomputed_verification_results() -> HashMap<String, (bool, String)> {
        let mut results = HashMap::new();
        
        // commitment_hash -> (verified, reason)
        results.insert(
            "valid_commitment_001".to_string(),
            (true, "Proof verified successfully".to_string()),
        );
        results.insert(
            "valid_commitment_002".to_string(),
            (true, "Proof verified successfully".to_string()),
        );
        results.insert(
            "invalid_commitment_003".to_string(),
            (false, "Proof verification failed: invalid signature".to_string()),
        );
        
        results
    }

    /// Generate historic event log for demo
    pub fn generate_historic_events() -> Vec<DemoEventFixture> {
        vec![
            DemoEventFixture {
                event_id: "evt_001".to_string(),
                event_type: "commitment.anchored".to_string(),
                commitment_hash: "abc123".to_string(),
                timestamp: 1708704000,  // Feb 23, 2026
                block_height: 1000,
                details: "Settlement commitment anchored".to_string(),
            },
            DemoEventFixture {
                event_id: "evt_002".to_string(),
                event_type: "proof.submitted".to_string(),
                commitment_hash: "abc123".to_string(),
                timestamp: 1708704060,
                block_height: 1001,
                details: "STARK proof submitted for verification".to_string(),
            },
            DemoEventFixture {
                event_id: "evt_003".to_string(),
                event_type: "proof.verified".to_string(),
                commitment_hash: "abc123".to_string(),
                timestamp: 1708704120,
                block_height: 1002,
                details: "Proof verified successfully".to_string(),
            },
        ]
    }
}

/// Demo event fixture structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoEventFixture {
    pub event_id: String,
    pub event_type: String,
    pub commitment_hash: String,
    pub timestamp: u64,
    pub block_height: u64,
    pub details: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_settlement_config() {
        let config = DemoSettlementConfig::default();
        assert_eq!(config.amount, 50_000_000_00);
        assert_eq!(config.formatted_amount(), "₹50000000");
        assert_eq!(config.settlement_window, "T+0");
    }

    #[test]
    fn test_flower_config() {
        let config = FloweRStablecoinConfig::default();
        assert_eq!(config.decimals, 6);
        assert_eq!(config.total_supply, 250_000_000_000_000);
        
        // Test conversion: 100 paise (1 INR) = 1,000,000 FLOWER base units
        let flower = config.inr_to_flower(100);
        assert_eq!(flower, 1_000_000);
        
        // Test reverse
        let inr = config.flower_to_inr(1_000_000);
        assert_eq!(inr, 100);
    }

    #[test]
    fn test_demo_scenario() {
        let config = DemoSettlementConfig::default();
        let mut scenario = DemoSettlementScenario::new("test_001".to_string(), config);
        
        assert_eq!(scenario.steps.len(), 8);
        assert_eq!(scenario.current_step, 1);
        assert_eq!(scenario.completion_percentage(), 0);
        
        // Start scenario
        scenario.start(1000);
        assert_eq!(scenario.started_at, Some(1000));
        
        // Complete first step
        scenario.complete_step(1, 1010).unwrap();
        assert_eq!(scenario.current_step, 2);
        assert_eq!(scenario.completion_percentage(), 12);
        
        // Complete remaining steps
        for step in 2..=8 {
            scenario.complete_step(step, 1000 + step as u64 * 10).unwrap();
        }
        
        assert!(scenario.is_complete());
        assert_eq!(scenario.completion_percentage(), 100);
    }

    #[test]
    fn test_demo_fixtures() {
        let settlements = DemoDataFixtures::generate_sample_settlements();
        assert_eq!(settlements.len(), 10);
        
        let commitment_hash = DemoDataFixtures::sample_commitment_hash("test");
        assert_eq!(commitment_hash.len(), 64);  // SHA256 hex string
        
        let proof_data = DemoDataFixtures::sample_proof_data(10);  // Even seed = valid
        assert_eq!(proof_data.last(), Some(&10));
    }
}
