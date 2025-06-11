use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use tracing::instrument;
use std::sync::Arc;

use crate::{
    event::Event,
    server::{AppState, app_error::AppError},
};

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    event_type: Option<String>,
    start: Option<u64>,
    end: Option<u64>,
}

/// Returns a list of events.
///
/// The list is filtered by event type and timestamp range, if specified.
#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_events(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<Event>>, AppError> {
    let result = state
        .store
        .get_events(params.event_type.as_deref(), params.start, params.end)
        .await
        .map_err(AppError::from)?;
    Ok(Json(result))
}

/// Inserts a new event into the event storage.
#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn post_event(
    State(state): State<Arc<AppState>>,
    Json(event): Json<Event>,
) -> Result<(), AppError> {
    state.store.store(event).await.map_err(AppError::from)?;
    Ok(())
}
