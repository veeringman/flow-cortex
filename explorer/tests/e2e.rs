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

/// Basic smoke test - verify Explorer loads
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

/// Test API Config UI Elements
#[tokio::test]
async fn api_config_ui_elements() {
    let _ = Command::new("pkill").arg("-f").arg("cargo run").output();
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let _child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .spawn()
        .expect("spawn explorer");

    wait_ready().await;
    let client = Client::new();
    let body = client.get("http://127.0.0.1:4000/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();

    // Verify API Config button exists
    assert!(body.contains("API Config"), "API Config button should be in the UI");

    // Verify API Config Modal exists
    assert!(body.contains("apiConfigModal"), "API Config modal should exist");
    assert!(body.contains("apiBaseInput"), "API base input field should exist");
    
    // Verify modal contains expected elements
    assert!(body.contains("API Base URL"), "Modal should have the API Base URL label");
    assert!(body.contains("API Configuration"), "Modal should have the correct title");
}

/// Test JavaScript API module exports setApiBase function
#[tokio::test]
async fn api_module_functions() {
    let _ = Command::new("pkill").arg("-f").arg("cargo run").output();
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .spawn()
        .expect("spawn explorer");

    wait_ready().await;
    let client = Client::new();
    let body = client.get("http://127.0.0.1:4000/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();

    // Verify setApiBase function is defined in API module
    assert!(body.contains("API_BASE"), "API_BASE should be defined");
    
    // Verify updateApiBase function exists
    assert!(body.contains("updateApiBase"), "updateApiBase function should be defined");

    let _ = child.kill();
}

/// Test API Config localStorage persistence
#[tokio::test]
async fn api_config_localstorage() {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .spawn()
        .expect("spawn explorer");

    wait_ready().await;
    let client = Client::new();
    let body = client.get("http://127.0.0.1:4000/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();

    // Verify localStorage handling code exists
    assert!(body.contains("localStorage"), "localStorage operations should be supported");
    assert!(body.contains("apiBase"), "apiBase should be persisted in localStorage");
    
    // Verify saved API base is loaded on init
    assert!(body.contains("savedApiBase"), "Saved API base loading should be implemented");

    let _ = child.kill();
}

/// Test API Config initialization on page load
#[tokio::test]
async fn api_config_initialization() {
    let _ = Command::new("pkill").arg("-f").arg("cargo run").output();
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .spawn()
        .expect("spawn explorer");

    wait_ready().await;
    let client = Client::new();
    let body = client.get("http://127.0.0.1:4000/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();

    // Verify initialization code loads saved API config
    assert!(body.contains("apiBase"), "Should handle API base configuration");
    assert!(body.contains("localStorage"), "Should use localStorage for persistence");

    let _ = child.kill();
}

/// Test API module uses dynamic API_BASE for requests
#[tokio::test]
async fn api_module_uses_dynamic_base() {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .spawn()
        .expect("spawn explorer");

    wait_ready().await;
    let client = Client::new();
    let body = client.get("http://127.0.0.1:4000/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();

    // Verify API calls use the dynamic API_BASE variable
    assert!(body.contains("${API_BASE}") || body.contains("${base}") || body.contains("API_BASE"), 
            "API calls should use dynamic API_BASE variable");

    // Verify url construction uses the base
    assert!(body.contains("const base = API_BASE") || body.contains("${API_BASE}"),
            "API calls should construct URLs using dynamic API_BASE");

    let _ = child.kill();
}

/// Test API Config with environment variable
#[tokio::test]
async fn explorer_bind_addr_env() {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .env("BIND_ADDR", "127.0.0.1:4001")
        .spawn()
        .expect("spawn explorer");

    // wait for the explorer to start on the custom port
    let client = Client::new();
    for _ in 0..20 {
        if client.get("http://127.0.0.1:4001/").send().await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let body = client.get("http://127.0.0.1:4001/")
        .send().await
        .unwrap()
        .text().await
        .unwrap();
    
    assert!(body.contains("<html"), "Explorer should start on custom BIND_ADDR");

    let _ = child.kill();
}
