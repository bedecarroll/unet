//! Server error types and HTTP response handling

use crate::api::ApiError;
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

    /// `DataStore` error
    #[error("DataStore error: {0}")]
    DataStore(#[from] DataStoreError),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Bad request error
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),
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
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let error = ApiError::new(self.to_string(), error_code.to_string());
        (status, Json(error)).into_response()
    }
}

/// Server result type
pub type ServerResult<T> = std::result::Result<T, ServerError>;
