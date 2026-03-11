#![allow(unused)]

use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() {

    // Build router
    let app = axum::Router::new();

    let listener = TcpListener::bind("0.0.0.0:3000").await.expect("Failed to bind to address");
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server failed");
}
