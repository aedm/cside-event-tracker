use axum::{Json, http::StatusCode, response::IntoResponse};
use tracing::warn;

use crate::storage::{RetrieveError, StoreError};

/// Error type for the REST API.
///
/// This error type is used to convert errors into HTTP responses.
/// The standard error response looks like this:
///
/// ```json
/// {
///     "error": "ERROR_CODE",
///     "message": "Error message"
/// }
/// ```
#[derive(Debug, thiserror::Error, strum::AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AppError {
    #[error("Invalid event type: '{0}'")]
    InvalidEventType(String),

    #[error("Result too large, limit is {0}")]
    ResultTooLarge(u64),
}

/// Converts errors into HTTP responses.
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // Error code is the enum variant name in SCREAMING_SNAKE_CASE.
        let error_code = self.as_ref();
        let message = self.to_string();
        let status_code = StatusCode::INTERNAL_SERVER_ERROR;
        let json = serde_json::json!({ "error": error_code, "message": message });

        warn!("Returning error {error_code}: {message}");
        (status_code, Json(json)).into_response()
    }
}

/// Converts storage errors into application errors.
impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::InvalidEventType(event_type) => AppError::InvalidEventType(event_type),
        }
    }
}

/// Converts retrieval errors into application errors.
impl From<RetrieveError> for AppError {
    fn from(error: RetrieveError) -> Self {
        match error {
            RetrieveError::ResultTooLarge(n) => AppError::ResultTooLarge(n),
        }
    }
}
