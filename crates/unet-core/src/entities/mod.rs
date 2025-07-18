//! `SeaORM` Entities for μNet Core Database Tables

pub mod interface_status;
pub mod links;
pub mod locations;
pub mod node_status;
pub mod nodes;
pub mod polling_tasks;

pub use interface_status::Entity as InterfaceStatus;
pub use links::Entity as Links;
pub use locations::Entity as Locations;
pub use node_status::Entity as NodeStatus;
pub use nodes::Entity as Nodes;
pub use polling_tasks::Entity as PollingTasks;

#[cfg(test)]
mod tests;
