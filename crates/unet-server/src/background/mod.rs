//! Background tasks for the μNet server - split into focused modules
//!
//! This module is organized into separate modules for better maintainability.

pub use manager::BackgroundTasks;

mod manager;
mod policy_task;
mod scheduler;
