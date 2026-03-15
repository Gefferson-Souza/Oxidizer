#![allow(unused)]

use axum::Router;
use tokio::net::TcpListener;
use std::sync::Arc;
use axum::Extension;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_service = Arc::new(tyrus_app::app_service::AppService::new_di());
    let app_controller = Arc::new(tyrus_app::app_controller::AppController::new_di(app_service.clone()));

    // Build router
    let app = axum::Router::new()
        .merge(tyrus_app::app_controller::AppController::router(app_controller.clone()))
        .layer(Extension(app_service.clone()))
        .layer(Extension(app_controller.clone()));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service())
        .await?;
    Ok(())
}
