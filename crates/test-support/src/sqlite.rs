use sea_orm::Statement;
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};
use std::sync::atomic::{AtomicU64, Ordering};
use unet_core::entities;

static DB_SEQ: AtomicU64 = AtomicU64::new(0);

/// Get a fresh in-memory `SQLite` connection with entity-based schema applied.
///
/// Each call creates its own uniquely named shared-cache database so every
/// pooled connection sees the same tables while tests stay isolated from
/// each other regardless of the test harness (cargo test, nextest, Bazel).
///
/// # Panics
///
/// Panics if the in-memory database cannot be opened or the entity schema
/// fails to apply — both indicate a broken test environment.
pub async fn entity_db() -> DatabaseConnection {
    let id = DB_SEQ.fetch_add(1, Ordering::Relaxed);
    let url = format!(
        "sqlite:file:test_support_{}_{id}?mode=memory&cache=shared",
        std::process::id()
    );
    let conn = Database::connect(&url)
        .await
        .expect("connect in-memory sqlite");
    apply_entity_schema(&conn).await.expect("apply schema");
    conn
}

async fn apply_entity_schema(
    connection: &impl ConnectionTrait,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = Schema::new(DatabaseBackend::Sqlite);

    for stmt in [
        schema.create_table_from_entity(entities::vendors::Entity),
        schema.create_table_from_entity(entities::locations::Entity),
        schema.create_table_from_entity(entities::nodes::Entity),
        schema.create_table_from_entity(entities::links::Entity),
        // Derived state tables that exist in entities
        schema.create_table_from_entity(entities::interface_status::Entity),
        schema.create_table_from_entity(entities::node_status::Entity),
        schema.create_table_from_entity(entities::polling_tasks::Entity),
    ] {
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await?;
    }
    // Seed vendors similar to migrations' expected initial data
    let vendor_names = ["Cisco", "Juniper"];
    for name in vendor_names {
        let active = entities::vendors::ActiveModel {
            name: Set(name.to_string()),
        };
        let _ = active.insert(connection).await; // ignore errors if already seeded
    }
    Ok(())
}

/// Convenience: get a `SqliteStore` bound to the shared connection.
pub async fn sqlite_store() -> unet_core::datastore::sqlite::SqliteStore {
    let conn = entity_db().await;
    unet_core::datastore::sqlite::SqliteStore::from_connection(conn)
}

/// Run a closure within a `SQLite` savepoint on the shared connection.
/// All changes are rolled back afterwards.
pub async fn with_savepoint<F, Fut, T>(name: &str, f: F) -> T
where
    F: FnOnce(unet_core::datastore::sqlite::SqliteStore) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let conn = entity_db().await;
    let backend = sea_orm::DatabaseBackend::Sqlite;
    let save = format!("SAVEPOINT {name}");
    let rollback = format!("ROLLBACK TO {name}");
    let release = format!("RELEASE {name}");
    let _ = conn.execute(Statement::from_string(backend, save)).await;
    let store = unet_core::datastore::sqlite::SqliteStore::from_connection(conn.clone());
    let out = f(store).await;
    let _ = conn
        .execute(Statement::from_string(backend, rollback))
        .await;
    let _ = conn.execute(Statement::from_string(backend, release)).await;
    out
}
