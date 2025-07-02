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
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NodeStatus::NodeId).text().not_null())
                    .col(ColumnDef::new(NodeStatus::LastUpdated).text().not_null())
                    .col(
                        ColumnDef::new(NodeStatus::Reachable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(NodeStatus::SystemInfo).text())
                    .col(ColumnDef::new(NodeStatus::Performance).text())
                    .col(ColumnDef::new(NodeStatus::Environmental).text())
                    .col(ColumnDef::new(NodeStatus::VendorMetrics).text())
                    .col(ColumnDef::new(NodeStatus::RawSnmpData).text())
                    .col(ColumnDef::new(NodeStatus::LastSnmpSuccess).text())
                    .col(ColumnDef::new(NodeStatus::LastError).text())
                    .col(
                        ColumnDef::new(NodeStatus::ConsecutiveFailures)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for node_status table
        manager
            .create_index(
                Index::create()
                    .name("idx_node_status_node_id")
                    .table(NodeStatus::Table)
                    .col(NodeStatus::NodeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_node_status_last_updated")
                    .table(NodeStatus::Table)
                    .col(NodeStatus::LastUpdated)
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
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::NodeStatusId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InterfaceStatus::Index).integer().not_null())
                    .col(ColumnDef::new(InterfaceStatus::Name).text().not_null())
                    .col(
                        ColumnDef::new(InterfaceStatus::InterfaceType)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InterfaceStatus::Mtu).integer())
                    .col(ColumnDef::new(InterfaceStatus::Speed).big_integer())
                    .col(ColumnDef::new(InterfaceStatus::PhysicalAddress).text())
                    .col(
                        ColumnDef::new(InterfaceStatus::AdminStatus)
                            .text()
                            .not_null()
                            .default("unknown"),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::OperStatus)
                            .text()
                            .not_null()
                            .default("unknown"),
                    )
                    .col(ColumnDef::new(InterfaceStatus::LastChange).integer())
                    .col(
                        ColumnDef::new(InterfaceStatus::InputStats)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::OutputStats)
                            .text()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for interface_status table
        manager
            .create_index(
                Index::create()
                    .name("idx_interface_status_node_status_id")
                    .table(InterfaceStatus::Table)
                    .col(InterfaceStatus::NodeStatusId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_interface_status_index")
                    .table(InterfaceStatus::Table)
                    .col(InterfaceStatus::NodeStatusId)
                    .col(InterfaceStatus::Index)
                    .unique()
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
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PollingTasks::NodeId).text().not_null())
                    .col(ColumnDef::new(PollingTasks::Target).text().not_null())
                    .col(ColumnDef::new(PollingTasks::Oids).text().not_null())
                    .col(
                        ColumnDef::new(PollingTasks::IntervalSeconds)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::SessionConfig)
                            .text()
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
                    .col(ColumnDef::new(PollingTasks::CreatedAt).text().not_null())
                    .col(ColumnDef::new(PollingTasks::LastSuccess).text())
                    .col(ColumnDef::new(PollingTasks::LastError).text())
                    .col(
                        ColumnDef::new(PollingTasks::ConsecutiveFailures)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for polling_tasks table
        manager
            .create_index(
                Index::create()
                    .name("idx_polling_tasks_node_id")
                    .table(PollingTasks::Table)
                    .col(PollingTasks::NodeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_polling_tasks_enabled")
                    .table(PollingTasks::Table)
                    .col(PollingTasks::Enabled)
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
