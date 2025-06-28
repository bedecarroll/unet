pub use sea_orm_migration::prelude::*;

mod m20241221_000001_create_locations_table;
mod m20241221_000002_create_nodes_table;
mod m20241221_000003_create_links_table;
mod m20241221_000004_create_derived_state_tables;
mod m20241224_000001_create_template_tables;
mod m20241226_000001_create_change_tracking_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241221_000001_create_locations_table::Migration),
            Box::new(m20241221_000002_create_nodes_table::Migration),
            Box::new(m20241221_000003_create_links_table::Migration),
            Box::new(m20241221_000004_create_derived_state_tables::Migration),
            Box::new(m20241224_000001_create_template_tables::Migration),
            Box::new(m20241226_000001_create_change_tracking_tables::Migration),
        ]
    }
}
