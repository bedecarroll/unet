use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create template table for template metadata
        manager
            .create_table(
                Table::create()
                    .table(Template::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Template::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(Template::Name).text().not_null())
                    .col(ColumnDef::new(Template::Path).text().not_null())
                    .col(ColumnDef::new(Template::Description).text())
                    .col(ColumnDef::new(Template::Vendor).text())
                    .col(ColumnDef::new(Template::Type).text().not_null())
                    .col(ColumnDef::new(Template::Version).text().not_null())
                    .col(ColumnDef::new(Template::GitRepository).text())
                    .col(ColumnDef::new(Template::GitBranch).text())
                    .col(ColumnDef::new(Template::GitCommit).text())
                    .col(ColumnDef::new(Template::ContentHash).text())
                    .col(ColumnDef::new(Template::MatchHeaders).text())
                    .col(
                        ColumnDef::new(Template::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Template::CustomData).text())
                    .col(
                        ColumnDef::new(Template::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(Template::UpdatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create template_assignment table for tracking node-template assignments
        manager
            .create_table(
                Table::create()
                    .table(TemplateAssignment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TemplateAssignment::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TemplateAssignment::NodeId).text().not_null())
                    .col(
                        ColumnDef::new(TemplateAssignment::TemplateId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TemplateAssignment::AssignmentType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TemplateAssignment::Priority)
                            .integer()
                            .not_null()
                            .default(100),
                    )
                    .col(
                        ColumnDef::new(TemplateAssignment::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(TemplateAssignment::ConfigSection).text())
                    .col(ColumnDef::new(TemplateAssignment::Variables).text())
                    .col(ColumnDef::new(TemplateAssignment::CustomData).text())
                    .col(
                        ColumnDef::new(TemplateAssignment::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .col(
                        ColumnDef::new(TemplateAssignment::UpdatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_assignment_node")
                            .from(TemplateAssignment::Table, TemplateAssignment::NodeId)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_assignment_template")
                            .from(TemplateAssignment::Table, TemplateAssignment::TemplateId)
                            .to(Template::Table, Template::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create template_version table for version control
        manager
            .create_table(
                Table::create()
                    .table(TemplateVersion::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TemplateVersion::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TemplateVersion::TemplateId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TemplateVersion::Version).text().not_null())
                    .col(ColumnDef::new(TemplateVersion::GitCommit).text().not_null())
                    .col(
                        ColumnDef::new(TemplateVersion::ContentHash)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TemplateVersion::Content).text().not_null())
                    .col(ColumnDef::new(TemplateVersion::ChangeLog).text())
                    .col(
                        ColumnDef::new(TemplateVersion::IsStable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(TemplateVersion::CustomData).text())
                    .col(
                        ColumnDef::new(TemplateVersion::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_version_template")
                            .from(TemplateVersion::Table, TemplateVersion::TemplateId)
                            .to(Template::Table, Template::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create template_usage table for analytics
        manager
            .create_table(
                Table::create()
                    .table(TemplateUsage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TemplateUsage::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TemplateUsage::TemplateId).text().not_null())
                    .col(ColumnDef::new(TemplateUsage::NodeId).text())
                    .col(ColumnDef::new(TemplateUsage::Operation).text().not_null())
                    .col(ColumnDef::new(TemplateUsage::Status).text().not_null())
                    .col(ColumnDef::new(TemplateUsage::RenderTime).integer())
                    .col(ColumnDef::new(TemplateUsage::OutputSize).integer())
                    .col(ColumnDef::new(TemplateUsage::ErrorMessage).text())
                    .col(ColumnDef::new(TemplateUsage::Context).text())
                    .col(ColumnDef::new(TemplateUsage::CustomData).text())
                    .col(
                        ColumnDef::new(TemplateUsage::CreatedAt)
                            .text()
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_usage_template")
                            .from(TemplateUsage::Table, TemplateUsage::TemplateId)
                            .to(Template::Table, Template::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_template_usage_node")
                            .from(TemplateUsage::Table, TemplateUsage::NodeId)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance

        // Template indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_template_name")
                    .table(Template::Table)
                    .col(Template::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_path")
                    .table(Template::Table)
                    .col(Template::Path)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_vendor_type")
                    .table(Template::Table)
                    .col(Template::Vendor)
                    .col(Template::Type)
                    .to_owned(),
            )
            .await?;

        // Template assignment indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_template_assignment_node")
                    .table(TemplateAssignment::Table)
                    .col(TemplateAssignment::NodeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_assignment_template")
                    .table(TemplateAssignment::Table)
                    .col(TemplateAssignment::TemplateId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_assignment_unique")
                    .table(TemplateAssignment::Table)
                    .col(TemplateAssignment::NodeId)
                    .col(TemplateAssignment::TemplateId)
                    .col(TemplateAssignment::ConfigSection)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Template version indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_template_version_template")
                    .table(TemplateVersion::Table)
                    .col(TemplateVersion::TemplateId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_version_unique")
                    .table(TemplateVersion::Table)
                    .col(TemplateVersion::TemplateId)
                    .col(TemplateVersion::Version)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_version_commit")
                    .table(TemplateVersion::Table)
                    .col(TemplateVersion::GitCommit)
                    .to_owned(),
            )
            .await?;

        // Template usage indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_template_usage_template")
                    .table(TemplateUsage::Table)
                    .col(TemplateUsage::TemplateId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_usage_node")
                    .table(TemplateUsage::Table)
                    .col(TemplateUsage::NodeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_template_usage_created_at")
                    .table(TemplateUsage::Table)
                    .col(TemplateUsage::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TemplateUsage::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(TemplateVersion::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(TemplateAssignment::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Template::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Template {
    Table,
    Id,
    Name,
    Path,
    Description,
    Vendor,
    Type,
    Version,
    GitRepository,
    GitBranch,
    GitCommit,
    ContentHash,
    MatchHeaders,
    IsActive,
    CustomData,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TemplateAssignment {
    Table,
    Id,
    NodeId,
    TemplateId,
    AssignmentType,
    Priority,
    IsActive,
    ConfigSection,
    Variables,
    CustomData,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TemplateVersion {
    Table,
    Id,
    TemplateId,
    Version,
    GitCommit,
    ContentHash,
    Content,
    ChangeLog,
    IsStable,
    CustomData,
    CreatedAt,
}

#[derive(DeriveIden)]
enum TemplateUsage {
    Table,
    Id,
    TemplateId,
    NodeId,
    Operation,
    Status,
    RenderTime,
    OutputSize,
    ErrorMessage,
    Context,
    CustomData,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Id,
}
