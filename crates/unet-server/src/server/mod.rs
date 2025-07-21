//! Server configuration and startup - split into focused modules
//!
//! This module is organized into separate modules for better maintainability.

pub use app_state::AppState;
pub use middleware::run;

mod app_state;
mod middleware;
mod routes;
