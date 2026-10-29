//! `--store` URL → boxed [`UdmEventStore`] dispatcher.

use std::sync::Arc;

use udm_eventstore::UdmEventStore;
use udm_eventstore_memory::MemoryStore;
use udm_eventstore_phycloud::{PhyCloudConfig, PhyCloudStore};
use url::Url;

use crate::{CliError, CliResult};

/// Type-erased adapter handle the CLI uses across every analysis
/// command.
pub type StoreHandle = Arc<dyn UdmEventStore>;

/// Parse `--store` URL and return the appropriate adapter.
///
/// Supported schemes:
///   - `memory:///absolute/path.ndjson`           — load NDJSON from disk
///   - `memory:./relative/path.ndjson`            — same, relative path
///   - `memory:`                                  — empty store (rare; testing)
///   - `phycloud://endpoint[?token=...]`          — PhyCloud (read path)
pub async fn from_url(url_str: &str) -> CliResult<StoreHandle> {
    if let Some(rest) = url_str.strip_prefix("memory:") {
        return load_memory(rest).await;
    }
    if url_str.starts_with("phycloud://") {
        let url = Url::parse(url_str)
            .map_err(|err| CliError::Usage(format!("invalid --store URL {url_str:?}: {err}")))?;
        return Ok(load_phycloud(&url));
    }
    Err(CliError::Usage(format!(
        "unsupported --store scheme in {url_str:?} (expected `memory:` or `phycloud://`)"
    )))
}

async fn load_memory(spec: &str) -> CliResult<StoreHandle> {
    let path = spec.trim_start_matches("//").trim_start_matches('/');
    // Two forms allowed: memory:./rel.ndjson  and  memory:///abs.ndjson
    let path = if spec.starts_with("///") {
        format!("/{path}")
    } else {
        path.to_owned()
    };
    if path.is_empty() {
        return Ok(Arc::new(MemoryStore::from_events(Vec::new())));
    }
    let store = MemoryStore::from_ndjson_path(&path).await?;
    Ok(Arc::new(store))
}

fn load_phycloud(url: &Url) -> StoreHandle {
    let endpoint = format!(
        "{}://{}{}",
        url.scheme().replace("phycloud", "https"),
        url.host_str().unwrap_or(""),
        url.path()
    );
    let token = url
        .query_pairs()
        .find(|(k, _)| k == "token")
        .map_or_else(String::new, |(_, v)| v.into_owned());
    Arc::new(PhyCloudStore::new(PhyCloudConfig::new(endpoint, token)))
}
