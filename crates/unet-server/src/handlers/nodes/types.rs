//! Type definitions for node API endpoints

use serde::Deserialize;

/// Query parameters for listing nodes
#[derive(Debug, Deserialize)]
pub struct ListNodesQuery {
    /// Page number (1-based)
    pub page: Option<u64>,
    /// Items per page
    pub per_page: Option<u64>,
    /// Filter by lifecycle
    pub lifecycle: Option<String>,
    /// Filter by role
    pub role: Option<String>,
    /// Filter by vendor
    pub vendor: Option<String>,
    /// Include derived state in response
    pub include_status: Option<bool>,
}
