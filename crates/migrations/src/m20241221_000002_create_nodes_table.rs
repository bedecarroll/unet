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
                    .col(ColumnDef::new(Node::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Node::Name).string().not_null())
                    .col(ColumnDef::new(Node::Fqdn).string())
                    .col(ColumnDef::new(Node::Domain).string())
                    .col(ColumnDef::new(Node::Vendor).string().not_null())
                    .col(ColumnDef::new(Node::Model).string().not_null())
                    .col(ColumnDef::new(Node::Role).string().not_null())
                    .col(ColumnDef::new(Node::Lifecycle).string().not_null())
                    .col(ColumnDef::new(Node::SerialNumber).string())
                    .col(ColumnDef::new(Node::AssetTag).string())
                    .col(ColumnDef::new(Node::LocationId).string())
                    .col(ColumnDef::new(Node::ManagementIp).string())
                    .col(ColumnDef::new(Node::Description).string())
                    .col(ColumnDef::new(Node::CustomData).string())
                    .col(ColumnDef::new(Node::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Node::UpdatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_node_location")
                            .from(Node::Table, Node::LocationId)
                            .to(Alias::new("location"), Alias::new("id")),
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
