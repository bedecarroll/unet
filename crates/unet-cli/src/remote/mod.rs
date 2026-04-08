//! Remote HTTP execution for CLI commands.

use anyhow::Result;
use reqwest::{Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{Commands, OutputFormat};

mod node_api;
mod nodes;

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ApiErrorEnvelope {
    error: String,
    code: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RemoteClientError {
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Failed to decode remote response: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("{code} ({status}): {message}")]
    Api {
        status: StatusCode,
        code: String,
        message: String,
    },
}

pub type RemoteResult<T> = std::result::Result<T, RemoteClientError>;

#[derive(Clone)]
pub struct RemoteClient {
    http: Client,
    base_url: String,
    token: Option<String>,
}

impl RemoteClient {
    pub(crate) fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
        Ok(Self {
            http: Client::builder().build()?,
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.map(str::to_string),
        })
    }

    pub(crate) fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let builder = self
            .http
            .request(method, format!("{}{}", self.base_url, path));

        if let Some(token) = self.token.as_deref() {
            builder.bearer_auth(token)
        } else {
            builder
        }
    }

    pub(crate) async fn send<T>(&self, builder: reqwest::RequestBuilder) -> RemoteResult<T>
    where
        T: DeserializeOwned,
    {
        let response = builder.send().await?;
        let status = response.status();
        let body = response.bytes().await?;

        if status.is_success() {
            let payload: ApiEnvelope<T> = serde_json::from_slice(&body)?;
            return Ok(payload.data);
        }

        if let Ok(error) = serde_json::from_slice::<ApiErrorEnvelope>(&body) {
            return Err(RemoteClientError::Api {
                status,
                code: error.code,
                message: error.error,
            });
        }

        Err(RemoteClientError::Api {
            status,
            code: format!("HTTP_{}", status.as_u16()),
            message: String::from_utf8_lossy(&body).into_owned(),
        })
    }
}

pub async fn dispatch(
    command: Commands,
    server_url: &str,
    token: Option<&str>,
    output: OutputFormat,
) -> Result<()> {
    let client = RemoteClient::new(server_url, token)?;

    match command {
        Commands::Nodes(command) => nodes::dispatch(command, &client, output).await,
        _ => Err(anyhow::anyhow!(
            "Remote mode currently supports node commands backed by the server API"
        )),
    }
}

pub fn parse_json_arg(
    value: Option<String>,
) -> Result<Option<serde_json::Value>, serde_json::Error> {
    value.map(|raw| serde_json::from_str(&raw)).transpose()
}

pub fn print_remote_output<T: Serialize>(data: &T, output: OutputFormat) -> Result<()> {
    crate::commands::print_output(data, output)
}
