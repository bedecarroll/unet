//! Server error types and HTTP response handling

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use unet_core::prelude::*;

/// Server error type for HTTP handlers
#[derive(Error, Debug)]
pub enum ServerError {
    /// Core library error
    #[error("Core error: {0}")]
    Core(#[from] unet_core::Error),

    /// DataStore error
    #[error("DataStore error: {0}")]
    DataStore(#[from] DataStoreError),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Bad request error
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error for other types
    #[error("Error: {0}")]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            Self::DataStore(DataStoreError::NotFound { .. }) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND")
            }
            Self::DataStore(DataStoreError::ValidationError { .. }) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR")
            }
            Self::DataStore(DataStoreError::ConstraintViolation { .. }) => {
                (StatusCode::CONFLICT, "CONSTRAINT_VIOLATION")
            }
            Self::DataStore(DataStoreError::ConnectionError { .. }) => {
                (StatusCode::SERVICE_UNAVAILABLE, "CONNECTION_ERROR")
            }
            Self::DataStore(DataStoreError::Timeout { .. }) => {
                (StatusCode::REQUEST_TIMEOUT, "TIMEOUT")
            }
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR"),
            Self::Other(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let error = serde_json::json!({
            "error": self.to_string(),
            "code": error_code,
            "success": false
        });
        (status, Json(error)).into_response()
    }
}

/// Server result type
pub type ServerResult<T> = std::result::Result<T, ServerError>;
