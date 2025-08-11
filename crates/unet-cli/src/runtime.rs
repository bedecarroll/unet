//! Runtime abstractions to decouple CLI from external systems for testing.
//! Not wired into main yet; intended for incremental adoption.

use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Abstract database connection handle (opaque to callers)
pub struct Db(pub sea_orm::DatabaseConnection);

/// Connector trait for obtaining database connections.
pub trait DbConnector: Send + Sync {
    fn connect(&self, url: &str) -> Pin<Box<dyn Future<Output = Result<Db>> + Send>>;
}

/// Migration runner abstraction.
pub trait MigrationRunner: Send + Sync {
    fn run(&self, db: &Db) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;
}

/// Default connector using `SeaORM`.
pub struct SeaOrmConnector;

#[allow(clippy::module_name_repetitions)]
impl DbConnector for SeaOrmConnector {
    fn connect(&self, url: &str) -> Pin<Box<dyn Future<Output = Result<Db>> + Send>> {
        use sea_orm::Database;
        use sea_orm::ConnectOptions;
        let url = url.to_owned();
        Box::pin(async move {
            let mut opt = ConnectOptions::new(&url);
            opt.sqlx_logging(false);
            let conn = Database::connect(opt).await?;
            Ok(Db(conn))
        })
    }
}

/// Default migration runner using our Migrator.
pub struct DefaultMigrator;

impl MigrationRunner for DefaultMigrator {
    fn run(&self, db: &Db) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        use migration::{Migrator, MigratorTrait as _};
        let conn = db.0.clone();
        Box::pin(async move {
            Migrator::up(&conn, None).await?;
            Ok(())
        })
    }
}

/// Application runtime context for dependency injection.
type ConnectFn = dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<Db>> + Send>> + Send + Sync;
type MigrateFn = dyn Fn(&Db) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync;

pub struct AppContext {
    pub connect: Box<ConnectFn>,
    pub migrate: Box<MigrateFn>,
}

impl Default for AppContext {
    fn default() -> Self {
        use sea_orm::{ConnectOptions, Database};
        let connect = Box::new(|url: &str| {
            let url = url.to_owned();
            Box::pin(async move {
                let mut opt = ConnectOptions::new(&url);
                opt.sqlx_logging(false);
                let conn = Database::connect(opt).await?;
                Ok(Db(conn))
            }) as Pin<Box<dyn Future<Output = Result<Db>> + Send>>
        });
        let migrate = Box::new(|db: &Db| {
            let conn = db.0.clone();
            Box::pin(async move {
                use migration::{Migrator, MigratorTrait as _};
                Migrator::up(&conn, None).await?;
                Ok(())
            }) as Pin<Box<dyn Future<Output = Result<()>> + Send>>
        });
        Self { connect, migrate }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopConnector;
    impl DbConnector for NoopConnector {
        fn connect(&self, _url: &str) -> Pin<Box<dyn Future<Output = Result<Db>> + Send>> {
            // Return an error-free placeholder by connecting to in-memory SQLite
            use sea_orm::{Database, ConnectOptions};
            Box::pin(async move {
                let mut opt = ConnectOptions::new("sqlite::memory:");
                opt.sqlx_logging(false);
                let conn = Database::connect(opt).await?;
                Ok(Db(conn))
            })
        }
    }

    struct NoopMigrator;
    impl MigrationRunner for NoopMigrator {
        fn run(&self, _db: &Db) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn test_noop_runtime_abstractions() {
        let db = NoopConnector.connect("sqlite::memory:").await.unwrap();
        let mig = NoopMigrator;
        assert!(mig.run(&db).await.is_ok());
    }
}
