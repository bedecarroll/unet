use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};
use sea_orm_migration::MigratorTrait;
use std::collections::HashMap;

use super::comparison::compare_schemas;
use super::formatting::normalize_create_table_sql;
use super::types::SchemaDifference;
use crate::Migrator;

/// Compare schemas created by migrations vs entities and return differences
async fn compare_migration_vs_entity_schemas()
-> Result<Vec<SchemaDifference>, Box<dyn std::error::Error>> {
    // Create two temporary in-memory SQLite databases
    let migration_db = Database::connect("sqlite::memory:").await?;
    let entity_db = Database::connect("sqlite::memory:").await?;

    // Apply migrations to first database
    Migrator::up(&migration_db, None).await?;

    // Apply entity-based schema to second database
    create_schema_from_entities(&entity_db).await?;

    // Extract CREATE TABLE statements from both databases
    let migration_schema = extract_create_table_statements(&migration_db).await?;
    let entity_schema = extract_create_table_statements(&entity_db).await?;

    // Compare schemas and return differences
    compare_schemas(&migration_schema, &entity_schema)
}

/// Create database schema using entity definitions (not migrations)
async fn create_schema_from_entities(
    connection: &impl ConnectionTrait,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = Schema::new(DatabaseBackend::Sqlite);

    // Create tables for all entities that have corresponding migrations
    // TODO: This will fail initially because we haven't imported entity modules yet

    // Create locations table
    let stmt = schema.create_table_from_entity(unet_core::entities::locations::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create nodes table
    let stmt = schema.create_table_from_entity(unet_core::entities::nodes::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create links table
    let stmt = schema.create_table_from_entity(unet_core::entities::links::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create derived state tables (interface_status, node_status, polling_tasks)
    let stmt = schema.create_table_from_entity(unet_core::entities::interface_status::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    let stmt = schema.create_table_from_entity(unet_core::entities::node_status::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    let stmt = schema.create_table_from_entity(unet_core::entities::polling_tasks::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create vendors table
    let stmt = schema.create_table_from_entity(unet_core::entities::vendors::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    Ok(())
}

/// Extract CREATE TABLE statements from a `SQLite` database
async fn extract_create_table_statements(
    connection: &impl ConnectionTrait,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use sea_orm::Statement;

    // Query SQLite's sqlite_master table to get CREATE TABLE statements
    let query = Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name".to_string()
    );

    let raw_result = connection.query_all(query).await?;
    let mut statements = HashMap::new();

    for row in raw_result {
        let table_name: String = row.try_get("", "name")?;
        let create_sql: String = row.try_get("", "sql")?;

        statements.insert(table_name, normalize_create_table_sql(&create_sql));
    }

    Ok(statements)
}
