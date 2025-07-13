use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Node::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Node::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(Node::Name).text().not_null())
                    .col(ColumnDef::new(Node::Fqdn).text())
                    .col(ColumnDef::new(Node::Domain).text())
                    .col(ColumnDef::new(Node::Vendor).text().not_null())
                    .col(ColumnDef::new(Node::Model).text().not_null())
                    .col(ColumnDef::new(Node::Role).text().not_null())
                    .col(ColumnDef::new(Node::Lifecycle).text().not_null())
                    .col(ColumnDef::new(Node::SerialNumber).text())
                    .col(ColumnDef::new(Node::AssetTag).text())
                    .col(ColumnDef::new(Node::LocationId).text())
                    .col(ColumnDef::new(Node::ManagementIp).text())
                    .col(ColumnDef::new(Node::Description).text())
                    .col(ColumnDef::new(Node::CustomData).text())
                    .col(
                        ColumnDef::new(Node::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(Node::UpdatedAt)
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
                    .name("idx_node_name")
                    .table(Node::Table)
                    .col(Node::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_fqdn")
                    .table(Node::Table)
                    .col(Node::Fqdn)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_location")
                    .table(Node::Table)
                    .col(Node::LocationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_role")
                    .table(Node::Table)
                    .col(Node::Role)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_lifecycle")
                    .table(Node::Table)
                    .col(Node::Lifecycle)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Node::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Id,
    Name,
    Fqdn,
    Domain,
    Vendor,
    Model,
    Role,
    Lifecycle,
    SerialNumber,
    AssetTag,
    LocationId,
    ManagementIp,
    Description,
    CustomData,
    CreatedAt,
    UpdatedAt,
}
