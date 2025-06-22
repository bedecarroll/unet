use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Location::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Location::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(Location::Name).text().not_null())
                    .col(ColumnDef::new(Location::LocationType).text().not_null())
                    .col(ColumnDef::new(Location::Path).text().not_null())
                    .col(ColumnDef::new(Location::ParentId).text())
                    .col(ColumnDef::new(Location::Description).text())
                    .col(ColumnDef::new(Location::Address).text())
                    .col(ColumnDef::new(Location::Coordinates).text())
                    .col(ColumnDef::new(Location::CustomData).text())
                    .col(
                        ColumnDef::new(Location::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(Location::UpdatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_location_path")
                    .table(Location::Table)
                    .col(Location::Path)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_location_parent")
                    .table(Location::Table)
                    .col(Location::ParentId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Location::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Location {
    Table,
    Id,
    Name,
    LocationType,
    Path,
    ParentId,
    Description,
    Address,
    Coordinates,
    CustomData,
    CreatedAt,
    UpdatedAt,
}
