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
                    .col(
                        ColumnDef::new(Location::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Location::Name).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("location_type"))
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Location::Path).string().not_null())
                    .col(ColumnDef::new(Location::ParentId).string())
                    .col(ColumnDef::new(Location::Description).string())
                    .col(ColumnDef::new(Location::Address).string())
                    .col(ColumnDef::new(Location::Coordinates).string())
                    .col(ColumnDef::new(Location::CustomData).string())
                    .col(ColumnDef::new(Location::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Location::UpdatedAt).string().not_null())
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
    Path,
    ParentId,
    Description,
    Address,
    Coordinates,
    CustomData,
    CreatedAt,
    UpdatedAt,
}
