use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Link::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Link::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Link::Name).string().not_null())
                    .col(ColumnDef::new(Link::NodeAId).string().not_null())
                    .col(ColumnDef::new(Link::InterfaceA).string().not_null())
                    .col(ColumnDef::new(Link::NodeBId).string())
                    .col(ColumnDef::new(Link::InterfaceB).string())
                    .col(ColumnDef::new(Link::Capacity).big_integer())
                    .col(ColumnDef::new(Link::Utilization).double())
                    .col(ColumnDef::new(Link::IsInternetCircuit).integer().not_null())
                    .col(ColumnDef::new(Link::CircuitId).string())
                    .col(ColumnDef::new(Link::Provider).string())
                    .col(ColumnDef::new(Link::Description).string())
                    .col(ColumnDef::new(Link::CustomData).string())
                    .col(ColumnDef::new(Link::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Link::UpdatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_link_node_a")
                            .from(Link::Table, Link::NodeAId)
                            .to(Alias::new("node"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_link_node_b")
                            .from(Link::Table, Link::NodeBId)
                            .to(Alias::new("node"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_link_name")
                    .table(Link::Table)
                    .col(Link::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_link_node_a")
                    .table(Link::Table)
                    .col(Link::NodeAId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_link_node_b")
                    .table(Link::Table)
                    .col(Link::NodeBId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_link_circuit_id")
                    .table(Link::Table)
                    .col(Link::CircuitId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Link::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Link {
    Table,
    Id,
    Name,
    NodeAId,
    InterfaceA,
    NodeBId,
    InterfaceB,
    Capacity,
    Utilization,
    IsInternetCircuit,
    CircuitId,
    Provider,
    Description,
    CustomData,
    CreatedAt,
    UpdatedAt,
}
