//! Change Tracking Service
//!
//! This module provides the service layer for tracking configuration changes,
//! managing audit trails, approval workflows, and rollback capabilities.

use crate::datastore::DataStore;
use crate::models::change_tracking::*;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

/// Service for managing configuration changes and audit trails
#[derive(Debug, Clone)]
pub struct ChangeTrackingService<D: DataStore + Clone> {
    datastore: D,
}

impl<D: DataStore + Clone> ChangeTrackingService<D> {
    /// Create a new change tracking service
    pub fn new(datastore: D) -> Self {
        Self { datastore }
    }

    /// Track a new configuration change
    pub async fn track_change(
        &self,
        change: ConfigurationChange,
    ) -> ChangeTrackingResult<ConfigurationChange> {
        // Create rollback snapshot before applying the change
        if let Some(old_value) = &change.old_value {
            let snapshot = ChangeRollbackSnapshot {
                id: Uuid::new_v4().to_string(),
                change_id: change.id.clone(),
                entity_type: change.entity_type.clone(),
                entity_id: change.entity_id.clone(),
                snapshot_type: SnapshotType::PreChange,
                state_snapshot: old_value.clone(),
                checksum: self.calculate_checksum(old_value),
                dependencies: None,
                metadata: None,
                created_at: Utc::now(),
            };

            self.create_rollback_snapshot(snapshot).await?;
        }

        // Store the configuration change
        self.store_configuration_change(&change).await?;

        // Create audit log entry for change creation
        let audit_entry = ChangeAuditLog {
            id: Uuid::new_v4().to_string(),
            change_id: change.id.clone(),
            action: AuditAction::Created,
            actor_id: change.user_id.clone(),
            actor_type: ActorType::User,
            details: Some(format!(
                "Change created: {}",
                change.description.as_deref().unwrap_or("No description")
            )),
            metadata: None,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };

        self.create_audit_log_entry(audit_entry).await?;

        // Create approval workflow if required
        if change.approval_required {
            let workflow = ChangeApprovalWorkflow {
                id: Uuid::new_v4().to_string(),
                change_id: change.id.clone(),
                workflow_type: WorkflowType::SingleApprover, // Default workflow
                status: WorkflowStatus::Pending,
                required_approvers: None,
                current_approvers: None,
                rules: None,
                comments: None,
                expires_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.create_approval_workflow(workflow).await?;
        }

        Ok(change)
    }

    /// Get configuration change by ID
    pub async fn get_change(
        &self,
        change_id: &str,
    ) -> ChangeTrackingResult<Option<ConfigurationChange>> {
        self.get_configuration_change(change_id).await
    }

    /// Get all changes for a specific entity
    pub async fn get_changes_for_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> ChangeTrackingResult<Vec<ConfigurationChange>> {
        self.get_configuration_changes_for_entity(entity_type, entity_id)
            .await
    }

    /// Approve a configuration change
    pub async fn approve_change(
        &self,
        change_id: &str,
        approver_id: &str,
        comment: Option<String>,
    ) -> ChangeTrackingResult<ConfigurationChange> {
        let mut change = self.get_change(change_id).await?.ok_or_else(|| {
            ChangeTrackingError::ChangeNotFound {
                id: change_id.to_string(),
            }
        })?;

        // Validate status transition
        if change.status != ChangeStatus::Pending {
            return Err(ChangeTrackingError::InvalidStatusTransition {
                from: format!("{:?}", change.status),
                to: "Approved".to_string(),
            });
        }

        // Update change status
        change.status = ChangeStatus::Approved;
        change.approved_by = Some(approver_id.to_string());
        change.approved_at = Some(Utc::now());
        change.updated_at = Utc::now();

        self.update_configuration_change(&change).await?;

        // Create audit log entry
        let audit_entry = ChangeAuditLog {
            id: Uuid::new_v4().to_string(),
            change_id: change_id.to_string(),
            action: AuditAction::Approved,
            actor_id: Some(approver_id.to_string()),
            actor_type: ActorType::User,
            details: comment,
            metadata: None,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };

        self.create_audit_log_entry(audit_entry).await?;

        // Update approval workflow
        if let Some(workflow) = self.get_approval_workflow_for_change(change_id).await? {
            let mut updated_workflow = workflow;
            updated_workflow.status = WorkflowStatus::Approved;
            updated_workflow.current_approvers = Some(format!("[\"{}\"]", approver_id));
            updated_workflow.updated_at = Utc::now();

            self.update_approval_workflow(&updated_workflow).await?;
        }

        Ok(change)
    }

    /// Apply a configuration change
    pub async fn apply_change(&self, change_id: &str) -> ChangeTrackingResult<ConfigurationChange> {
        let mut change = self.get_change(change_id).await?.ok_or_else(|| {
            ChangeTrackingError::ChangeNotFound {
                id: change_id.to_string(),
            }
        })?;

        // Validate status and approval
        if change.approval_required && change.status != ChangeStatus::Approved {
            return Err(ChangeTrackingError::ApprovalRequired {
                id: change_id.to_string(),
            });
        }

        if ![ChangeStatus::Pending, ChangeStatus::Approved].contains(&change.status) {
            return Err(ChangeTrackingError::InvalidStatusTransition {
                from: format!("{:?}", change.status),
                to: "Applied".to_string(),
            });
        }

        // Update change status
        change.status = ChangeStatus::Applied;
        change.applied_at = Some(Utc::now());
        change.updated_at = Utc::now();

        self.update_configuration_change(&change).await?;

        // Create post-change snapshot
        if let Some(new_value) = &change.new_value {
            let snapshot = ChangeRollbackSnapshot {
                id: Uuid::new_v4().to_string(),
                change_id: change.id.clone(),
                entity_type: change.entity_type.clone(),
                entity_id: change.entity_id.clone(),
                snapshot_type: SnapshotType::PostChange,
                state_snapshot: new_value.clone(),
                checksum: self.calculate_checksum(new_value),
                dependencies: None,
                metadata: None,
                created_at: Utc::now(),
            };

            self.create_rollback_snapshot(snapshot).await?;
        }

        // Create audit log entry
        let audit_entry = ChangeAuditLog {
            id: Uuid::new_v4().to_string(),
            change_id: change_id.to_string(),
            action: AuditAction::Applied,
            actor_id: None, // System applied
            actor_type: ActorType::System,
            details: Some("Change applied successfully".to_string()),
            metadata: None,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };

        self.create_audit_log_entry(audit_entry).await?;

        Ok(change)
    }

    /// Rollback a configuration change
    pub async fn rollback_change(
        &self,
        change_id: &str,
        user_id: &str,
    ) -> ChangeTrackingResult<ConfigurationChange> {
        let mut change = self.get_change(change_id).await?.ok_or_else(|| {
            ChangeTrackingError::ChangeNotFound {
                id: change_id.to_string(),
            }
        })?;

        // Validate that change can be rolled back
        if change.status != ChangeStatus::Applied {
            return Err(ChangeTrackingError::InvalidStatusTransition {
                from: format!("{:?}", change.status),
                to: "RolledBack".to_string(),
            });
        }

        // Get rollback snapshot
        let snapshot = self
            .get_rollback_snapshot_for_change(change_id, SnapshotType::PreChange)
            .await?
            .ok_or_else(|| ChangeTrackingError::SnapshotFailed {
                reason: "No pre-change snapshot found".to_string(),
            })?;

        // Update change status
        change.status = ChangeStatus::RolledBack;
        change.rolled_back_at = Some(Utc::now());
        change.updated_at = Utc::now();

        self.update_configuration_change(&change).await?;

        // Create audit log entry
        let audit_entry = ChangeAuditLog {
            id: Uuid::new_v4().to_string(),
            change_id: change_id.to_string(),
            action: AuditAction::RolledBack,
            actor_id: Some(user_id.to_string()),
            actor_type: ActorType::User,
            details: Some("Change rolled back to previous state".to_string()),
            metadata: Some(format!("{{\"snapshot_id\": \"{}\"}}", snapshot.id)),
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
        };

        self.create_audit_log_entry(audit_entry).await?;

        Ok(change)
    }

    /// Get audit trail for a change
    pub async fn get_audit_trail(
        &self,
        change_id: &str,
    ) -> ChangeTrackingResult<Vec<ChangeAuditLog>> {
        self.get_audit_log_entries_for_change(change_id).await
    }

    /// Get change history for an entity
    pub async fn get_change_history(
        &self,
        entity_type: &str,
        entity_id: &str,
        limit: Option<usize>,
    ) -> ChangeTrackingResult<Vec<ConfigurationChange>> {
        let mut changes = self.get_changes_for_entity(entity_type, entity_id).await?;

        // Sort by creation time, most recent first
        changes.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            changes.truncate(limit);
        }

        Ok(changes)
    }

