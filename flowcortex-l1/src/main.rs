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

    // start minimal block producer that cuts a block every 5 seconds
    let _producer_handle = start_block_producer(node.clone(), Duration::from_secs(5));

    let app = make_router(node.clone());

    println!("L1 node running with admin='{}'", admin);
    // determine bind address (allow override via BIND_ADDR env var)
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    println!("binding L1 node on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
