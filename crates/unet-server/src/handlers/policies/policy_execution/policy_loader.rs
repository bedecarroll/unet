//! Policy loading logic for policy execution

use crate::error::ServerError;
use tracing::error;
use unet_core::policy::PolicyRule;
use unet_core::prelude::PolicyService;

use crate::handlers::policies::types::PolicyEvaluationRequest;

/// Load policies for a request, either from the request itself or from the policy service
pub fn load_policies_for_request(
    policy_service: &mut PolicyService,
    request: &PolicyEvaluationRequest,
) -> Result<Vec<PolicyRule>, ServerError> {
    request.policies.as_ref().map_or_else(
        || {
            policy_service.load_policies().map_err(|e| {
                error!("Failed to load policies: {}", e);
                ServerError::Internal(format!("Failed to load policies: {e}"))
            })
        },
        |policies| Ok(policies.clone()),
    )
}
