#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

pub use sea_orm_migration::prelude::*;

mod m20241221_000001_create_locations_table;
mod m20241221_000002_create_nodes_table;
mod m20241221_000003_create_links_table;
mod m20241221_000004_create_derived_state_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241221_000001_create_locations_table::Migration),
            Box::new(m20241221_000002_create_nodes_table::Migration),
            Box::new(m20241221_000003_create_links_table::Migration),
            Box::new(m20241221_000004_create_derived_state_tables::Migration),
        ]
    }
}
