use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};
use tokio::sync::OnceCell;
use unet_core::entities;
use sea_orm::Statement;

static DB_CONN: OnceCell<DatabaseConnection> = OnceCell::const_new();

/// Get a shared in-memory SQLite connection with entity-based schema created once.
pub async fn entity_db() -> DatabaseConnection {
    DB_CONN
        .get_or_init(|| async {
            let conn = Database::connect("sqlite::memory:")
                .await
                .expect("connect sqlite::memory:");
            apply_entity_schema(&conn).await.expect("apply schema");
            conn
        })
        .await
        .clone()
}

async fn apply_entity_schema(connection: &impl ConnectionTrait) -> Result<(), Box<dyn std::error::Error>> {
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
    use sea_orm::{ActiveModelTrait, Set};
    let vendor_names = ["Cisco", "Juniper"];
    for name in vendor_names {
        let active = entities::vendors::ActiveModel { name: Set(name.to_string()) };
        let _ = active.insert(connection).await; // ignore errors if already seeded
    }
    Ok(())
}

/// Convenience: get a `SqliteStore` bound to the shared connection.
pub async fn sqlite_store() -> unet_core::datastore::sqlite::SqliteStore {
    let conn = entity_db().await;
    unet_core::datastore::sqlite::SqliteStore::from_connection(conn)
}

/// Run a closure within a SQLite savepoint on the shared connection.
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
    let _ = conn
        .execute(Statement::from_string(backend, save))
        .await;
    let store = unet_core::datastore::sqlite::SqliteStore::from_connection(conn.clone());
    let out = f(store).await;
    let _ = conn
        .execute(Statement::from_string(backend, rollback))
        .await;
    let _ = conn
        .execute(Statement::from_string(backend, release))
        .await;
    out
}
