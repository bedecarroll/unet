//! CORS policy construction for the HTTP server.

use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use unet_core::{Error, Result, config::ServerConfig};

pub(super) fn build_cors_layer(config: &ServerConfig) -> Result<CorsLayer> {
    let methods = config
        .cors_methods
        .iter()
        .map(|method| parse_method(method))
        .collect::<Result<Vec<_>>>()?;
    let headers = config
        .cors_headers
        .iter()
        .map(|header| parse_header_name(header))
        .collect::<Result<Vec<_>>>()?;
    let origins = config
        .cors_origins
        .iter()
        .map(|origin| parse_origin(origin))
        .collect::<Result<Vec<_>>>()?;

    let mut layer = CorsLayer::new();
    if !methods.is_empty() {
        layer = layer.allow_methods(AllowMethods::list(methods));
    }
    if !headers.is_empty() {
        layer = layer.allow_headers(AllowHeaders::list(headers));
    }
    if !origins.is_empty() {
        layer = layer.allow_origin(AllowOrigin::list(origins));
    }

    Ok(layer)
}

fn parse_method(value: &str) -> Result<Method> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::config("CORS methods cannot contain empty entries"));
    }

    trimmed
        .parse()
        .map_err(|e| Error::config(format!("Invalid CORS method '{trimmed}': {e}")))
}

fn parse_header_name(value: &str) -> Result<HeaderName> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::config("CORS headers cannot contain empty entries"));
    }

    trimmed
        .parse()
        .map_err(|e| Error::config(format!("Invalid CORS header '{trimmed}': {e}")))
}

fn parse_origin(value: &str) -> Result<HeaderValue> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::config("CORS origins cannot contain empty entries"));
    }

    HeaderValue::from_str(trimmed)
        .map_err(|e| Error::config(format!("Invalid CORS origin '{trimmed}': {e}")))
}
