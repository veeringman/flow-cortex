use tonic::transport::Channel;
use base64::Engine;

pub mod proto {
    tonic::include_proto!("l1");
}

use proto::l1_client::L1Client;
use proto::{BalanceRequest, Empty, CapsuleUploadRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the gRPC server
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    let mut client = L1Client::new(channel);

    println!("=== FlowCortex L1 gRPC Client Examples ===\n");

    // Example 1: Get Balance
    println!("1. Get Balance Example:");
    example_get_balance(&mut client).await?;

    // Example 2: List Blocks
    println!("\n2. List Blocks Example:");
    example_list_blocks(&mut client).await?;

    // Example 3: List Anchors
    println!("\n3. List Anchors Example:");
    example_list_anchors(&mut client).await?;

    // Example 4: Get Snapshot
    println!("\n4. Get Snapshot Example:");
    example_snapshot(&mut client).await?;

    // Example 5: Upload Capsule
    println!("\n5. Upload Capsule Example:");
    example_upload_capsule(&mut client).await?;

    // Example 6: List Capsules
    println!("\n6. List Capsules Example:");
    example_list_capsules(&mut client).await?;

    Ok(())
}

async fn example_get_balance(
    client: &mut L1Client<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(BalanceRequest {
        account: "admin".to_string(),
        token: "Proof".to_string(),
    });

    let response = client.get_balance(request).await?;
    let balance_resp = response.into_inner();
    
    println!("  Account: {}", balance_resp.account);
    println!("  Token: {}", balance_resp.token);
    println!("  Balance: {}", balance_resp.balance);

    Ok(())
}

async fn example_list_blocks(
    client: &mut L1Client<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(Empty {});

    let response = client.list_blocks(request).await?;
    let blocks_resp = response.into_inner();

    println!("  Total blocks: {}", blocks_resp.blocks.len());
    for (i, block) in blocks_resp.blocks.iter().enumerate() {
        println!("    Block {}: height={}", i, block.height);
    }

    Ok(())
}

async fn example_list_anchors(
    client: &mut L1Client<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(Empty {});

    let response = client.list_anchors(request).await?;
    let anchors_resp = response.into_inner();

    println!("  Total anchors: {}", anchors_resp.ids.len());
    for (i, id) in anchors_resp.ids.iter().enumerate() {
        println!("    Anchor {}: {}", i, id);
    }

    Ok(())
}

async fn example_snapshot(
    client: &mut L1Client<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(Empty {});

    let response = client.snapshot(request).await?;
    let snapshot_resp = response.into_inner();

    println!("  Snapshot root: {}", snapshot_resp.root);

    Ok(())
}

async fn example_upload_capsule(
    client: &mut L1Client<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Example capsule code (simple WASM or bytecode)
    let sample_code = b"example capsule code".to_vec();
    let encoded_code = base64::engine::general_purpose::STANDARD.encode(&sample_code);

    let request = tonic::Request::new(CapsuleUploadRequest {
        id: "capsule_example_1".to_string(),
        code: encoded_code.into_bytes(),
    });

    let response = client.upload_capsule(request).await?;
    let upload_resp = response.into_inner();

    println!("  Upload success: {}", upload_resp.success);
    if !upload_resp.error.is_empty() {
        println!("  Error: {}", upload_resp.error);
    }

    Ok(())
}

async fn example_list_capsules(
    client: &mut L1Client<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(Empty {});

    let response = client.list_capsules(request).await?;
    let capsules_resp = response.into_inner();

    println!("  Total capsules: {}", capsules_resp.capsules.len());
    for (i, capsule_id) in capsules_resp.capsules.iter().enumerate() {
        println!("    Capsule {}: {}", i, capsule_id);
    }

    Ok(())
}
