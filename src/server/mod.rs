mod app_error;
mod handlers;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use strum::{AsRefStr, EnumDiscriminants, EnumString, IntoStaticStr};
use thiserror::Error;

use crate::{
    event::Event,
    server::handlers::{get_events, post_event},
    storage::{InMemoryStorage, RetrieveError, Storage, StoreError},
};

struct AppState {
    store: Arc<dyn Storage + Send + Sync + 'static>,
}

async fn welcome() -> impl IntoResponse {
    "I'm completely operational, and all my circuits are functioning perfectly."
}

pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let shared_state = Arc::new(AppState {
        store: Arc::new(InMemoryStorage::new()),
    });

    let app = Router::new()
        .route("/events", get(get_events).post(post_event))
        .route("/", get(welcome))
        .with_state(shared_state);

    let port = 3000;
    println!("Listening on http://localhost:{port}");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
