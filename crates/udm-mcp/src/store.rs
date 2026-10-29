//! Adapter selection helper shared with `udm-cli`.

use std::sync::Arc;

use udm_eventstore::UdmEventStore;
use udm_eventstore_memory::MemoryStore;
use udm_eventstore_phycloud::{PhyCloudConfig, PhyCloudStore};
use url::Url;

/// Type-erased adapter handle the MCP server uses.
pub type StoreHandle = Arc<dyn UdmEventStore>;

/// Parse a `--store` URL into a boxed [`UdmEventStore`].
pub async fn store_from_url(url_str: &str) -> Result<StoreHandle, StoreOpenError> {
    if let Some(rest) = url_str.strip_prefix("memory:") {
        return load_memory(rest).await;
    }
    if url_str.starts_with("phycloud://") {
        let url = Url::parse(url_str)
            .map_err(|err| StoreOpenError::Url(format!("invalid URL {url_str:?}: {err}")))?;
        return Ok(load_phycloud(&url));
    }
    Err(StoreOpenError::UnknownScheme(url_str.to_owned()))
}

async fn load_memory(spec: &str) -> Result<StoreHandle, StoreOpenError> {
    let path = spec.trim_start_matches("//").trim_start_matches('/');
    let path = if spec.starts_with("///") {
        format!("/{path}")
    } else {
        path.to_owned()
    };
    if path.is_empty() {
        return Ok(Arc::new(MemoryStore::from_events(Vec::new())));
    }
    let store = MemoryStore::from_ndjson_path(&path)
        .await
        .map_err(StoreOpenError::EventStore)?;
    Ok(Arc::new(store))
}

fn load_phycloud(url: &Url) -> StoreHandle {
    // `phycloud://` maps to `https://` by default. Pass `insecure=true` to
    // target a local `http://` deployment (e.g. a dev PhyCloud on
    // `localhost:8000`).
    let insecure = url
        .query_pairs()
        .any(|(k, v)| k == "insecure" && v == "true");
    let scheme = if insecure { "http" } else { "https" };
    let mut authority = url.host_str().unwrap_or("").to_owned();
    if let Some(port) = url.port() {
        authority = format!("{authority}:{port}");
    }
    let endpoint = format!("{scheme}://{authority}{}", url.path());
    let token = url
        .query_pairs()
        .find(|(k, _)| k == "token")
        .map_or_else(String::new, |(_, v)| v.into_owned());
    let mut config = PhyCloudConfig::new(endpoint, token);
    // Scope every query to one mission when `session_id` is supplied, so a
    // shared PhyCloud can hold many missions without cross-talk.
    if let Some((_, session)) = url.query_pairs().find(|(k, _)| k == "session_id") {
        config = config.with_session(session.into_owned());
    }
    Arc::new(PhyCloudStore::new(config))
}

/// Failure modes for [`store_from_url`].
#[derive(Debug, thiserror::Error)]
pub enum StoreOpenError {
    /// Underlying eventstore error.
    #[error("eventstore: {0}")]
    EventStore(#[from] udm_eventstore::Error),
    /// Malformed URL.
    #[error("invalid store URL: {0}")]
    Url(String),
    /// Unsupported URL scheme.
    #[error("unsupported --store scheme in {0:?} (expected `memory:` or `phycloud://`)")]
    UnknownScheme(String),
}
