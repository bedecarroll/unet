//! Server-side API authentication helpers.

use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use unet_core::config::types::AuthConfig;

use crate::api::ApiError;

#[derive(Clone, Debug)]
pub struct ApiAuth {
    enabled: bool,
    token: Option<String>,
}

impl ApiAuth {
    #[must_use]
    pub fn from_config(config: &AuthConfig) -> Self {
        Self {
            enabled: config.enabled,
            token: config.token.clone(),
        }
    }
}

pub async fn require_bearer_auth(
    State(auth): State<ApiAuth>,
    request: Request,
    next: Next,
) -> Response {
    if !auth.enabled {
        return next.run(request).await;
    }

    let Some(expected_token) = auth.token.as_deref() else {
        return unauthorized(
            "AUTH_REQUIRED",
            "Authentication is enabled but no token is configured",
        );
    };

    let Some(header_value) = request.headers().get(header::AUTHORIZATION) else {
        return unauthorized("AUTH_REQUIRED", "Missing bearer token");
    };

    let Ok(header_value) = header_value.to_str() else {
        return unauthorized("INVALID_AUTH_TOKEN", "Invalid authorization header");
    };

    let Some(token) = header_value.strip_prefix("Bearer ") else {
        return unauthorized("AUTH_REQUIRED", "Missing bearer token");
    };

    if token != expected_token {
        return unauthorized("INVALID_AUTH_TOKEN", "Invalid bearer token");
    }

    next.run(request).await
}

fn unauthorized(code: &str, message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiError::new(message.to_string(), code.to_string())),
    )
        .into_response()
}