    /// Get pending changes requiring approval
    pub async fn get_pending_approvals(&self) -> ChangeTrackingResult<Vec<ConfigurationChange>> {
        self.get_configuration_changes_by_status(ChangeStatus::Pending)
            .await
    }

    /// Get detailed audit trail with analysis
    pub async fn get_detailed_audit_trail(
        &self,
        change_id: &str,
    ) -> ChangeTrackingResult<AuditTrailReport> {
        let change = self.get_change(change_id).await?.ok_or_else(|| {
            ChangeTrackingError::ChangeNotFound {
                id: change_id.to_string(),
            }
        })?;

        let audit_entries = self.get_audit_trail(change_id).await?;

        let mut report = AuditTrailReport {
            change_id: change_id.to_string(),
            entity_type: change.entity_type.clone(),
            entity_id: change.entity_id.clone(),
            total_actions: audit_entries.len(),
            timeline: audit_entries.clone(),
            approval_history: Vec::new(),
            status_changes: Vec::new(),
            rollback_history: Vec::new(),
            summary: "".to_string(),
        };

        // Analyze audit entries for patterns
        for entry in &audit_entries {
            match entry.action {
                AuditAction::Approved | AuditAction::Rejected => {
                    report.approval_history.push(entry.clone());
                }
                AuditAction::Applied
                | AuditAction::Cancelled
                | AuditAction::Created
                | AuditAction::Submitted => {
                    report.status_changes.push(entry.clone());
                }
                AuditAction::RolledBack => {
                    report.rollback_history.push(entry.clone());
                }
                _ => {}
            }
        }

        // Generate summary
        report.summary = self.generate_audit_summary(&change, &audit_entries);

        Ok(report)
    }

