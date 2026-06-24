use super::*;
use crate::snmp::config::SnmpCredentials;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn create_test_config() -> SessionConfig {
    SessionConfig {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
        version: 2,
        credentials: SnmpCredentials::Community {
            community: "test-community".to_string(),
        },
        timeout: Duration::from_secs(5),
        retries: 3,
        max_vars_per_request: 50,
    }
}

#[test]
fn test_session_creation() {
    let config = create_test_config();
    let session = SnmpSession::new(config.clone());

    assert_eq!(session.config().address, config.address);
    assert!(!session.id().is_nil());
}

#[tokio::test]
async fn test_session_health() {
    let config = create_test_config();
    let session = SnmpSession::new(config);

    assert!(!session.is_healthy(Duration::from_secs(300)).await);

    session.update_success_timestamp(true).await;

    assert!(session.is_healthy(Duration::from_secs(300)).await);
}

#[tokio::test]
async fn test_connection_attempts() {
    let config = create_test_config();
    let session = SnmpSession::new(config);

    assert_eq!(session.connection_attempts().await, 0);

    session.increment_connection_attempts().await;
    assert_eq!(session.connection_attempts().await, 1);

    session.increment_connection_attempts().await;
    assert_eq!(session.connection_attempts().await, 2);
}

#[tokio::test]
async fn test_session_clone() {
    let config = create_test_config();
    let session1 = SnmpSession::new(config);
    let session2 = session1.clone();

    assert_ne!(session1.id(), session2.id());
    assert_eq!(session1.config().address, session2.config().address);
    assert_eq!(session1.config().version, session2.config().version);
}

#[tokio::test]
async fn test_session_debug_format() {
    let config = create_test_config();
    let session = SnmpSession::new(config);

    let debug_str = format!("{session:?}");
    assert!(debug_str.contains("SnmpSession"));
    assert!(debug_str.contains("session_id"));
    assert!(debug_str.contains("config"));
}

#[tokio::test]
async fn test_update_success_timestamp_false() {
    let config = create_test_config();
    let session = SnmpSession::new(config);

    session.update_success_timestamp(false).await;
    assert!(!session.is_healthy(Duration::from_secs(300)).await);
}

#[tokio::test]
async fn test_session_health_expired() {
    let config = create_test_config();
    let session = SnmpSession::new(config);

    session.update_success_timestamp(true).await;

    assert!(session.is_healthy(Duration::from_secs(300)).await);
    assert!(!session.is_healthy(Duration::from_nanos(1)).await);
}

#[tokio::test]
async fn test_create_client_community() {
    let config = SessionConfig {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
        version: 2,
        credentials: SnmpCredentials::Community {
            community: "test-community".to_string(),
        },
        timeout: Duration::from_secs(5),
        retries: 3,
        max_vars_per_request: 50,
    };

    let result = SnmpSession::create_client(&config).await;
    match result {
        Ok(_) | Err(SnmpError::Protocol { .. }) => {}
        Err(error) => panic!("Unexpected error type: {error:?}"),
    }
}

#[tokio::test]
async fn test_create_client_requires_explicit_community() {
    let config = SessionConfig::default();

    let result = SnmpSession::create_client(&config).await;
    assert!(result.is_err());
    if let Err(SnmpError::Protocol { message }) = result {
        assert!(message.contains("SNMP community must be configured explicitly"));
    } else {
        panic!("Expected protocol error for missing SNMP community");
    }
}

#[tokio::test]
async fn test_create_client_user_based() {
    let config = SessionConfig {
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 161),
        version: 3,
        credentials: SnmpCredentials::UserBased {
            username: "test".to_string(),
            auth: Some(("SHA".to_string(), "auth".to_string())),
            privacy: Some(("AES".to_string(), "priv".to_string())),
        },
        timeout: Duration::from_secs(5),
        retries: 3,
        max_vars_per_request: 50,
    };

    let result = SnmpSession::create_client(&config).await;
    assert!(result.is_err());
    if let Err(SnmpError::Protocol { message }) = result {
        assert!(message.contains("SNMPv3 user-based security not supported"));
    } else {
        panic!("Expected Protocol error for SNMPv3");
    }
}

#[tokio::test]
async fn test_get_client() {
    let config = create_test_config();
    let mut session = SnmpSession::new(config);

    let result = session.get_client().await;
    match result {
        Ok(_) | Err(SnmpError::Protocol { .. }) => {}
        Err(error) => panic!("Unexpected error type: {error:?}"),
    }
}

#[test]
fn test_client_ref_returns_protocol_error_when_client_not_initialized() {
    let session = SnmpSession::new(create_test_config());
    let result = session.client_ref();

    assert!(result.is_err());
    if let Err(SnmpError::Protocol { message }) = result {
        assert!(message.contains("SNMP client not initialized"));
    } else {
        panic!("Expected protocol error when client is missing");
    }
}

#[test]
fn test_session_const_methods() {
    let config = create_test_config();
    let session = SnmpSession::new(config.clone());

    let _id = session.id();
    let _config = session.config();

    assert_eq!(session.config().address, config.address);
}
