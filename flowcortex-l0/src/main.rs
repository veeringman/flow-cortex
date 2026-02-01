use flowcortex_l0::{qct::QCTNode, types::Transaction, verify::verify_subtree};

#[tokio::main]
async fn main() {
    let mut root = QCTNode::new(vec![]);

    let txs = vec![
        Transaction {
            key: vec![0xAA, 0x01],
            amount: 100,
        },
        Transaction {
            key: vec![0xAA, 0x02],
            amount: -30,
        },
        Transaction {
            key: vec![0xBB, 0x01],
            amount: 50,
        },
    ];

    for tx in &txs {
        root.insert(tx, 0);
    }

    let ok = verify_subtree(&root).await;
    println!("QCT verification result: {}", ok);
}
