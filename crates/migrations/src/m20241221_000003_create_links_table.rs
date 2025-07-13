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
                    .col(ColumnDef::new(Link::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(Link::Name).text().not_null())
                    .col(ColumnDef::new(Link::NodeAId).text().not_null())
                    .col(ColumnDef::new(Link::InterfaceA).text().not_null())
                    .col(ColumnDef::new(Link::NodeBId).text())
                    .col(ColumnDef::new(Link::InterfaceB).text())
                    .col(ColumnDef::new(Link::Capacity).big_integer())
                    .col(ColumnDef::new(Link::Utilization).double())
                    .col(
                        ColumnDef::new(Link::IsInternetCircuit)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Link::CircuitId).text())
                    .col(ColumnDef::new(Link::Provider).text())
                    .col(ColumnDef::new(Link::Description).text())
                    .col(ColumnDef::new(Link::CustomData).text())
                    .col(
                        ColumnDef::new(Link::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(Link::UpdatedAt)
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
