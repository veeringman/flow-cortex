use reqwest::Client;
use std::{process::Command, time::Duration};

async fn wait_ready() {
    let client = Client::new();
    for _ in 0..20 {
        if client.get("http://127.0.0.1:4000/").send().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("explorer did not start in time");
}

#[tokio::test]
async fn explorer_smoke() {
    // spawn the explorer binary via cargo to ensure it is built
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .spawn()
        .expect("spawn explorer");

    // the child process will be killed after we finish

    wait_ready().await;
    let client = Client::new();
    let body = client.get("http://127.0.0.1:4000/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();
    assert!(body.contains("<html"));

    let _ = child.kill();
}