    /// Get change history with trend analysis
    pub async fn get_change_history_with_trends(
        &self,
        entity_type: &str,
        entity_id: &str,
        days: u32,
    ) -> ChangeTrackingResult<ChangeHistoryReport> {
        let changes = self.get_changes_for_entity(entity_type, entity_id).await?;

        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
        let recent_changes: Vec<_> = changes
            .into_iter()
            .filter(|c| c.created_at > cutoff_date)
            .collect();

        let mut report = ChangeHistoryReport {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            period_days: days,
            total_changes: recent_changes.len(),
            changes_by_status: std::collections::HashMap::new(),
            changes_by_type: std::collections::HashMap::new(),
            changes_by_source: std::collections::HashMap::new(),
            recent_changes: recent_changes.clone(),
            change_frequency: 0.0,
            most_active_period: None,
            approval_rate: 0.0,
            rollback_rate: 0.0,
        };

        // Analyze change patterns
        for change in &recent_changes {
            // Count by status
            *report
                .changes_by_status
                .entry(format!("{:?}", change.status))
                .or_insert(0) += 1;

            // Count by type
            *report
                .changes_by_type
                .entry(format!("{:?}", change.change_type))
                .or_insert(0) += 1;

            // Count by source
            *report
                .changes_by_source
                .entry(format!("{:?}", change.source))
                .or_insert(0) += 1;
        }

        // Calculate metrics
        report.change_frequency = recent_changes.len() as f64 / days as f64;

        let approved_count = report
            .changes_by_status
            .get("Approved")
            .or_else(|| report.changes_by_status.get("Applied"))
            .unwrap_or(&0);
        let total_requiring_approval = recent_changes
            .iter()
            .filter(|c| c.approval_required)
            .count();

        if total_requiring_approval > 0 {
            report.approval_rate = *approved_count as f64 / total_requiring_approval as f64;
        }

        let rollback_count = report.changes_by_status.get("RolledBack").unwrap_or(&0);
        if !recent_changes.is_empty() {
            report.rollback_rate = *rollback_count as f64 / recent_changes.len() as f64;
        }

        Ok(report)
    }

