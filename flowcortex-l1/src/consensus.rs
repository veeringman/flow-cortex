use crate::rpc::SharedNode;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};

/// Start a very simple block producer that periodically takes whatever is
/// in the transaction pool and creates a block. It runs in the background and
/// logs each produced block to stdout.
///
/// This represents the "consensus" layer in the most minimal sense: a single
/// authority that decides when to cut a new block.
///
/// Returns a JoinHandle that can be awaited or aborted by the caller.
pub fn start_block_producer(node: SharedNode, interval: Duration) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = time::interval(interval);
        loop {
            ticker.tick().await;
            let mut n = node.lock().unwrap();
            if !n.pool.is_empty() {
                let block = n.create_block();
                let _ = n.save("node_state.json");
                println!("[producer] new block height {} ({} txs)", block.height, block.transactions.len());
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::types::{Token, Transaction};
    use std::sync::{Arc, Mutex};
    use tokio::time::Duration;

    #[tokio::test]
    async fn producer_creates_block() {
        let admin = "admin".to_string();
        let mut node = Node::new(admin.clone());
        node.create_account(&admin);
        let shared = Arc::new(Mutex::new(node));
        // submit a tx to pool so producer has work
        {
            let mut n = shared.lock().unwrap();
            let _ = n.submit_transaction(&admin, Transaction {
                kind: crate::types::TransactionKind::Mint { to: "alice".to_string(), token: Token::Proof, amount: 1 },
                rw_set: Default::default(),
                proof: None,
            }).unwrap();
        }
        let handle = start_block_producer(shared.clone(), Duration::from_millis(50));
        // wait for a bit to allow producer to run
        tokio::time::sleep(Duration::from_millis(150)).await;
        // check that block was produced
        let n = shared.lock().unwrap();
        assert!(!n.blocks.is_empty());
        // stop background task
        handle.abort();
    }
}
