use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use askama::Template;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> impl IntoResponse {
    let body = IndexTemplate {}.render().unwrap();
    Html(body)
}

/// Reverse-proxy: forward /api/* to FlowCortex L1 backend.
/// The path after /api/ is sent as-is to the L1 node.
/// This avoids browser cross-origin self-signed-cert rejections.
async fn api_proxy(req: Request<Body>) -> impl IntoResponse {
    let l1_base = std::env::var("L1_API_BASE")
        .unwrap_or_else(|_| "https://127.0.0.1:3000".to_string());

    // Strip the /api prefix
    let path = req.uri().path().strip_prefix("/api").unwrap_or(req.uri().path());
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("{}{}{}", l1_base, path, query);
    let method_str = req.method().as_str().to_string();

    // Build upstream request — skip TLS verification (self-signed certs)
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let body_bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }
    };

    let rmethod: reqwest::Method = reqwest::Method::from_bytes(method_str.as_bytes()).unwrap();
    let upstream = client
        .request(rmethod, &url)
        .header("Content-Type", "application/json")
        .body(body_bytes.to_vec())
        .send()
        .await;

    match upstream {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();
            let bytes = resp.bytes().await.unwrap_or_default();
            (
                status,
                [
                    ("content-type", content_type.as_str().to_string()),
                    ("access-control-allow-origin", "*".to_string()),
                ],
                bytes,
            )
                .into_response()
        }
        Err(e) => {
            (StatusCode::BAD_GATEWAY, format!("upstream error: {}", e)).into_response()
        }
    }
}

// simple static files handler; askama templates are baked at compile time
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/{*rest}", axum::routing::any(api_proxy))
        .nest_service(
            "/static",
            ServeDir::new("static").precompressed_gzip(),
        )
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-cache, must-revalidate"),
        ));

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".to_string());

    let tls_enabled = matches!(
        std::env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );

    if tls_enabled {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("install rustls ring CryptoProvider");
        let cert_path = std::env::var("TLS_CERT_PATH")
            .unwrap_or_else(|_| "../../certs/explorer/explorer.crt".to_string());
        let key_path = std::env::var("TLS_KEY_PATH")
            .unwrap_or_else(|_| "../../certs/explorer/explorer.key".to_string());
        println!(
            "Explorer UI listening on https://{} (cert={}, key={})",
            bind_addr, cert_path, key_path
        );
        let tls_config =
            axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
                .await
                .unwrap();
        axum_server::bind_rustls(bind_addr.parse().unwrap(), tls_config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        println!("Explorer UI listening on http://{}", bind_addr);
        let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