    /// Search changes across all entities with advanced filtering
    pub async fn search_changes(
        &self,
        search_params: ChangeSearchParams,
    ) -> ChangeTrackingResult<Vec<ConfigurationChange>> {
        // This would use the datastore's query capabilities
        // For now, we'll implement basic filtering logic

        let mut filters = Vec::new();

        if let Some(entity_type) = &search_params.entity_type {
            filters.push(crate::datastore::Filter {
                field: "entity_type".to_string(),
                operation: crate::datastore::FilterOperation::Equals,
                value: crate::datastore::FilterValue::String(entity_type.clone()),
            });
        }

        if let Some(status) = &search_params.status {
            filters.push(crate::datastore::Filter {
                field: "status".to_string(),
                operation: crate::datastore::FilterOperation::Equals,
                value: crate::datastore::FilterValue::String(format!("{:?}", status)),
            });
        }

        if let Some(user_id) = &search_params.user_id {
            filters.push(crate::datastore::Filter {
                field: "user_id".to_string(),
                operation: crate::datastore::FilterOperation::Equals,
                value: crate::datastore::FilterValue::String(user_id.clone()),
            });
        }

        let options = crate::datastore::QueryOptions {
            filters,
            sort: vec![crate::datastore::Sort {
                field: "created_at".to_string(),
                direction: crate::datastore::SortDirection::Descending,
            }],
            pagination: search_params.limit.map(|limit| {
                crate::datastore::Pagination::new(limit, search_params.offset.unwrap_or(0))
                    .unwrap_or_else(|_| crate::datastore::Pagination {
                        limit: 50,
                        offset: 0,
                    })
            }),
        };

        let paged_result = self
            .datastore
            .list_configuration_changes(&options)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(paged_result.items)
    }

    /// Generate comprehensive system audit report
    pub async fn generate_system_audit_report(
        &self,
        from_date: chrono::DateTime<Utc>,
        to_date: chrono::DateTime<Utc>,
    ) -> ChangeTrackingResult<SystemAuditReport> {
        // This would require more complex queries across all entities
        // Implementation would depend on the specific datastore backend

        let report = SystemAuditReport {
            report_period: format!(
                "{} to {}",
                from_date.format("%Y-%m-%d"),
                to_date.format("%Y-%m-%d")
            ),
            total_changes: 0,
            total_entities_affected: 0,
            compliance_score: 0.0,
            security_events: Vec::new(),
            top_change_sources: std::collections::HashMap::new(),
            unusual_patterns: Vec::new(),
            recommendations: Vec::new(),
        };

        // This would be implemented with specific queries for the reporting period
        // For now, return a basic report structure

        Ok(report)
    }

    /// Helper method to generate audit summary
    fn generate_audit_summary(
        &self,
        change: &ConfigurationChange,
        audit_entries: &[ChangeAuditLog],
    ) -> String {
        let mut summary = format!(
            "Change {} on {} {}",
            change.change_type, change.entity_type, change.entity_id
        );

        if !audit_entries.is_empty() {
            summary.push_str(&format!(". {} audit events recorded", audit_entries.len()));

            let approval_count = audit_entries
                .iter()
                .filter(|e| matches!(e.action, AuditAction::Approved))
                .count();

            let rejection_count = audit_entries
                .iter()
                .filter(|e| matches!(e.action, AuditAction::Rejected))
                .count();

            if approval_count > 0 {
                summary.push_str(&format!(", {} approvals", approval_count));
            }

            if rejection_count > 0 {
                summary.push_str(&format!(", {} rejections", rejection_count));
            }

            if audit_entries
                .iter()
                .any(|e| matches!(e.action, AuditAction::RolledBack))
            {
                summary.push_str(", was rolled back");
            }
        }

        summary.push('.');
        summary
    }

    /// Calculate checksum for integrity verification
    fn calculate_checksum(&self, data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // Internal datastore methods using the DataStore trait
    async fn store_configuration_change(
        &self,
        change: &ConfigurationChange,
    ) -> ChangeTrackingResult<()> {
        self.datastore
            .create_configuration_change(change)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn get_configuration_change(
        &self,
        change_id: &str,
    ) -> ChangeTrackingResult<Option<ConfigurationChange>> {
        self.datastore
            .get_configuration_change(change_id)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })
    }

