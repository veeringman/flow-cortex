use flowcortex_l1::chain_params;
use flowcortex_l1::consensus::start_block_producer;
use flowcortex_l1::node::Node;
use flowcortex_l1::rpc::make_router;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // admin account that will be allowed to mint tokens
    let admin = "admin".to_string();

    // try loading existing state, otherwise create fresh
    let node = if let Ok(saved) = Node::load("node_state.json") {
        println!("loaded node state from disk ({} blocks)", saved.blocks.len());
        Arc::new(Mutex::new(saved))
    } else {
        let n = Node::new(admin.clone());
        let n = Arc::new(Mutex::new(n));
        // create a few default accounts
        {
            let mut nlock = n.lock().unwrap();
            nlock.create_account(&admin);
            nlock.create_account(&"alice".to_string());
            nlock.create_account(&"bob".to_string());
        }
        n
    };

    // Block production interval: default from chain_params, override via BLOCK_INTERVAL_MS env var.
    let block_interval_ms = std::env::var("BLOCK_INTERVAL_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(chain_params::BLOCK_INTERVAL_MS);
    println!(
        "chain_id={} protocol={} block_interval={}ms",
        chain_params::CHAIN_ID_NUMERIC,
        chain_params::PROTOCOL_VERSION,
        block_interval_ms
    );
    let _producer_handle = start_block_producer(node.clone(), Duration::from_millis(block_interval_ms));

    let app = make_router(node.clone());

    // also start gRPC server on separate port (default 50051)
    let grpc_addr: std::net::SocketAddr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string()) // chain_params::DEFAULT_GRPC_ENDPOINT port
        .parse()
        .expect("invalid GRPC_ADDR");
    let grpc_node = node.clone();
    tokio::spawn(async move {
        println!("gRPC service listening on {}", grpc_addr);
        if let Err(e) = flowcortex_l1::grpc::serve_grpc(grpc_node, grpc_addr).await {
            eprintln!("gRPC server error: {}", e);
        }
    });

    println!("L1 node running with admin='{}'", admin);
    // determine bind address (allow override via BIND_ADDR env var)
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string()); // chain_params::DEFAULT_HTTP_ENDPOINT port
    println!("binding L1 node on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
