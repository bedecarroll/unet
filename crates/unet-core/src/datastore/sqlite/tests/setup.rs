//! Test setup utilities for `SQLite` datastore tests

use crate::datastore::sqlite::SqliteStore;
use crate::models::{DeviceRole, Lifecycle, Node, Vendor};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DatabaseBackend, Schema, Set};
use serde_json::json;
use std::net::IpAddr;
use tempfile::NamedTempFile;
use uuid::Uuid;

/// Test database wrapper that ensures cleanup
pub struct TestDb {
    pub store: SqliteStore,
    _temp_file: NamedTempFile,
}

impl TestDb {
    /// Create a new test database instance
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create a temporary SQLite file
        let temp_file = NamedTempFile::new()?;
        let db_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());

        // Connect to the database and run migrations to set up schema
        let connection = Database::connect(&db_url).await?;
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Create tables for entities
        let stmt = schema.create_table_from_entity(crate::entities::links::Entity);
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await?;

        let stmt = schema.create_table_from_entity(crate::entities::nodes::Entity);
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await?;

        let stmt = schema.create_table_from_entity(crate::entities::locations::Entity);
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await?;

        let store = SqliteStore::from_connection(connection);

        Ok(Self {
            store,
            _temp_file: temp_file,
        })
    }
}

/// Setup a test database for use in tests
pub async fn setup_test_db() -> TestDb {
    TestDb::new().await.expect("Failed to create test database")
}

/// Create a test node in the database
pub async fn create_test_node(
    store: &SqliteStore,
    id: Uuid,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let node = Node {
        id,
        name: name.to_string(),
        domain: "test.local".to_string(),
        fqdn: format!("{name}.test.local"),
        vendor: Vendor::Cisco,
        model: "Test Device".to_string(),
        role: DeviceRole::Router,
        lifecycle: Lifecycle::Live,
        management_ip: Some(IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1))),
        location_id: None,
        platform: Some("Test Platform".to_string()),
        version: Some("1.0.0".to_string()),
        serial_number: Some("TEST123".to_string()),
        asset_tag: None,
        purchase_date: None,
        warranty_expires: None,
        custom_data: json!({}),
    };

    let active_node = crate::entities::nodes::ActiveModel {
        id: Set(node.id.to_string()),
        name: Set(node.name.clone()),
        domain: Set(Some(node.domain.clone())),
        fqdn: Set(Some(node.fqdn.clone())),
        vendor: Set(node.vendor.to_string()),
        model: Set(node.model.clone()),
        role: Set(node.role.to_string()),
        lifecycle: Set(node.lifecycle.to_string()),
        management_ip: Set(node.management_ip.map(|ip| ip.to_string())),
        location_id: Set(node.location_id.map(|id| id.to_string())),
        serial_number: Set(node.serial_number.clone()),
        asset_tag: Set(node.asset_tag.clone()),
        description: Set(None),
        custom_data: Set(Some(serde_json::to_string(&node.custom_data)?)),
        created_at: Set(Utc::now().to_rfc3339()),
        updated_at: Set(Utc::now().to_rfc3339()),
    };

    active_node.insert(store.connection()).await?;
    Ok(())
}
