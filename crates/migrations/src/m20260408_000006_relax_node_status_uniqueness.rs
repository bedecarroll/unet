use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_node_status_node_id")
                    .table(NodeStatus::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_status_node_id_last_updated")
                    .table(NodeStatus::Table)
                    .col(NodeStatus::NodeId)
                    .col(NodeStatus::LastUpdated)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_node_status_node_id_last_updated")
                    .table(NodeStatus::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_status_node_id")
                    .table(NodeStatus::Table)
                    .col(NodeStatus::NodeId)
                    .unique()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum NodeStatus {
    Table,
    NodeId,
    LastUpdated,
}
