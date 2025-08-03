use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Vendor::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Vendor::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // Seed common vendors
        let vendors = [
            "Cisco", "Juniper", "Arista", "PaloAlto", "Fortinet", "Hpe", "Dell", "Extreme",
            "Mikrotik", "Ubiquiti", "Generic",
        ];
        for name in vendors {
            manager
                .get_connection()
                .execute(Statement::from_string(
                    manager.get_database_backend(),
                    format!("INSERT INTO vendor (name) VALUES ('{name}')"),
                ))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vendor::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Vendor {
    Table,
    Name,
}
