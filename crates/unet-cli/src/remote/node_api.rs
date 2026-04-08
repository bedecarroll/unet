//! Shared helpers for remote node API calls.

use anyhow::Result;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::str::FromStr;
use unet_core::models::{
    Node,
    derived::{InterfaceStatus, NodeStatus, PerformanceMetrics},
};

use super::{RemoteClient, RemoteClientError};

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct RemoteNodeResponse {
    #[serde(flatten)]
    pub(super) node: Node,
}

#[derive(Debug, Deserialize)]
pub(super) struct RemotePage<T> {
    pub(super) data: Vec<T>,
    pub(super) total: u64,
    pub(super) page: u64,
    pub(super) per_page: u64,
    pub(super) total_pages: u64,
    pub(super) has_next: bool,
    pub(super) has_prev: bool,
}

pub(super) async fn fetch_node(
    client: &RemoteClient,
    node_id: uuid::Uuid,
) -> Result<RemoteNodeResponse> {
    fetch(client, format!("/api/v1/nodes/{node_id}")).await
}

pub(super) async fn fetch_status(client: &RemoteClient, node_id: uuid::Uuid) -> Result<NodeStatus> {
    fetch(client, format!("/api/v1/nodes/{node_id}/status")).await
}

pub(super) async fn fetch_interfaces(
    client: &RemoteClient,
    node_id: uuid::Uuid,
) -> Result<Vec<InterfaceStatus>> {
    fetch(client, format!("/api/v1/nodes/{node_id}/interfaces")).await
}

pub(super) async fn fetch_metrics(
    client: &RemoteClient,
    node_id: uuid::Uuid,
) -> std::result::Result<Option<PerformanceMetrics>, RemoteClientError> {
    match client
        .send(client.request(Method::GET, &format!("/api/v1/nodes/{node_id}/metrics")))
        .await
    {
        Ok(metrics) => Ok(Some(metrics)),
        Err(RemoteClientError::Api { status, .. }) if status == StatusCode::NOT_FOUND => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) fn parse_value<T>(value: &str, field: &str) -> Result<T>
where
    T: FromStr<Err = String>,
{
    value
        .parse::<T>()
        .map_err(|error| anyhow::anyhow!("Invalid {field} '{value}': {error}"))
}

async fn fetch<T>(client: &RemoteClient, path: String) -> Result<T>
where
    T: DeserializeOwned,
{
    client
        .send(client.request(Method::GET, &path))
        .await
        .map_err(Into::into)
}
