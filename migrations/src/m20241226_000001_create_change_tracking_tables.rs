use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create configuration_changes table for tracking all configuration changes
        manager
            .create_table(
                Table::create()
                    .table(ConfigurationChange::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConfigurationChange::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ConfigurationChange::ChangeType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConfigurationChange::EntityType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConfigurationChange::EntityId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ConfigurationChange::UserId).text())
                    .col(
                        ColumnDef::new(ConfigurationChange::Source)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ConfigurationChange::Description).text())
                    .col(ColumnDef::new(ConfigurationChange::OldValue).text())
                    .col(ColumnDef::new(ConfigurationChange::NewValue).text())
                    .col(ColumnDef::new(ConfigurationChange::DiffContent).text())
                    .col(ColumnDef::new(ConfigurationChange::GitCommit).text())
                    .col(ColumnDef::new(ConfigurationChange::GitBranch).text())
                    .col(
                        ColumnDef::new(ConfigurationChange::Status)
                            .text()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(ConfigurationChange::ApprovalRequired)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(ConfigurationChange::ApprovedBy).text())
                    .col(ColumnDef::new(ConfigurationChange::ApprovedAt).text())
                    .col(ColumnDef::new(ConfigurationChange::AppliedAt).text())
                    .col(ColumnDef::new(ConfigurationChange::RolledBackAt).text())
                    .col(ColumnDef::new(ConfigurationChange::CustomData).text())
                    .col(
                        ColumnDef::new(ConfigurationChange::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(ConfigurationChange::UpdatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create change_audit_log table for detailed audit trails
        manager
            .create_table(
                Table::create()
                    .table(ChangeAuditLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChangeAuditLog::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChangeAuditLog::ChangeId).text().not_null())
                    .col(ColumnDef::new(ChangeAuditLog::Action).text().not_null())
                    .col(ColumnDef::new(ChangeAuditLog::ActorId).text())
                    .col(ColumnDef::new(ChangeAuditLog::ActorType).text().not_null())
                    .col(ColumnDef::new(ChangeAuditLog::Details).text())
                    .col(ColumnDef::new(ChangeAuditLog::Metadata).text())
                    .col(ColumnDef::new(ChangeAuditLog::IpAddress).text())
                    .col(ColumnDef::new(ChangeAuditLog::UserAgent).text())
                    .col(
                        ColumnDef::new(ChangeAuditLog::Timestamp)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_log_change")
                            .from(ChangeAuditLog::Table, ChangeAuditLog::ChangeId)
                            .to(ConfigurationChange::Table, ConfigurationChange::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create change_approval_workflow table for approval processes
        manager
            .create_table(
                Table::create()
                    .table(ChangeApprovalWorkflow::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChangeApprovalWorkflow::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChangeApprovalWorkflow::ChangeId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeApprovalWorkflow::WorkflowType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeApprovalWorkflow::Status)
                            .text()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(ChangeApprovalWorkflow::RequiredApprovers).text())
                    .col(ColumnDef::new(ChangeApprovalWorkflow::CurrentApprovers).text())
                    .col(ColumnDef::new(ChangeApprovalWorkflow::Rules).text())
                    .col(ColumnDef::new(ChangeApprovalWorkflow::Comments).text())
                    .col(ColumnDef::new(ChangeApprovalWorkflow::ExpiresAt).text())
                    .col(
                        ColumnDef::new(ChangeApprovalWorkflow::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(ChangeApprovalWorkflow::UpdatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_approval_workflow_change")
                            .from(
                                ChangeApprovalWorkflow::Table,
                                ChangeApprovalWorkflow::ChangeId,
                            )
                            .to(ConfigurationChange::Table, ConfigurationChange::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create change_rollback_snapshots table for rollback capabilities
        manager
            .create_table(
                Table::create()
                    .table(ChangeRollbackSnapshot::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::ChangeId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::EntityType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::EntityId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::SnapshotType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::StateSnapshot)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::Checksum)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChangeRollbackSnapshot::Dependencies).text())
                    .col(ColumnDef::new(ChangeRollbackSnapshot::Metadata).text())
                    .col(
                        ColumnDef::new(ChangeRollbackSnapshot::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rollback_snapshot_change")
                            .from(
                                ChangeRollbackSnapshot::Table,
                                ChangeRollbackSnapshot::ChangeId,
                            )
                            .to(ConfigurationChange::Table, ConfigurationChange::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance

        // Configuration changes indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_config_change_entity")
                    .table(ConfigurationChange::Table)
                    .col(ConfigurationChange::EntityType)
                    .col(ConfigurationChange::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_config_change_status")
                    .table(ConfigurationChange::Table)
                    .col(ConfigurationChange::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_config_change_created_at")
                    .table(ConfigurationChange::Table)
                    .col(ConfigurationChange::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_config_change_user")
                    .table(ConfigurationChange::Table)
                    .col(ConfigurationChange::UserId)
                    .to_owned(),
            )
            .await?;

        // Audit log indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_log_change")
                    .table(ChangeAuditLog::Table)
                    .col(ChangeAuditLog::ChangeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_log_timestamp")
                    .table(ChangeAuditLog::Table)
                    .col(ChangeAuditLog::Timestamp)
                    .to_owned(),
            )
            .await?;

        // Approval workflow indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_approval_workflow_change")
                    .table(ChangeApprovalWorkflow::Table)
                    .col(ChangeApprovalWorkflow::ChangeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_approval_workflow_status")
                    .table(ChangeApprovalWorkflow::Table)
                    .col(ChangeApprovalWorkflow::Status)
                    .to_owned(),
            )
            .await?;

        // Rollback snapshot indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_rollback_snapshot_change")
                    .table(ChangeRollbackSnapshot::Table)
                    .col(ChangeRollbackSnapshot::ChangeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rollback_snapshot_entity")
                    .table(ChangeRollbackSnapshot::Table)
                    .col(ChangeRollbackSnapshot::EntityType)
                    .col(ChangeRollbackSnapshot::EntityId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ChangeRollbackSnapshot::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ChangeApprovalWorkflow::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ChangeAuditLog::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ConfigurationChange::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ConfigurationChange {
    Table,
    Id,
    ChangeType,
    EntityType,
    EntityId,
    UserId,
    Source,
    Description,
    OldValue,
    NewValue,
    DiffContent,
    GitCommit,
    GitBranch,
    Status,
    ApprovalRequired,
    ApprovedBy,
    ApprovedAt,
    AppliedAt,
    RolledBackAt,
    CustomData,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ChangeAuditLog {
    Table,
    Id,
    ChangeId,
    Action,
    ActorId,
    ActorType,
    Details,
    Metadata,
    IpAddress,
    UserAgent,
    Timestamp,
}

#[derive(DeriveIden)]
enum ChangeApprovalWorkflow {
    Table,
    Id,
    ChangeId,
    WorkflowType,
    Status,
    RequiredApprovers,
    CurrentApprovers,
    Rules,
    Comments,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ChangeRollbackSnapshot {
    Table,
    Id,
    ChangeId,
    EntityType,
    EntityId,
    SnapshotType,
    StateSnapshot,
    Checksum,
    Dependencies,
    Metadata,
    CreatedAt,
}