    async fn get_configuration_changes_for_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> ChangeTrackingResult<Vec<ConfigurationChange>> {
        self.datastore
            .get_configuration_changes_for_entity(entity_type, entity_id)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })
    }

    async fn get_configuration_changes_by_status(
        &self,
        status: ChangeStatus,
    ) -> ChangeTrackingResult<Vec<ConfigurationChange>> {
        self.datastore
            .get_configuration_changes_by_status(status)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })
    }

    async fn update_configuration_change(
        &self,
        change: &ConfigurationChange,
    ) -> ChangeTrackingResult<()> {
        self.datastore
            .update_configuration_change(change)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn create_audit_log_entry(&self, entry: ChangeAuditLog) -> ChangeTrackingResult<()> {
        self.datastore
            .create_audit_log_entry(&entry)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn get_audit_log_entries_for_change(
        &self,
        change_id: &str,
    ) -> ChangeTrackingResult<Vec<ChangeAuditLog>> {
        self.datastore
            .get_audit_log_entries_for_change(change_id)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })
    }

    async fn create_approval_workflow(
        &self,
        workflow: ChangeApprovalWorkflow,
    ) -> ChangeTrackingResult<()> {
        self.datastore
            .create_approval_workflow(&workflow)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn get_approval_workflow_for_change(
        &self,
        change_id: &str,
    ) -> ChangeTrackingResult<Option<ChangeApprovalWorkflow>> {
        self.datastore
            .get_approval_workflow_for_change(change_id)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })
    }

    async fn update_approval_workflow(
        &self,
        workflow: &ChangeApprovalWorkflow,
    ) -> ChangeTrackingResult<()> {
        self.datastore
            .update_approval_workflow(workflow)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn create_rollback_snapshot(
        &self,
        snapshot: ChangeRollbackSnapshot,
    ) -> ChangeTrackingResult<()> {
        self.datastore
            .create_rollback_snapshot(&snapshot)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn get_rollback_snapshot_for_change(
        &self,
        change_id: &str,
        snapshot_type: SnapshotType,
    ) -> ChangeTrackingResult<Option<ChangeRollbackSnapshot>> {
        self.datastore
            .get_rollback_snapshot_for_change(change_id, snapshot_type)
            .await
            .map_err(|e| ChangeTrackingError::DatabaseError {
                message: e.to_string(),
            })
    }
}

/// Helper functions for creating configuration changes
impl<D: DataStore + Clone> ChangeTrackingService<D> {
    /// Track a node change
    pub async fn track_node_change(
        &self,
        node_id: &str,
        change_type: ChangeType,
        old_value: Option<Value>,
        new_value: Option<Value>,
        user_id: Option<String>,
        source: ChangeSource,
        description: Option<String>,
    ) -> ChangeTrackingResult<ConfigurationChange> {
        let change = ConfigurationChangeBuilder::new()
            .change_type(change_type)
            .entity("node".to_string(), node_id.to_string())
            .source(source)
            .values(
                old_value.map(|v| serde_json::to_string(&v).unwrap_or_default()),
                new_value.map(|v| serde_json::to_string(&v).unwrap_or_default()),
            )
            .description(description.unwrap_or_else(|| format!("Node {} change", change_type)))
            .approval_required(true) // Node changes require approval by default
            .build();

        if let Some(user) = user_id {
            let mut change = change;
            change.user_id = Some(user);
            self.track_change(change).await
        } else {
            self.track_change(change).await
        }
    }

    /// Track a template change
    pub async fn track_template_change(
        &self,
        template_id: &str,
        change_type: ChangeType,
        old_value: Option<Value>,
        new_value: Option<Value>,
        user_id: Option<String>,
        source: ChangeSource,
        description: Option<String>,
    ) -> ChangeTrackingResult<ConfigurationChange> {
        let change = ConfigurationChangeBuilder::new()
            .change_type(change_type)
            .entity("template".to_string(), template_id.to_string())
            .source(source)
            .values(
                old_value.map(|v| serde_json::to_string(&v).unwrap_or_default()),
                new_value.map(|v| serde_json::to_string(&v).unwrap_or_default()),
            )
            .description(description.unwrap_or_else(|| format!("Template {} change", change_type)))
            .approval_required(matches!(source, ChangeSource::Api | ChangeSource::Cli))
            .build();

        if let Some(user) = user_id {
            let mut change = change;
            change.user_id = Some(user);
            self.track_change(change).await
        } else {
            self.track_change(change).await
        }
    }

    /// Track a policy change
    pub async fn track_policy_change(
        &self,
        policy_id: &str,
        change_type: ChangeType,
        old_value: Option<Value>,
        new_value: Option<Value>,
        user_id: Option<String>,
        source: ChangeSource,
        description: Option<String>,
    ) -> ChangeTrackingResult<ConfigurationChange> {
        let change = ConfigurationChangeBuilder::new()
            .change_type(change_type)
            .entity("policy".to_string(), policy_id.to_string())
            .source(source)
            .values(
                old_value.map(|v| serde_json::to_string(&v).unwrap_or_default()),
                new_value.map(|v| serde_json::to_string(&v).unwrap_or_default()),
            )
            .description(description.unwrap_or_else(|| format!("Policy {} change", change_type)))
            .approval_required(true) // Policy changes always require approval
            .build();

        if let Some(user) = user_id {
            let mut change = change;
            change.user_id = Some(user);
            self.track_change(change).await
        } else {
            self.track_change(change).await
        }
    }
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Create => write!(f, "create"),
            ChangeType::Update => write!(f, "update"),
            ChangeType::Delete => write!(f, "delete"),
            ChangeType::BulkUpdate => write!(f, "bulk_update"),
            ChangeType::TemplateApply => write!(f, "template_apply"),
            ChangeType::PolicyApply => write!(f, "policy_apply"),
            ChangeType::GitSync => write!(f, "git_sync"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::datastore::MemoryDataStore; // TODO: MemoryDataStore implementation needed

    // #[tokio::test]
    // async fn test_change_tracking_service_creation() {
    //     let datastore = MemoryDataStore::new();
    //     let service = ChangeTrackingService::new(datastore);

    //     // Test that service can be created
    //     assert!(std::mem::size_of_val(&service) > 0);
    // }

    // #[tokio::test]
    // async fn test_track_node_change() {
    //     let datastore = MemoryDataStore::new();
    //     let service = ChangeTrackingService::new(datastore);

    //     let old_value = Some(serde_json::json!({"name": "old-router"}));
    //     let new_value = Some(serde_json::json!({"name": "new-router"}));

    //     let result = service
    //         .track_node_change(
    //             "node-123",
    //             ChangeType::Update,
    //             old_value,
    //             new_value,
    //             Some("user-456".to_string()),
    //             ChangeSource::Api,
    //             Some("Updated router name".to_string()),
    //         )
    //         .await;

    //     // Test that change tracking completes without error
    //     assert!(result.is_ok());
    // }

    // #[test]
    // fn test_calculate_checksum() {
    //     let datastore = MemoryDataStore::new();
    //     let service = ChangeTrackingService::new(datastore);

    //     let data = "test data";
    //     let checksum1 = service.calculate_checksum(data);
    //     let checksum2 = service.calculate_checksum(data);

    //     // Same data should produce same checksum
    //     assert_eq!(checksum1, checksum2);

    //     // Different data should produce different checksum
    //     let different_checksum = service.calculate_checksum("different data");
    //     assert_ne!(checksum1, different_checksum);
    // }

    #[test]
    fn test_configuration_change_builder() {
        let change = ConfigurationChangeBuilder::new()
            .change_type(ChangeType::Create)
            .entity("node".to_string(), "test-node".to_string())
            .user_id("user-123".to_string())
            .source(ChangeSource::Api)
            .description("Test change".to_string())
            .approval_required(true)
            .build();

        assert_eq!(change.change_type, ChangeType::Create);
        assert_eq!(change.entity_type, "node");
        assert_eq!(change.entity_id, "test-node");
        assert_eq!(change.user_id, Some("user-123".to_string()));
        assert_eq!(change.source, ChangeSource::Api);
        assert_eq!(change.description, Some("Test change".to_string()));
        assert!(change.approval_required);
        assert_eq!(change.status, ChangeStatus::Pending);
    }
}
