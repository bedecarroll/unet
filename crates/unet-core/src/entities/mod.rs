//! SeaORM Entities for μNet Core Database Tables

pub mod api_keys;
pub mod change_approval_workflow;
pub mod change_audit_log;
pub mod change_rollback_snapshot;
pub mod configuration_changes;
pub mod interface_status;
pub mod links;
pub mod locations;
pub mod node_status;
pub mod nodes;
pub mod polling_tasks;
pub mod roles;
pub mod template_assignments;
pub mod template_usage;
pub mod template_versions;
pub mod templates;
pub mod user_roles;
pub mod users;

pub use api_keys::Entity as ApiKeys;
pub use change_approval_workflow::Entity as ChangeApprovalWorkflow;
pub use change_audit_log::Entity as ChangeAuditLog;
pub use change_rollback_snapshot::Entity as ChangeRollbackSnapshot;
pub use configuration_changes::Entity as ConfigurationChanges;
pub use interface_status::Entity as InterfaceStatus;
pub use links::Entity as Links;
pub use locations::Entity as Locations;
pub use node_status::Entity as NodeStatus;
pub use nodes::Entity as Nodes;
pub use polling_tasks::Entity as PollingTasks;
pub use roles::Entity as Roles;
pub use template_assignments::Entity as TemplateAssignments;
pub use template_usage::Entity as TemplateUsage;
pub use template_versions::Entity as TemplateVersions;
pub use templates::Entity as Templates;
pub use user_roles::Entity as UserRoles;
pub use users::Entity as Users;
