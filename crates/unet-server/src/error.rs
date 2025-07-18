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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use unet_core::datastore::types::DataStoreError;

    #[test]
    fn test_server_error_from_core_error() {
        let core_error = unet_core::Error::Validation {
            field: "test".to_string(),
            message: "Invalid value".to_string(),
            value: None,
        };
        let server_error: ServerError = core_error.into();

        match server_error {
            ServerError::Core(_) => (),
            _ => panic!("Expected Core error"),
        }
    }

    #[test]
    fn test_server_error_from_datastore_error() {
        let ds_error = DataStoreError::NotFound {
            entity_type: "node".to_string(),
            id: "123".to_string(),
        };
        let server_error: ServerError = ds_error.into();

        match server_error {
            ServerError::DataStore(_) => (),
            _ => panic!("Expected DataStore error"),
        }
    }

    #[test]
    fn test_server_error_not_found() {
        let error = ServerError::NotFound("Resource not found".to_string());
        assert_eq!(error.to_string(), "Not found: Resource not found");
    }

    #[test]
    fn test_server_error_bad_request() {
        let error = ServerError::BadRequest("Invalid parameter".to_string());
        assert_eq!(error.to_string(), "Bad request: Invalid parameter");
    }

    #[test]
    fn test_server_error_internal() {
        let error = ServerError::Internal("Something went wrong".to_string());
        assert_eq!(
            error.to_string(),
            "Internal server error: Something went wrong"
        );
    }

    #[test]
    fn test_server_error_into_response_not_found_datastore() {
        let ds_error = DataStoreError::NotFound {
            entity_type: "node".to_string(),
            id: "123".to_string(),
        };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_server_error_into_response_validation_error() {
        let ds_error = DataStoreError::ValidationError {
            message: "Invalid data".to_string(),
        };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_server_error_into_response_constraint_violation() {
        let ds_error = DataStoreError::ConstraintViolation {
            message: "Unique constraint violated".to_string(),
        };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_server_error_into_response_connection_error() {
        let ds_error = DataStoreError::ConnectionError {
            message: "Database connection failed".to_string(),
        };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_server_error_into_response_timeout() {
        let ds_error = DataStoreError::Timeout { seconds: 30 };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn test_server_error_into_response_not_found() {
        let server_error = ServerError::NotFound("Item not found".to_string());
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_server_error_into_response_bad_request() {
        let server_error = ServerError::BadRequest("Invalid input".to_string());
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_server_error_into_response_internal() {
        let server_error = ServerError::Internal("Internal error".to_string());
        let response = server_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_server_result_ok() {
        let result: ServerResult<i32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected Ok(42), got error");
        }
    }

    #[test]
    fn test_server_result_err() {
        let result: ServerResult<i32> = Err(ServerError::NotFound("Test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_display_messages() {
        let errors = vec![
            ServerError::NotFound("test".to_string()),
            ServerError::BadRequest("test".to_string()),
            ServerError::Internal("test".to_string()),
        ];

        for error in errors {
            let message = error.to_string();
            assert!(!message.is_empty());
            assert!(message.contains("test"));
        }
    }

    #[test]
    fn test_datastore_error_transaction_error() {
        let ds_error = DataStoreError::TransactionError {
            message: "Transaction failed".to_string(),
        };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        // Transaction errors default to internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_datastore_error_internal_error() {
        let ds_error = DataStoreError::InternalError {
            message: "Internal operation failed".to_string(),
        };
        let server_error = ServerError::DataStore(ds_error);
        let response = server_error.into_response();
        // Internal errors default to internal server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
