use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create node_status table
        manager
            .create_table(
                Table::create()
                    .table(NodeStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NodeStatus::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NodeStatus::NodeId).uuid().not_null())
                    .col(
                        ColumnDef::new(NodeStatus::LastUpdated)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeStatus::Reachable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(NodeStatus::SystemInfo).json())
                    .col(ColumnDef::new(NodeStatus::Performance).json())
                    .col(ColumnDef::new(NodeStatus::Environmental).json())
                    .col(ColumnDef::new(NodeStatus::VendorMetrics).json())
                    .col(ColumnDef::new(NodeStatus::RawSnmpData).json())
                    .col(ColumnDef::new(NodeStatus::LastSnmpSuccess).timestamp_with_time_zone())
                    .col(ColumnDef::new(NodeStatus::LastError).text())
                    .col(
                        ColumnDef::new(NodeStatus::ConsecutiveFailures)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-node_status-node_id")
                            .from(NodeStatus::Table, NodeStatus::NodeId)
                            .to(Nodes::Table, Nodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-node_status-node_id")
                            .col(NodeStatus::NodeId)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx-node_status-last_updated")
                            .col(NodeStatus::LastUpdated),
                    )
                    .to_owned(),
            )
            .await?;

        // Create interface_status table
        manager
            .create_table(
                Table::create()
                    .table(InterfaceStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InterfaceStatus::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::NodeStatusId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InterfaceStatus::Index).integer().not_null())
                    .col(ColumnDef::new(InterfaceStatus::Name).string().not_null())
                    .col(
                        ColumnDef::new(InterfaceStatus::InterfaceType)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InterfaceStatus::Mtu).integer())
                    .col(ColumnDef::new(InterfaceStatus::Speed).big_integer())
                    .col(ColumnDef::new(InterfaceStatus::PhysicalAddress).string())
                    .col(
                        ColumnDef::new(InterfaceStatus::AdminStatus)
                            .string()
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::OperStatus)
                            .string()
                            .not_null()
                            .default("unknown"),
                    )
                    .col(ColumnDef::new(InterfaceStatus::LastChange).integer())
                    .col(
                        ColumnDef::new(InterfaceStatus::InputStats)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::OutputStats)
                            .json()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-interface_status-node_status_id")
                            .from(InterfaceStatus::Table, InterfaceStatus::NodeStatusId)
                            .to(NodeStatus::Table, NodeStatus::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-interface_status-node_status_id")
                            .col(InterfaceStatus::NodeStatusId),
                    )
                    .index(
                        Index::create()
                            .name("idx-interface_status-index")
                            .col(InterfaceStatus::NodeStatusId)
                            .col(InterfaceStatus::Index)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create polling_tasks table
        manager
            .create_table(
                Table::create()
                    .table(PollingTasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PollingTasks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PollingTasks::NodeId).uuid().not_null())
                    .col(ColumnDef::new(PollingTasks::Target).string().not_null())
                    .col(ColumnDef::new(PollingTasks::Oids).json().not_null())
                    .col(
                        ColumnDef::new(PollingTasks::IntervalSeconds)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::SessionConfig)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::Priority)
                            .small_integer()
                            .not_null()
                            .default(128),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PollingTasks::LastSuccess).timestamp_with_time_zone())
                    .col(ColumnDef::new(PollingTasks::LastError).text())
                    .col(
                        ColumnDef::new(PollingTasks::ConsecutiveFailures)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-polling_tasks-node_id")
                            .from(PollingTasks::Table, PollingTasks::NodeId)
                            .to(Nodes::Table, Nodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-polling_tasks-node_id")
                            .col(PollingTasks::NodeId),
                    )
                    .index(
                        Index::create()
                            .name("idx-polling_tasks-enabled")
                            .col(PollingTasks::Enabled),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        manager
            .drop_table(Table::drop().table(PollingTasks::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InterfaceStatus::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(NodeStatus::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum NodeStatus {
    Table,
    Id,
    NodeId,
    LastUpdated,
    Reachable,
    SystemInfo,
    Performance,
    Environmental,
    VendorMetrics,
    RawSnmpData,
    LastSnmpSuccess,
    LastError,
    ConsecutiveFailures,
}

#[derive(DeriveIden)]
enum InterfaceStatus {
    Table,
    Id,
    NodeStatusId,
    Index,
    Name,
    InterfaceType,
    Mtu,
    Speed,
    PhysicalAddress,
    AdminStatus,
    OperStatus,
    LastChange,
    InputStats,
    OutputStats,
}

#[derive(DeriveIden)]
enum PollingTasks {
    Table,
    Id,
    NodeId,
    Target,
    Oids,
    IntervalSeconds,
    SessionConfig,
    Priority,
    Enabled,
    CreatedAt,
    LastSuccess,
    LastError,
    ConsecutiveFailures,
}

#[derive(DeriveIden)]
enum Nodes {
    Table,
    Id,
}
