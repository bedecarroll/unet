//! Tests for SNMP configuration

#[cfg(test)]
mod snmp_config_tests {
    use super::super::{SessionConfig, SnmpClientConfig, SnmpCredentials};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::time::Duration;

    #[test]
    fn test_snmp_credentials_default() {
        let creds = SnmpCredentials::default();
        match creds {
            SnmpCredentials::Community { community } => {
                assert_eq!(community, "public");
            }
            SnmpCredentials::UserBased { .. } => panic!("Expected Community credentials"),
        }
    }

    #[test]
    fn test_snmp_credentials_community() {
        let creds = SnmpCredentials::Community {
            community: "private".to_string(),
        };
        match creds {
            SnmpCredentials::Community { community } => {
                assert_eq!(community, "private");
            }
            SnmpCredentials::UserBased { .. } => panic!("Expected Community credentials"),
        }
    }

    #[test]
    fn test_snmp_credentials_user_based() {
        let creds = SnmpCredentials::UserBased {
            username: "admin".to_string(),
            auth: Some(("SHA".to_string(), "auth_password".to_string())),
            privacy: Some(("AES".to_string(), "priv_password".to_string())),
        };

        match creds {
            SnmpCredentials::UserBased {
                username,
                auth,
                privacy,
            } => {
                assert_eq!(username, "admin");
                assert!(auth.is_some());
                assert!(privacy.is_some());

                if let Some((auth_proto, auth_pass)) = auth {
                    assert_eq!(auth_proto, "SHA");
                    assert_eq!(auth_pass, "auth_password");
                }

                if let Some((priv_proto, priv_pass)) = privacy {
                    assert_eq!(priv_proto, "AES");
                    assert_eq!(priv_pass, "priv_password");
                }
            }
            SnmpCredentials::Community { .. } => panic!("Expected UserBased credentials"),
        }
    }

    #[test]
    fn test_snmp_credentials_user_based_no_auth() {
        let creds = SnmpCredentials::UserBased {
            username: "readonly".to_string(),
            auth: None,
            privacy: None,
        };

        match creds {
            SnmpCredentials::UserBased {
                username,
                auth,
                privacy,
            } => {
                assert_eq!(username, "readonly");
                assert!(auth.is_none());
                assert!(privacy.is_none());
            }
            SnmpCredentials::Community { .. } => panic!("Expected UserBased credentials"),
        }
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.version, 2);
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retries, 3);
        assert_eq!(config.max_vars_per_request, 10);

        // Test default credentials
        match config.credentials {
            SnmpCredentials::Community { community } => {
                assert_eq!(community, "public");
            }
            SnmpCredentials::UserBased { .. } => panic!("Expected Community credentials"),
        }
    }

    #[test]
    fn test_session_config_custom() {
        let custom_creds = SnmpCredentials::Community {
            community: "custom".to_string(),
        };

        let config = SessionConfig {
            address: SocketAddr::from_str("192.168.1.100:161").unwrap(),
            version: 3,
            credentials: custom_creds,
            timeout: Duration::from_secs(10),
            retries: 5,
            max_vars_per_request: 20,
        };

        assert_eq!(config.version, 3);
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.retries, 5);
        assert_eq!(config.max_vars_per_request, 20);
        assert_eq!(config.address.ip().to_string(), "192.168.1.100");
        assert_eq!(config.address.port(), 161);
    }

    #[test]
    fn test_snmp_client_config_default() {
        let config = SnmpClientConfig::default();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
        assert_eq!(config.session_timeout, Duration::from_secs(300));

        // Test default session config
        assert_eq!(config.default_session.version, 2);
        assert_eq!(config.default_session.timeout, Duration::from_secs(5));
        assert_eq!(config.default_session.retries, 3);
    }

    #[test]
    fn test_snmp_client_config_custom() {
        let custom_session = SessionConfig {
            address: SocketAddr::from_str("10.0.0.1:1161").unwrap(),
            version: 1,
            credentials: SnmpCredentials::Community {
                community: "readonly".to_string(),
            },
            timeout: Duration::from_secs(15),
            retries: 2,
            max_vars_per_request: 5,
        };

        let config = SnmpClientConfig {
            max_connections: 50,
            default_session: custom_session,
            health_check_interval: Duration::from_secs(30),
            session_timeout: Duration::from_secs(600),
        };

        assert_eq!(config.max_connections, 50);
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
        assert_eq!(config.session_timeout, Duration::from_secs(600));
        assert_eq!(config.default_session.version, 1);
        assert_eq!(config.default_session.timeout, Duration::from_secs(15));
        assert_eq!(config.default_session.retries, 2);
        assert_eq!(config.default_session.max_vars_per_request, 5);
    }

    #[test]
    fn test_snmp_credentials_clone() {
        let creds = SnmpCredentials::Community {
            community: "test".to_string(),
        };
        let cloned = creds.clone();

        match (creds, cloned) {
            (
                SnmpCredentials::Community { community: c1 },
                SnmpCredentials::Community { community: c2 },
            ) => {
                assert_eq!(c1, c2);
            }
            _ => panic!("Clone failed"),
        }
    }
}
