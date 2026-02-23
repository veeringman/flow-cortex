/// Demo Test Program
/// Tests Phase 13 Demo-Specific Features
///
/// This program creates a test settlement and executes all 8 steps
/// to verify the demo orchestration works correctly

use flowcortex_l1::demo::*;
use flowcortex_l1::grpc::demo::*;
use flowcortex_l1::node::Node;
use flowcortex_l1::rpc::SharedNode;
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== FlowCortex Phase 13 Demo Test ===\n");

    // Create a test node
    let node = Arc::new(Mutex::new(Node::new("admin".to_string())));
    let demo_service = DemoService::new(node.clone());

    // Test 1: Create Demo Settlement
    println!("Test 1: Creating demo settlement...");
    let create_req = CreateDemoSettlementRequest {
        scenario_id: "test_settlement_001".to_string(),
        amount: Some(50_000_000_00),  // ₹50M
        sender_id: None,
        receiver_id: None,
    };

    let create_resp = demo_service.create_demo_settlement(create_req);
    assert!(create_resp.success, "Failed to create settlement");
    println!("✓ Settlement created: {}", create_resp.scenario_id);
    println!("  Amount: {}", create_resp.config.formatted_amount());
    println!("  Sender: {}", create_resp.config.sender.name);
    println!("  Receiver: {}", create_resp.config.receiver.name);
    println!("  Steps: {}", create_resp.steps.len());
    println!();

    // Test 2: Check Initial Status
    println!("Test 2: Checking initial status...");
    let status_req = GetSettlementStatusRequest {
        scenario_id: "test_settlement_001".to_string(),
    };
    let status_resp = demo_service.get_settlement_status(status_req.clone());
    assert!(status_resp.success);
    println!("✓ Status retrieved");
    println!("  Current step: {}", status_resp.current_step);
    println!("  Completion: {}%", status_resp.completion_percentage);
    println!("  Complete: {}", status_resp.is_complete);
    println!();

    // Test 3: Execute Steps 1-8
    println!("Test 3: Executing all 8 steps...");
    for step in 1..=8 {
        println!("\n  Step {}/8:", step);
        let step_req = ExecuteStepRequest {
            scenario_id: "test_settlement_001".to_string(),
            step_number: step,
        };
        let step_resp = demo_service.execute_step(step_req);
        
        if !step_resp.success {
            panic!("Step {} failed: {}", step, step_resp.message);
        }
        
        println!("  ✓ {}", step_resp.step_name);
        println!("    Message: {}", step_resp.message);
        
        if let Some(hash) = &step_resp.commitment_hash {
            println!("    Commitment Hash: {}...", &hash[..16]);
        }
        if let Some(hash) = &step_resp.proof_hash {
            println!("    Proof Hash: {}...", &hash[..16]);
        }
        if let Some(height) = step_resp.block_height {
            println!("    Block Height: {}", height);
        }
    }
    println!();

    // Test 4: Check Final Status
    println!("Test 4: Checking final status...");
    let final_status = demo_service.get_settlement_status(status_req.clone());
    assert!(final_status.success);
    assert!(final_status.is_complete, "Settlement should be complete");
    assert_eq!(final_status.completion_percentage, 100, "Should be 100% complete");
    println!("✓ Settlement completed successfully");
    println!("  Current step: {}", final_status.current_step);
    println!("  Completion: {}%", final_status.completion_percentage);
    println!("  Started at: {:?}", final_status.started_at);
    println!("  Completed at: {:?}", final_status.completed_at);
    println!();

    // Test 5: List All Settlements
    println!("Test 5: Listing all settlements...");
    let list_req = ListSettlementsRequest { limit: Some(10) };
    let list_resp = demo_service.list_settlements(list_req);
    assert!(list_resp.success);
    println!("✓ Found {} settlement(s)", list_resp.total_count);
    for (i, summary) in list_resp.scenarios.iter().enumerate() {
        println!("  {}. {} - {} → {} ({}%)", 
            i + 1, 
            summary.scenario_id, 
            summary.sender, 
            summary.receiver,
            summary.completion_percentage
        );
    }
    println!();

    // Test 6: Get Events
    println!("Test 6: Retrieving events...");
    let events_req = GetEventsRequest {
        scenario_id: Some("test_settlement_001".to_string()),
        limit: Some(20),
    };
    let events_resp = demo_service.get_events(events_req);
    assert!(events_resp.success);
    println!("✓ Found {} event(s)", events_resp.total_count);
    for (i, event) in events_resp.events.iter().enumerate() {
        println!("  {}. [{}] {} at block {}", 
            i + 1, 
            event.event_type,
            event.details,
            event.block_height
        );
    }
    println!();

    // Test 7: Get Dashboard Stats
    println!("Test 7: Retrieving dashboard statistics...");
    let stats = demo_service.get_dashboard_stats();
    println!("✓ Dashboard stats:");
    println!("  Total Settlements: {}", stats.total_settlements);
    println!("  Completed: ", stats.completed_settlements);
    println!("  In Progress: {}", stats.in_progress_settlements);
    println!("  Total Events: {}", stats.total_events);
    println!("  Total Commitments: {}", stats.total_commitments);
    println!("  Total Proofs: {}", stats.total_proofs);
    println!("  Total Value: {}", stats.total_value_formatted);
    println!("  Block Height: {}", stats.block_height);
    println!();

    // Test 8: Test Auto-Execute on New Settlement
    println!("Test 8: Testing auto-execute mode...");
    let auto_create_req = CreateDemoSettlementRequest {
        scenario_id: "test_auto_001".to_string(),
        amount: Some(100_000_000_00),  // ₹100M
        sender_id: None,
        receiver_id: None,
    };
    demo_service.create_demo_settlement(auto_create_req);
    
    let auto_resp = demo_service.auto_execute_settlement("test_auto_001".to_string());
    assert!(auto_resp.success);
    assert!(auto_resp.is_complete);
    println!("✓ Auto-execute completed");
    println!("  Completion: {}%", auto_resp.completion_percentage);
    println!();

    // Test 9: Test FloweR Stablecoin Calculations
    println!("Test 9: Testing FloweR stablecoin calculations...");
    let flower_config = FloweRStablecoinConfig::default();
    
    // Test INR to FLOWER conversion
    let inr_amount = 50_000_000_00;  // ₹50M in paise
    let flower_amount = flower_config.inr_to_flower(inr_amount);
    println!("✓ INR to FLOWER conversion:");
    println!("  ₹{} = {}", inr_amount / 100, flower_config.format_flower(flower_amount));
    
    // Test FLOWER to INR conversion
    let converted_back = flower_config.flower_to_inr(flower_amount);
    assert_eq!(inr_amount, converted_back, "Conversion round-trip failed");
    println!("  Round-trip verification: ✓");
    println!();

    // Test 10: Test Demo Data Fixtures
    println!("Test 10: Testing demo data fixtures...");
    let sample_settlements = DemoDataFixtures::generate_sample_settlements();
    println!("✓ Generated {} sample settlements", sample_settlements.len());
    
    let commitment_hash = DemoDataFixtures::sample_commitment_hash("test");
    println!("  Sample commitment hash: {}...", &commitment_hash[..16]);
    
    let proof_hash = DemoDataFixtures::sample_proof_hash(&commitment_hash);
    println!("  Sample proof hash: {}...", &proof_hash[..16]);
    
    let proof_data = DemoDataFixtures::sample_proof_data(10);  // Even = valid
    println!("  Sample proof data length: {} bytes", proof_data.len());
    println!();

    println!("=== All Tests Passed! ===\n");
    println!("Phase 13 Demo Features Status: ✅ WORKING");
    println!();
    println!("Summary:");
    println!("  ✓ Mock settlement configuration");
    println!("  ✓ FloweR stablecoin module");
    println!("  ✓ 8-step settlement orchestrator");
    println!("  ✓ Demo data fixtures");
    println!("  ✓ Demo service APIs");
    println!("  ✓ Event streaming");
    println!("  ✓ Dashboard statistics");
    println!("  ✓ Auto-execute mode");
}
