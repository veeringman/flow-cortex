use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use askama::Template;
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> impl IntoResponse {
    let body = IndexTemplate {}.render().unwrap();
    Html(body)
}

// simple static files handler; askama templates are baked at compile time
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"));

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
