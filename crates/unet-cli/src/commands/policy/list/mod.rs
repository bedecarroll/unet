/// Policy listing functionality split into focused modules
pub mod display;
pub mod operations;

#[cfg(test)]
mod test_display;
#[cfg(test)]
mod test_operations;

// Re-export the public interface
pub use operations::{list_policies, show_policy};

// Import argument types from parent module
use super::{ListPolicyArgs, ShowPolicyArgs};
