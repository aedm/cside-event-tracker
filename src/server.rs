use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};

use crate::{
    event::Event,
    storage::{InMemoryStorage, Storage},
};

struct AppState {
    store: Arc<dyn Storage + Send + Sync + 'static>,
}

#[axum::debug_handler]
async fn get_events(State(store): State<Arc<AppState>>) -> impl IntoResponse {
    let events = store.store.get_events(None, None, None);
    Json(events)
}

#[axum::debug_handler]
async fn post_event(
    State(store): State<Arc<AppState>>,
    Json(event): Json<Event>,
) -> impl IntoResponse {
    store.store.store(event);
    ""
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
