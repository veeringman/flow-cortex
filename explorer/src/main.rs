use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use askama::Template;

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
    let app = Router::new().route("/", get(index));
    println!("Explorer UI listening on http://127.0.0.1:4000");
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".to_string());
    println!("Explorer UI listening on http://{}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
