//! Error conversion implementations for μNet Core Error types

use super::Error;

// Implement From traits for common error conversions
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            path: "<unknown>".to_string(),
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            format: "JSON".to_string(),
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let converted: Error = io_error.into();

        match converted {
            Error::Io { path, message, .. } => {
                assert_eq!(path, "<unknown>");
                assert!(message.contains("File not found"));
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_io_error_permission_denied() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let converted: Error = io_error.into();

        match converted {
            Error::Io { path, message, .. } => {
                assert_eq!(path, "<unknown>");
                assert!(message.contains("Access denied"));
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_serde_json_error_conversion() {
        let json_str = r#"{"invalid": json syntax}"#;
        let json_error: serde_json::Error =
            serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let converted: Error = json_error.into();

        match converted {
            Error::Serialization {
                format, message, ..
            } => {
                assert_eq!(format, "JSON");
                assert!(!message.is_empty());
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_serde_json_error_serialization() {
        use serde_json::Value;

        let invalid_json = r#"{"incomplete": }"#;
        let json_error: serde_json::Error =
            serde_json::from_str::<Value>(invalid_json).unwrap_err();
        let converted: Error = json_error.into();

        match converted {
            Error::Serialization {
                format, message, ..
            } => {
                assert_eq!(format, "JSON");
                assert!(message.contains("expected"));
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_error_chain_preservation() {
        let io_error = io::Error::new(io::ErrorKind::TimedOut, "Connection timeout");
        let converted: Error = io_error.into();

        match converted {
            Error::Io { source, .. } => {
                assert_eq!(source.kind(), io::ErrorKind::TimedOut);
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_multiple_conversions() {
        let io_error1 = io::Error::new(io::ErrorKind::NotFound, "File 1 not found");
        let io_error2 = io::Error::new(io::ErrorKind::PermissionDenied, "File 2 access denied");

        let converted1: Error = io_error1.into();
        let converted2: Error = io_error2.into();

        match (&converted1, &converted2) {
            (Error::Io { message: msg1, .. }, Error::Io { message: msg2, .. }) => {
                assert!(msg1.contains("File 1 not found"));
                assert!(msg2.contains("File 2 access denied"));
            }
            _ => panic!("Expected Io errors"),
        }
    }
}
