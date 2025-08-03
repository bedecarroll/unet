use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Self::create_node_status_table(manager).await?;
        Box::pin(Self::create_interface_status_table(manager)).await?;
        Self::create_polling_tasks_table(manager).await?;
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

impl Migration {
    async fn create_node_status_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        // Create node_status table
        manager
            .create_table(
                Table::create()
                    .table(NodeStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NodeStatus::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NodeStatus::NodeId).string().not_null())
                    .col(ColumnDef::new(NodeStatus::LastUpdated).string().not_null())
                    .col(ColumnDef::new(NodeStatus::Reachable).boolean().not_null())
                    .col(ColumnDef::new(NodeStatus::SystemInfo).string())
                    .col(ColumnDef::new(NodeStatus::Performance).string())
                    .col(ColumnDef::new(NodeStatus::Environmental).string())
                    .col(ColumnDef::new(NodeStatus::VendorMetrics).string())
                    .col(ColumnDef::new(NodeStatus::RawSnmpData).string())
                    .col(ColumnDef::new(NodeStatus::LastSnmpSuccess).string())
                    .col(ColumnDef::new(NodeStatus::LastError).string())
                    .col(
                        ColumnDef::new(NodeStatus::ConsecutiveFailures)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_node_status_node")
                            .from(NodeStatus::Table, NodeStatus::NodeId)
                            .to(Alias::new("node"), Alias::new("id")),
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

        Ok(())
    }

    async fn create_interface_status_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        // Create interface_status table
        manager
            .create_table(
                Table::create()
                    .table(InterfaceStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InterfaceStatus::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::NodeStatusId)
                            .string()
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
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::OperStatus)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InterfaceStatus::LastChange).integer())
                    .col(
                        ColumnDef::new(InterfaceStatus::InputStats)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InterfaceStatus::OutputStats)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_interface_status_node_status")
                            .from(InterfaceStatus::Table, InterfaceStatus::NodeStatusId)
                            .to(Alias::new("node_status"), Alias::new("id")),
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

        Ok(())
    }

    async fn create_polling_tasks_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        // Create polling_tasks table
        manager
            .create_table(
                Table::create()
                    .table(PollingTasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PollingTasks::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PollingTasks::NodeId).string().not_null())
                    .col(ColumnDef::new(PollingTasks::Target).string().not_null())
                    .col(ColumnDef::new(PollingTasks::Oids).string().not_null())
                    .col(
                        ColumnDef::new(PollingTasks::IntervalSeconds)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::SessionConfig)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PollingTasks::Priority)
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PollingTasks::Enabled).boolean().not_null())
                    .col(ColumnDef::new(PollingTasks::CreatedAt).string().not_null())
                    .col(ColumnDef::new(PollingTasks::LastSuccess).string())
                    .col(ColumnDef::new(PollingTasks::LastError).string())
                    .col(
                        ColumnDef::new(PollingTasks::ConsecutiveFailures)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_polling_tasks_node")
                            .from(PollingTasks::Table, PollingTasks::NodeId)
                            .to(Alias::new("node"), Alias::new("id")),
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
