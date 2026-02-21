use reqwest::Client;
use std::process::{Child, Command, Stdio};
use std::{time::Duration};

async fn wait_ready() {
    let client = Client::new();
    // give the server ample time to start (compilation or slow CI may take a few seconds)
    for _ in 0..50 {
        if client.get("http://127.0.0.1:3000/blocks").send().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("server did not become ready in time");
}

#[tokio::test]
async fn l1_smoke_end_to_end() {
    // spawn the l1 server via `cargo run` so tests don't depend on build path
    // spawn the node using `cargo run` in the same crate so the binary is
    // built automatically. This may take a couple of seconds, which is why our
    // `wait_ready` timeout is generous.
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start l1 node");

    // the child will be killed at the end of this function

    wait_ready().await;
    let client = Client::new();

    client
        .post("http://127.0.0.1:3000/account")
        .json(&serde_json::json!({"account":"test1"}))
        .send()
        .await
        .unwrap();

    client
        .post("http://127.0.0.1:3000/mint")
        .json(&serde_json::json!({
            "caller": "admin",
            "to": "test1",
            "token": "proof",
            "amount": 100
        }))
        .send()
        .await
        .unwrap();

    let bal: serde_json::Value = client
        .get("http://127.0.0.1:3000/balance/test1/proof")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(bal["balance"].as_i64().unwrap() >= 100);

    client
        .post("http://127.0.0.1:3000/account")
        .json(&serde_json::json!({"account":"test2"}))
        .send()
        .await
        .unwrap();

    client
        .post("http://127.0.0.1:3000/transfer")
        .json(&serde_json::json!({
            "from": "test1",
            "to": "test2",
            "token": "proof",
            "amount": 25
        }))
        .send()
        .await
        .unwrap();

    let bal2: serde_json::Value = client
        .get("http://127.0.0.1:3000/balance/test2/proof")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(bal2["balance"].as_i64().unwrap(), 25);

    // shut down server
    let _ = child.kill();
    let _ = child.wait();
}
