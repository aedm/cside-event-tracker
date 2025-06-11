mod app_error;
mod handlers;

use anyhow::{Context, Result};
use axum::{Router, response::IntoResponse, routing::get};
use std::sync::Arc;

use crate::{
    server::handlers::{get_events, post_event},
    storage::{InMemoryStorage, Storage},
};

/// Default port for the server
const PORT: u16 = 3000;

/// Shared application state.
struct AppState {
    store: Arc<dyn Storage + Send + Sync + 'static>,
}

/// Dummy handler to show the server is running.
async fn welcome() -> impl IntoResponse {
    "I'm completely operational, and all my circuits are functioning perfectly."
}

/// Creates a new server with the default storage. Used for testing, too.
pub fn make_server() -> Router {
    let store = Arc::new(InMemoryStorage::new());
    let shared_state = Arc::new(AppState { store });
    Router::new()
        .route("/events", get(get_events).post(post_event))
        .route("/", get(welcome))
        .with_state(shared_state)
}

/// Starts the server on the default port.
pub async fn serve() -> Result<()> {
    let app = make_server();

    println!("Listening on http://localhost:{}", PORT);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .with_context(|| format!("Failed to bind to port {PORT}"))?;

    axum::serve(listener, app)
        .await
        .with_context(|| "Failed to start server")
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;

    use crate::{event::Event, server::make_server};

    fn make_test_server() -> TestServer {
        let app = make_server();
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_single_event() {
        let server = make_test_server();
        let event = Event {
            event_type: "test".to_string(),
            timestamp: 42,
            payload: serde_json::json!({"test": "data"}),
        };
        let response = server.post("/events").json(&event).await;
        assert_eq!(response.status_code(), 200);

        let events = server.get("/events").await;
        assert_eq!(events.status_code(), 200);
        let events = events.json::<Vec<Event>>();
        assert_eq!(events, vec![event]);
    }
}
