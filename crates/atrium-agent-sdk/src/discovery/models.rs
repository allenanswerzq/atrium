//! Model discovery — probes `/v1/models` on OpenAI-compatible endpoints.
//!
//! Call [`discover_models`] with a base URL and optional API key to get
//! a list of models available on the provider. Works with OpenAI, Anthropic
//! (via bridge), Ollama, LM Studio, or any endpoint that returns the
//! standard `{ "data": [{ "id": "...", ... }] }` format.

use std::time::Duration;

use atrium_error::{Error, ErrorKind, Result};

/// Timeout for the model discovery HTTP request.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

/// A model discovered from a provider's `/v1/models` endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredModel {
    /// Model identifier (e.g. `claude-sonnet-4`, `gpt-4.1`).
    pub id: String,
    /// Human-readable display name, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Provider/owner tag (e.g. `copilot`, `claude-code`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
}

/// Probe a provider's `/v1/models` endpoint and return available models.
///
/// The `base_url` should include the `/v1` prefix (e.g. `http://localhost:5168/v1`).
pub async fn discover_models(
    base_url: &str,
    api_key: Option<&str>,
) -> Result<Vec<DiscoveredModel>> {
    let client = reqwest::Client::builder()
        .connect_timeout(DISCOVERY_TIMEOUT)
        .timeout(DISCOVERY_TIMEOUT)
        .build()
        .unwrap_or_default();

    let url = format!("{}/models", base_url.trim_end_matches('/'));

    tracing::info!(url = %url, "discovering models");

    let mut request = client.get(&url);
    if let Some(key) = api_key {
        if !key.is_empty() {
            request = request.bearer_auth(key);
        }
    }

    let response = request.send().await.map_err(|e| {
        Error::new(
            ErrorKind::Network,
            format!("model discovery request failed: {e}"),
        )
        .set_source(e)
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(Error::new(
            ErrorKind::Network,
            format!("model discovery failed: HTTP {status}"),
        )
        .with_context("body", body));
    }

    let payload: serde_json::Value = response.json().await.map_err(|e| {
        Error::new(
            ErrorKind::DataInvalid,
            format!("model discovery JSON error: {e}"),
        )
        .set_source(e)
    })?;

    let models = payload
        .get("data")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let id = model.get("id")?.as_str()?.to_owned();
            let display_name = model
                .get("name")
                .or_else(|| model.get("display_name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());
            let owned_by = model
                .get("owned_by")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());
            Some(DiscoveredModel {
                id,
                display_name,
                owned_by,
            })
        })
        .collect::<Vec<_>>();

    tracing::info!(url = %url, count = models.len(), "models discovered");

    Ok(models)
}
